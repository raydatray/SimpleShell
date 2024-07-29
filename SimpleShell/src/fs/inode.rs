use std::{cmp::min, intrinsics::size_of, mem};
use slotmap::{new_key_type, SlotMap};

use crate::fs::block::Block;
use crate::fs::cache::Cache;
use crate::fs::free_map::Freemap;
use super::{block::{BlockSectorT, BLOCK_SECTOR_SIZE}, fs_errors::FsErrors, free_map::free_map_release};

const DIRECT_BLOCKS_COUNT: u32 = 123u32;
const INDIRECT_BLOCKS_PER_SECTOR: u32 = 128u32;
const INODE_SIGNATURE: u32 = 0x494e4f44;

new_key_type! {
  pub struct InodeKey;
}

pub struct InodeList<'b, 'a: 'b> {
  block: &'b Block<'a>,
  cache: &'b Cache,
  inner: SlotMap<InodeKey, MemoryInode>
}

pub struct MemoryInode {
  sector: BlockSectorT,
  open_cnt: u32,
  removed: bool,
  deny_write_count: u32,
  data: DiskInode
}

struct DiskInode {
  direct_blocks: [BlockSectorT; DIRECT_BLOCKS_COUNT as usize],
  indirect_block: BlockSectorT,
  doubly_indirect_block: BlockSectorT,

  is_directory: bool,
  length: u32,
  signature: u32
}

struct IndirectBlockSector {
  blocks: [BlockSectorT; INDIRECT_BLOCKS_PER_SECTOR as usize]
}

#[inline]
fn bytes_to_sectors(size: u32) -> u32 {
  size.div_ceil(BLOCK_SECTOR_SIZE)
}

impl<'b, 'a: 'b> InodeList<'b,'a> {
  pub fn new(block: &'a Block, cache: &'a Cache) -> Self {
    Self {
      block,
      cache,
      inner: SlotMap::with_key()
    }
  }

  pub fn open_inode(&mut self, sector: BlockSectorT) -> Result<InodeKey, FsErrors> {
    return match self.inner.iter().find(|(_, inode)| { inode.sector == sector }) {
      Some((inode_key, _)) =>  Ok(inode_key),
      None => {
        let memory_inode = MemoryInode::new(&self.block, &self.cache, sector)?;
        let inode_key = self.inner.insert(memory_inode);
        Ok(inode_key)
      }
    }
  }

  pub fn close_inode(&mut self, inode_key: InodeKey) -> Result<(), FsErrors> {
    return match self.inner.get_mut(inode_key) {
      Some(inode) => {
       todo!();
      },
      None => {
        todo!()
      }
    }
  }
}

impl MemoryInode {
  fn new(block: &Block, cache: &Cache, sector: BlockSectorT) -> Result<Self, FsErrors> {
    let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
    cache.read_cache_to_buffer(block, sector, &mut buffer)?;

    let disk_inode: DiskInode = unsafe { std::ptr::read(buffer.as_ptr() as *const _) };

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

  fn byte_to_sector(&self, cache: &Cache, block: &Block, pos: u32) -> Result<BlockSectorT, FsErrors> {
    if pos >= self.data.length {
      Err(FsErrors::PastEOF())
    }

    let index = pos / BLOCK_SECTOR_SIZE;
    self.data.index_to_sector(cache, block, index)
  }

  pub fn read_at(&mut self, cache: &Cache, block: &Block, buffer: &mut [u8], length: u32, offset: u32) -> Result<u32, FsErrors> {
    let mut bytes_read = 0u32;
    let mut bounce = None;

    while length > 0 {
      let sector_index = self.byte_to_sector(cache, block, offset);
      let sector_offset = offset % BLOCK_SECTOR_SIZE;

      let remaining_inode = self.get_length() - offset;
      let remaining_sector = BLOCK_SECTOR_SIZE - sector_offset;
      let remaining_min = min(remaining_inode, remaining_sector);

      let chunk_size = min(length, remaining_min);

      if chunk_size == 0 { break }

      if sector_offset == 0 && chunk_size == BLOCK_SECTOR_SIZE {
        cache.read_cache_to_buffer(block, sector_index, &mut buffer[bytes_read as usize..(bytes_read + BLOCK_SECTOR_SIZE) as usize])?;
      } else {
        if bounce.is_none() {
          bounce = Some([0u8; BLOCK_SECTOR_SIZE as usize]);
        }

        let bounce = bounce.as_mut().unwrap();
        cache.read_cache_to_buffer(block, sector_index, bounce)?;

        buffer[bytes_read as usize..(bytes_read + chunk_size) as usize].copy_from_slice(&bounce[sector_offset as usize..(sector_offset + chunk_size) as usize]);
      }
      length -= chunk_size;
      offset += chunk_size;
      bytes_read += chunk_size;
    }
    Ok(bytes_read)
  }

  pub fn write_at(&mut self, cache: &Cache, block: &Block, freemap: &Freemap, buffer: &[u8], length: u32, offset: u32) -> Result<u32, FsErrors> {
    let mut bytes_written = 0u32;
    let mut bounce = None;

    if self.deny_write_count { todo!("Return an error or just 0 bytes written?!") }

    if let Err(FsErrors::PastEOF) = self.byte_to_sector(cache, block, offset + length - 1) {
      self.data.reserve(cache, block, freemap, offset + length)?;
      self.data.length = offset + size;

      cache.write_cache_from_buffer(block, self.sector, self.data.to_bytes())?;
    }

    while length > 0 {
      let sector_index = self.byte_to_sector(cache, block, offset)?;
      let sector_offset = offset % BLOCK_SECTOR_SIZE;

      let remaining_inode = self.get_length() - offset;
      let remaining_sector = BLOCK_SECTOR_SIZE - sector_offset;
      let remaining_min = min(remaining_inode, remaining_sector);

      let chunk_size = min(length, remaining_min);

      if chunk_size == 0 { break }

      if sector_offset == 0 && chunk_size == BLOCK_SECTOR_SIZE {
        cache.write_cache_from_buffer(block, sector_index, &buffer[bytes_written as usize..(bytes_written + BLOCK_SECTOR_SIZE) as usize])?;
      } else {
        if bounce.is_none() {
          bounce = Some([0u8; BLOCK_SECTOR_SIZE as usize]);
        }

        let bounce = bounce.as_mut().unwrap();

        if sector_offset > 0 || chunk_size < remaining_sector {
          cache.read_cache_to_buffer(block, sector_index, bounce)?;
        } else {
          bounce.fill(0);
        }

        bounce[sector_offset as usize..(sector_offset + chunk_size) as usize].copy_from_slice(&buffer[bytes_written as usize..(bytes_written + chunk_size) as usize]);
        cache.write_cache_from_buffer(block, sector_index, bounce)?;
      }
      length -= chunk_size;
      offset += chunk_size;
      bytes_written += chunk_size;
      }
      Ok(bytes_written)
    }
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

  fn is_directory(&self) -> bool {
    self.data.is_directory
  }

  fn is_removed(&self) -> bool {
    self.removed
  }

  fn get_inode_number(&self) -> u32 {
    self.sector
  }

  pub fn get_length(&self) -> u32 {
    self.data.length
  }

  fn get_data_sectors(&self, cache: &mut Cache, block: &Block) -> Result<Vec<BlockSectorT>, FsErrors> {
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
        assert!(num_sectors == 0);
        return Ok(data_sectors);
      }

      let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
      cache.read_cache_to_buffer(block, self.data.indirect_block, &mut buffer)?;

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
        assert!(num_sectors == 0);
        return Ok(data_sectors);
      }

      let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
      cache.read_cache_to_buffer(block, self.data.doubly_indirect_block, &mut buffer)?;

      let doubly_indirect_block = IndirectBlockSector::new(buffer);
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

    assert!(num_sectors == 0);
    Ok(data_sectors)
  }

  fn deallocate(&self, cache: &Cache, block: &Block) -> Result<(),FsErrors> {
    let file_length = self.data.length;

    if file_length < 0 {
      todo!("REturn err");
    }

    let mut num_sectors = bytes_to_sectors(file_length);
    let mut limit;

    //Direct Blocks
    limit = min(num_sectors, DIRECT_BLOCKS_COUNT);
    for i in 0..limit {
      free_map_release(self.data.direct_blocks[i as uize], 1);
    }
    num_sectors -= limit;

    //Single Indirect Block
    limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR);
    if limit <= 0 {
      assert!(num_sectors == 0);
      Ok(())
    }
    Self::deallocate_indirect(cache, block, self.data.indirect_block, limit, 1)?;
    num_sectors -= limit;

    //Doubly indirect Blocks
    limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR * INDIRECT_BLOCKS_PER_SECTOR);
    if limit <= 0 {
      assert!(num_sectors == 0);
      Ok(())
    }
    Self::deallocate_indirect(cache, block, self.data.doubly_indirect_block, limit, 2)?;
    num_sectors -= limit;

    assert!(num_sectors == 0);
    Ok(())
  }

  fn deallocate_indirect(cache: &Cache, block: &Block, entry: BlockSectorT, num_sectors: u32, lvl: u32) -> Result<(), FsErrors> {
    assert!(lvl <= 2);

    if lvl == 0 {
      free_map_release(entry, 1);
      Ok(())
    }

    let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
    cache.read_cache_to_buffer(block, entry, &mut buffer)?;

    let indirect_block = IndirectBlockSector::new(buffer);

    let unit = if lvl == 1 { 1 } else { INDIRECT_BLOCKS_PER_SECTOR };
    let limit = num_sectors.div_ceil(unit);

    for i in 0..limit {
      let subsize = min(num_sectors, unit);
      Self::deallocate_indirect(cache, block, indirect_block.blocks[i as usize], subsize, lvl - 1)?;
      num_sectors -= subsize
    }

    assert!(num_sectors == 0);
    free_map_release(entry, 1);
    Ok(())
  }
}

impl DiskInode {
  fn to_bytes(&self) -> &[u8; 512] {
    assert_eq!(mem::size_of::<Self>(), 512);

    let bytes = unsafe {
      std::slice::from_raw_parts((self as *const Self) as *const u8, mem::size_of::<Self>())
    };

    assert!(bytes.len(), 512);
    bytes
  }

  fn index_to_sector(&self, cache: &Cache, block: &Block, index: u32) -> Result<BlockSectorT, FsErrors> {
    let (mut index_base, mut index_limit) = (0u32, DIRECT_BLOCKS_COUNT);

    //Direct Blocks
    if index < index_limit {
      return Ok(self.direct_blocks[index as usize]);
    }

    //Indirect Blocks
    index_base = index_limit;
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

  pub fn allocate(&mut self, cache: &Cache, block: &Block, freemap: &Freemap) -> Result<(), FsErrors> {
    self.reserve(cache, block, freemap, self.length)
  }

  fn reserve(&mut self, cache: &Cache, block: &Block, freemap: &Freemap, length: u32) -> Result<(), FsErrors> {
    const EMPTY_BUFFER: [u8; 512] = [0u8; BLOCK_SECTOR_SIZE as usize];

    if length < 0 {
      todo!("Return err")
    }

    let num_sectors = bytes_to_sectors(length);
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

    if num_sectors == 0 { Ok(()) }

    //Indirect Block
    {
      limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR);
      let index = Self::reserve_indirect(cache, block, freemap, limit, 1)?;
      self.indirect_block = index;
      num_sectors -= limit;
    }

    if num_sectors == 0 { Ok(()) }

    //Doubly Indirect Block
    {
      limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR * INDIRECT_BLOCKS_PER_SECTOR);
      let index = Self::reserve_indirect(cache, block, freemap, limit, 2)?;
      self.doubly_indirect_block = index;
      num_sectors -= limit;
    }

    assert!(num_sectors == 0);
    Ok(())
  }

  fn reserve_indirect(cache: &Cache, block: &Block, freemap: &Freemap, num_sectors: u32, lvl: u32) -> Result<BlockSectorT, FsErrors> {
    const EMPTY_BUFFER: [u8; 512] = [0u8; BLOCK_SECTOR_SIZE as usize];
    assert!(lvl <= 2);

    if lvl == 0 {
      let index = freemap.allocate(1)?;
      cache.write_cache_from_buffer(block, index, &EMPTY_BUFFER)?;
      Ok(index)
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

    assert!(num_sectors == 0);
    cache.write_cache_from_buffer(block, index, &indirect_block.blocks)?;
    Ok(index)
  }
}

impl IndirectBlockSector {
  fn new(buffer: [u8; 512]) -> Self {
    Self {
      blocks: {
        buffer.chunks_exact(4).map(|chunk| {
          let bytes: [u8; 4] = chunk.try_into().unwrap();
          BlockSectorT::from_ne_bytes(bytes)
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
      }
    }
  }
}
