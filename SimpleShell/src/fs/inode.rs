use std::{cmp::min, f64::consts::PI};
use slotmap::{new_key_type, SlotMap};

use crate::fs::block::Block;
use crate::fs::cache::Cache;
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
        if (inode.open_cnt -= 1)  == 0 {
          todo!()
        })
      },
      None => {
        todo!()
      }
    }
  }
}

pub struct MemoryInode {
  sector: BlockSectorT,
  open_cnt: u32,
  removed: bool,
  deny_write_count: u32,
  data: DiskInode
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

  fn byte_to_sector(&self, cache: &mut Cache, block: &Block, pos: u32) -> Result<BlockSectorT, FsErrors> {
    if pos >= self.data.length {
      todo!("Out of bounds error");
    }

    let index = pos / BLOCK_SECTOR_SIZE;
    self.data.index_to_sector(cache, block, index)
  }

  pub fn read_at() -> Result<(), FsErrors> {
    todo!()
  }

  pub fn write_memory_inode_to_disk() -> Result<(), FsErrors> {
    todo!()
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

  pub fn get_inode_length(&self) -> u32 {
    self.data.length
  }

  fn get_inode_data_sectors(&self, cache: &mut Cache, block: &Block) -> Result<Vec<BlockSectorT>, FsErrors> {
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

  fn deallocate(&self) -> Result<(),FsErrors> {
    let file_length = self.data.length;

    if file_length < 0 {
      todo!("REturn err");
    }

    let mut num_sectors = bytes_to_sectors(file_length);
    let mut limit;

    //Direct Blocks
    limit = min(num_sectors, DIRECT_BLOCKS_COUNT);
    for i in 0..limit {
      free_map_release();
    }
    num_sectors -= limit;

    //Single Indirect Block
    limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR);
    if l <= 0 {
      assert!(num_sectors == 0);
      Ok(())
    }

    Self::deallocate_indirect()?;


    //Doubly indirect Blocks
    limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR * INDIRECT_BLOCKS_PER_SECTOR);
    if l <= 0 {
      assert!(num_sectors == 0);
      Ok(())
    }

    Self::deallocate_indirect(cache, block, entry, num_sectors, lvl)


    assert!(num_sectors == 0);
    Ok(())
  }

  fn deallocate_indirect(cache: &mut Cache, block: &Block, entry: BlockSectorT, num_sectors: u32, lvl: u32) -> Result<(), FsErrors> {
    todo!()
  }
}

//We must ensure this is exactly 512 bytes
//DO NOT PACK THIS NOR C IT
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

impl DiskInode {
  fn index_to_sector (&self, cache: &mut Cache, block: &Block, index: u32) -> Result<BlockSectorT, FsErrors> {
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

  fn inode_reserve() -> Result<(), FsErrors> {

  }

  fn inode_reserve_indirect() -> Result<(), FsErrors> {

  }

  fn inode_dellocate() -> Result<(), FsErrors> {

  }

  fn inode_deallocate_indirect() -> Result<(), FsErrors> {

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
