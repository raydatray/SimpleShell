use std::{
  cell::RefCell,
  rc::Rc,
  cmp::min,
  mem
};

use bytemuck::{
  from_bytes,
  Pod,
  Zeroable
};

use crate::fs::block::{
  Block,
  BlockSectorT,
  BLOCK_SECTOR_SIZE
};
use crate::fs::cache::Cache;
use crate::fs::free_map::Freemap;
use crate::fs::file_sys::State;
use crate::fs::fs_errors::FsErrors;

const DIRECT_BLOCKS_COUNT: u32 = 123u32;
const INDIRECT_BLOCKS_PER_SECTOR: u32 = 128u32;
const INODE_SIGNATURE: u32 = 0x494e4f44;

pub struct InodeList {
  inner: Vec<Rc<RefCell<MemoryInode>>>
}

pub struct MemoryInode {
  sector: BlockSectorT,
  open_cnt: u32, //Redundant since we wrap this type in a RC ??
  removed: bool,
  deny_write_count: u32,
  data: DiskInode
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct DiskInode {
  direct_blocks: [BlockSectorT; DIRECT_BLOCKS_COUNT as usize],
  indirect_block: BlockSectorT,
  doubly_indirect_block: BlockSectorT,

  is_directory: u8, //This is a bool,
  length: u32,
  signature: u32
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(transparent)]
struct IndirectBlockSector {
  blocks: [BlockSectorT; INDIRECT_BLOCKS_PER_SECTOR as usize]
}

#[inline]
fn bytes_to_sectors(size: u32) -> u32 {
  size.div_ceil(BLOCK_SECTOR_SIZE)
}

impl InodeList {
  pub fn new() -> Self {
    Self {
      inner: Vec::new()
    }
  }

  pub fn open_inode(&mut self, block: &Block, cache: &Cache, sector: BlockSectorT) -> Result<Rc<RefCell<MemoryInode>>, FsErrors> {
    return match self.inner.iter().find(|memory_inode| { memory_inode.borrow().sector == sector }) {
      Some(memory_inode) => {
        Ok(memory_inode.clone())
      },
      None => {
        let memory_inode = MemoryInode::new(block, cache, sector)?;
        let celled_inode = Rc::new(RefCell::new(memory_inode));
        let return_inode = celled_inode.clone();
        self.inner.push(celled_inode);

        Ok(return_inode)
      }
    }
  }

  pub fn close_inode(self: &mut Self, state: &mut State, inode: Rc<RefCell<MemoryInode>>) -> Result<(), FsErrors> {
    let mut memory_inode = inode.borrow_mut();
    let close_inode =  { memory_inode.open_cnt -= 1; memory_inode.open_cnt == 0 };

    if memory_inode.removed {
      state.freemap.release(memory_inode.sector, 1);
      memory_inode.deallocate(&state.block, &state.cache)?;
    }

    drop(memory_inode);

    if close_inode {
      return match self.inner.iter().position(|inner_inode| Rc::ptr_eq(inner_inode, &inode)) {
        Some(index) => {
          self.inner.remove(index);
          Ok(())
        },
        None => Err(todo!())
      }
    }
    Ok(())
  }
}

impl MemoryInode {
  fn new(block: &Block, cache: &Cache, sector: BlockSectorT) -> Result<Self, FsErrors> {
    let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
    cache.read_to_buffer(block, sector, &mut buffer)?;

    let disk_inode = from_bytes::<DiskInode>(&buffer).to_owned();

    Ok(
      Self {
        sector,
        open_cnt: 1u32,
        removed: false,
        deny_write_count: 0u32,
        data: disk_inode
      }
    )
  }

  fn byte_to_sector(&self, block: &Block, cache: &Cache, pos: u32) -> Result<BlockSectorT, FsErrors> {
    if pos >= self.data.length {
      return Err(FsErrors::PastEOF())
    }

    let index = pos / BLOCK_SECTOR_SIZE;
    self.data.index_to_sector(cache, block, index)
  }

  pub fn read_at(&mut self, block: &Block, cache: &Cache, buffer: &mut [u8], mut length: u32, mut offset: u32) -> Result<u32, FsErrors> {
    let mut bytes_read = 0u32;
    let mut bounce = None;

    while length > 0 {
      let sector_index = self.byte_to_sector(block, cache, offset)?;
      let sector_offset = offset % BLOCK_SECTOR_SIZE;

      let remaining_inode = self.get_length() - offset;
      let remaining_sector = BLOCK_SECTOR_SIZE - sector_offset;
      let remaining_min = min(remaining_inode, remaining_sector);

      let chunk_size = min(length, remaining_min);

      if chunk_size == 0 { break }

      if sector_offset == 0 && chunk_size == BLOCK_SECTOR_SIZE {
        cache.read_to_buffer(block, sector_index, &mut buffer[bytes_read as usize..(bytes_read + BLOCK_SECTOR_SIZE) as usize])?;
      } else {
        if bounce.is_none() {
          bounce = Some([0u8; BLOCK_SECTOR_SIZE as usize]);
        }

        let bounce = bounce.as_mut().unwrap();
        cache.read_to_buffer(block, sector_index, bounce)?;

        buffer[bytes_read as usize..(bytes_read + chunk_size) as usize].copy_from_slice(&bounce[sector_offset as usize..(sector_offset + chunk_size) as usize]);
      }
      length -= chunk_size;
      offset += chunk_size;
      bytes_read += chunk_size;
    }
    Ok(bytes_read)
  }

  pub fn write_at(&mut self, state: &mut State, buffer: &[u8], mut length: u32, mut offset: u32) -> Result<u32, FsErrors> {
    let mut bytes_written = 0u32;
    let mut bounce = None;

    if self.deny_write_count > 0 { todo!("Return an error or just 0 bytes written?!") }

    if let Err(FsErrors::PastEOF()) = self.byte_to_sector(&state.block, &state.cache, offset + length - 1) {
      self.data.reserve(&state.block, &state.cache, &mut state.freemap, offset + length)?;
      self.data.length = offset + length;

      state.cache.write_from_buffer(&state.block, self.sector, self.data.to_bytes())?;
    }

    while length > 0 {
      let sector_index = self.byte_to_sector(&state.block, &state.cache, offset)?;
      let sector_offset = offset % BLOCK_SECTOR_SIZE;

      let remaining_inode = self.get_length() - offset;
      let remaining_sector = BLOCK_SECTOR_SIZE - sector_offset;
      let remaining_min = min(remaining_inode, remaining_sector);

      let chunk_size = min(length, remaining_min);

      if chunk_size == 0 { break }

      if sector_offset == 0 && chunk_size == BLOCK_SECTOR_SIZE {
        state.cache.write_from_buffer(&state.block, sector_index, &buffer[bytes_written as usize..(bytes_written + BLOCK_SECTOR_SIZE) as usize])?;
      } else {
        if bounce.is_none() {
          bounce = Some([0u8; BLOCK_SECTOR_SIZE as usize]);
        }

        let bounce = bounce.as_mut().unwrap();

        if sector_offset > 0 || chunk_size < remaining_sector {
          state.cache.read_to_buffer(&state.block, sector_index, bounce)?;
        } else {
          bounce.fill(0);
        }

        bounce[sector_offset as usize..(sector_offset + chunk_size) as usize].copy_from_slice(&buffer[bytes_written as usize..(bytes_written + chunk_size) as usize]);
        state.cache.write_from_buffer(&state.block, sector_index, bounce)?;
      }
      length -= chunk_size;
      offset += chunk_size;
      bytes_written += chunk_size;
    }
    Ok(bytes_written)
  }

  pub fn deny_write(&mut self) -> () {
    self.deny_write_count += 1;
    assert!(self.deny_write_count <= self.open_cnt);
  }

  pub fn allow_write(&mut self) -> () {
    assert!(self.deny_write_count > 0);
    assert!(self.deny_write_count <= self.open_cnt);
    self.deny_write_count -= 1;
  }

  pub fn is_dir(&self) -> bool {
    self.data.is_directory
  }

  pub fn is_removed(&self) -> bool {
    self.removed
  }

  pub fn get_inode_number(&self) -> u32 {
    self.sector
  }

  pub fn get_length(&self) -> u32 {
    self.data.length
  }

  fn get_data_sectors(&self, block: &Block, cache: &Cache) -> Result<Vec<BlockSectorT>, FsErrors> {
    let file_length = self.data.length;

    if file_length < 0 {
      todo!("Return some error");
    }

    let mut num_sectors = bytes_to_sectors(file_length);
    let mut data_sectors = Vec::<BlockSectorT>::with_capacity(num_sectors as usize);

    let mut current_index = 0usize;
    let mut limit;

    //Direct blocks
    {
      limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR);

      for i in 0..limit {
        data_sectors[current_index] = self.data.direct_blocks[i as usize];
        current_index += 1;
      }

      num_sectors -= limit;
    }

    //Indirect Block
    {
      limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR);
      if limit <= 0 {
        assert_eq!(num_sectors, 0);
        return Ok(data_sectors);
      }

      let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
      cache.read_to_buffer(block, self.data.indirect_block, &mut buffer)?;

      let indirect_block = IndirectBlockSector::new(buffer);
      let indirect_limit = limit;

      for i in 0..indirect_limit {
        data_sectors[current_index] = indirect_block.blocks[i as usize];
        current_index += 1;
      }

      num_sectors -= limit;
    }

    //Doubly Indirect Blocks
    {
      limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR * INDIRECT_BLOCKS_PER_SECTOR);
      if limit <= 0 {
        assert_eq!(num_sectors, 0);
        return Ok(data_sectors);
      }

      let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
      cache.read_to_buffer(block, self.data.doubly_indirect_block, &mut buffer)?;


      let doubly_indirect_block = from_bytes::<IndirectBlockSector>(&buffer);
      let doubly_indirect_limit = limit.div_ceil(INDIRECT_BLOCKS_PER_SECTOR);

      let mut num_indirect_sectors = limit;

      for i in 0..doubly_indirect_limit {
        let subsize = min(num_indirect_sectors, INDIRECT_BLOCKS_PER_SECTOR);

        let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
        cache.read_cache_to_buffer(block, doubly_indirect_block.blocks[i as usize], &mut buffer)?;

        let indirect_block = IndirectBlockSector::new(buffer);
        let indirect_block_limit = subsize;

        for j in 0..indirect_block_limit {
          data_sectors[current_index] = indirect_block.blocks[j as usize];
          current_index += 1;
        }

        num_indirect_sectors -= subsize;
      }
      num_sectors -= limit;
    }

    assert_eq!(num_sectors, 0);
    Ok(data_sectors)
  }

  pub fn deallocate(&self, block: &Block, cache: &Cache) -> Result<(),FsErrors> {
    let file_length = self.data.length;

    if file_length < 0 {
      todo!("REturn err");
    }

    let mut num_sectors = bytes_to_sectors(file_length);
    let mut limit;

    //Direct Blocks
    limit = min(num_sectors, DIRECT_BLOCKS_COUNT);
    for i in 0..limit {
      free_map_release(self.data.direct_blocks[i as usize], 1);
    }
    num_sectors -= limit;

    //Single Indirect Block
    limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR);
    if limit <= 0 {
      assert_eq!(num_sectors, 0);
      return Ok(())
    }
    Self::deallocate_indirect(block, cache, self.data.indirect_block, limit, 1)?;
    num_sectors -= limit;

    //Doubly indirect Blocks
    limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR * INDIRECT_BLOCKS_PER_SECTOR);
    if limit <= 0 {
      assert_eq!(num_sectors, 0);
      return Ok(())
    }
    Self::deallocate_indirect(block, cache, self.data.doubly_indirect_block, limit, 2)?;
    num_sectors -= limit;

    assert_eq!(num_sectors, 0);
    Ok(())
  }

  fn deallocate_indirect(block: &Block, cache: &Cache, entry: BlockSectorT, mut num_sectors: u32, lvl: u32) -> Result<(), FsErrors> {
    assert!(lvl <= 2);

    if lvl == 0 {
      free_map_release(entry, 1);
      return Ok(())
    }

    let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
    cache.read_cache_to_buffer(block, entry, &mut buffer)?;

    let indirect_block = IndirectBlockSector::new(buffer);

    let unit = if lvl == 1 { 1 } else { INDIRECT_BLOCKS_PER_SECTOR };
    let limit = num_sectors.div_ceil(unit);

    for i in 0..limit {
      let subsize = min(num_sectors, unit);
      Self::deallocate_indirect(block, cache, indirect_block.blocks[i as usize], subsize, lvl - 1)?;
      num_sectors -= subsize
    }

    assert_eq!(num_sectors, 0);
    free_map_release(entry, 1);
    Ok(())
  }
}

impl DiskInode {
  const _: () = {
    let disk_inode_size = mem::size_of::<Self>();
    //assert_eq!(disk_inode_size, BLOCK_SECTOR_SIZE as usize, "Disk inodes were not {} bytes in size, actual size: {}", BLOCK_SECTOR_SIZE, disk_inode_size);
  };

  pub fn new(state: &mut State, sector: BlockSectorT, length: u32, is_directory: bool) -> Result<(), FsErrors>{
    let mut disk_inode = Self {
      direct_blocks: [0u32; DIRECT_BLOCKS_COUNT as usize],
      indirect_block: 0u32,
      doubly_indirect_block: 0u32,
      is_directory,
      length,
      signature: INODE_SIGNATURE
    };

    disk_inode.allocate(&state.block, &state.cache, &mut state.freemap)?;
    state.cache.write_cache_from_buffer(&state.block, sector, disk_inode.to_bytes())?;
    Ok(())
  }

  fn to_bytes(&self) -> &[u8] {
    assert_eq!(mem::size_of::<Self>(), 512);

    let bytes = unsafe {
      std::slice::from_raw_parts((self as *const Self) as *const u8, mem::size_of::<Self>())
    };

    assert_eq!(bytes.len(), 512);
    bytes
  }

  fn index_to_sector(&self, cache: &Cache, block: &Block, index: u32) -> Result<BlockSectorT, FsErrors> {
    let mut index_limit = DIRECT_BLOCKS_COUNT;
    let mut index_base = index_limit;

    //Direct Blocks
    if index < index_limit {
      return Ok(self.direct_blocks[index as usize]);
    }

    //Indirect Blocks
    index_limit += INDIRECT_BLOCKS_PER_SECTOR;

    if index < index_limit {
      //We allocate 512 bytes to read the indirect_sector into
      let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];

      cache.read_cache_to_buffer(block, self.indirect_block, &mut buffer)?;

      //We now coerce those 512 bytes (u8s) into 128 BlockSectorT's (u32s)
      let indirect_sector = IndirectBlockSector::new(buffer);

      return Ok(indirect_sector.blocks[(index - index_base) as usize]);
    }

    //Doubly indirect blocks
    index_base = index_limit;
    index_limit += INDIRECT_BLOCKS_PER_SECTOR * INDIRECT_BLOCKS_PER_SECTOR;

    if index < index_limit {
      let index_first = (index - index_base) / INDIRECT_BLOCKS_PER_SECTOR;
      let index_second = (index - index_base) % INDIRECT_BLOCKS_PER_SECTOR;

      let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
      cache.read_cache_to_buffer(block, self.doubly_indirect_block, &mut buffer)?;

    }
    //You fucked up
    todo!("Out of bounds error");
  }

  pub fn allocate(&mut self, block: &Block, cache: &Cache, freemap: &mut Freemap) -> Result<(), FsErrors> {
    self.reserve(cache, block, freemap, self.length)
  }

  fn reserve(&mut self, block: &Block, cache: &Cache, freemap: &mut Freemap, length: u32) -> Result<(), FsErrors> {
    const EMPTY_BUFFER: [u8; 512] = [0u8; BLOCK_SECTOR_SIZE as usize];

    if length < 0 {
      todo!("Return err")
    }

    let mut num_sectors = bytes_to_sectors(length);
    let limit;

    //Direct Blocks
    {
      limit = min(num_sectors, DIRECT_BLOCKS_COUNT);

      for i in 0..limit {
        if self.direct_blocks[i as usize] == 0 {
          let index = freemap.allocate(1)?;
          self.direct_blocks[i as usize] = index;
          cache.write_cache_from_buffer(block, self.indirect_block[i as usize], &EMPTY_BUFFER)?;
        }
      }

      num_sectors -= limit;
    }

    if num_sectors == 0 { return Ok(()) }

    //Indirect Block
    {
      limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR);
      let index = Self::reserve_indirect(cache, block, freemap, limit, 1)?;
      self.indirect_block = index;
      num_sectors -= limit;
    }

    if num_sectors == 0 { return Ok(()) }

    //Doubly Indirect Block
    {
      limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR * INDIRECT_BLOCKS_PER_SECTOR);
      let index = Self::reserve_indirect(cache, block, freemap, limit, 2)?;
      self.doubly_indirect_block = index;
      num_sectors -= limit;
    }

    assert_eq!(num_sectors, 0);
    Ok(())
  }

  fn reserve_indirect(cache: &Cache, block: &Block, freemap: &mut Freemap, mut num_sectors: u32, lvl: u32) -> Result<BlockSectorT, FsErrors> {
    const EMPTY_BUFFER: [u8; 512] = [0u8; BLOCK_SECTOR_SIZE as usize];
    assert!(lvl <= 2);

    if lvl == 0 {
      let index = freemap.allocate(1)?;
      cache.write_cache_from_buffer(block, index, &EMPTY_BUFFER)?;
      return Ok(index)
    }

    let index = freemap.allocate(cnt)?;
    cache.write_cache_from_buffer(block, index, &EMPTY_BUFFER)?;

    let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
    cache.read_cache_to_buffer(block, index, &mut buffer)?;

    let mut indirect_block = IndirectBlockSector::new(buffer);
    let unit = if lvl == 1 { 1 } else { INDIRECT_BLOCKS_PER_SECTOR };
    let limit = num_sectors.div_ceil(unit);

    for i in 0..limit {
      let subsize = min(num_sectors, unit);
      let indirect_index = Self::reserve_indirect(cache, block, freemap, subsize, lvl - 1)?;

      indirect_block.blocks[i as usize] = indirect_index;
      num_sectors -= subsize;
    }

    assert_eq!(num_sectors, 0);
    cache.write_cache_from_buffer(block, index, indirect_block.to_bytes())?;
    Ok(index)
  }
}
