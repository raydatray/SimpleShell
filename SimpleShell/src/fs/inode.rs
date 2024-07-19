use crate::fs::block::Block;
use crate::fs::cache::Cache;
use super::{block::{BlockSectorT, BLOCK_SECTOR_SIZE}, fs_errors::FsErrors};

const DIRECT_BLOCKS_COUNT: usize = 123usize;
const INDIRECT_BLOCKS_PER_SECTOR: usize = 128usize;
const INODE_SIGNATURE: usize = 0x494e4f44;

//We must ensure this is exactly 512 bytes
//DO NOT PACK THIS NOR C IT
struct DiskInode {
  direct_blocks: [BlockSectorT; DIRECT_BLOCKS_COUNT],
  indirect_block: BlockSectorT,
  doubly_indirect_block: BlockSectorT,

  is_directory: bool,
  length: u32,
  signature: u32
}

struct MemoryInode {
  //Element in inode list
  sector: BlockSectorT,
  open_cnt: usize,
  removed: bool,
  deny_write_count: usize,
  data: DiskInode
}

struct IndirectBlockSector {
  blocks: [BlockSectorT; INDIRECT_BLOCKS_PER_SECTOR]
}

#[inline]
fn bytes_to_sectors(size: u32) -> u32{
  size / BLOCK_SECTOR_SIZE as u32
}

impl DiskInode {
  fn index_to_sector (&self, cache: &mut Cache, block: &Block, index: u32) -> Result<BlockSectorT, FsErrors> {
    let (mut index_base, mut index_limit) = (0u32, DIRECT_BLOCKS_COUNT as u32);

    //Direct Blocks
    if index < index_limit {
      return Ok(self.direct_blocks[index as usize]);
    }

    //Indirect Blocks
    index_base = index_limit;
    index_limit += INDIRECT_BLOCKS_PER_SECTOR as u32;

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
    index_limit += (INDIRECT_BLOCKS_PER_SECTOR * INDIRECT_BLOCKS_PER_SECTOR) as u32;

    if index < index_limit {
      let index_first = (index - index_base) / INDIRECT_BLOCKS_PER_SECTOR as u32;
      let index_second = (index - index_base) % INDIRECT_BLOCKS_PER_SECTOR as u32;

      let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
      cache.read_cache_to_buffer(block, self.doubly_indirect_block, &mut buffer)?;

    }
    //You fucked up
    todo!("Out of bounds error");
  }
}

impl MemoryInode {
  pub fn read_memory_inode_from_disk() -> Result<Self, FsErrors> {

  }

  pub fn write_memory_inode_to_disk() -> Result<(), FsErrors> {

  }

  fn byte_to_sector(&self, cache: &mut Cache, block: &Block, pos: u32) -> Result<BlockSectorT, FsErrors> {
    if 0 <= pos && pos < self.data.length {
      let index = bytes_to_sectors(pos);
      return self.data.index_to_sector(cache, block, index)
    }
    todo!("Out of bounds error");
  }
}

impl IndirectBlockSector {
  ///Scary function with alot of shit that can go wrong
  ///Reads 4 bytes at a time, coerces them into a BlockSectorT,
  ///Collects into a vec and try_into into buffer
  ///No unsafe code, likely goes bad at from_ne_bytes
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
