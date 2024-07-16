use crate::fs::block::Block;

use super::{block::{BlockSectorT, BLOCK_SECTOR_SIZE}, fs_errors::FsErrors};

const CACHE_SIZE: u8 = 64u8;

struct CacheEntry {
  occupied: bool,
  disk_sector: Option<BlockSectorT>,
  buffer: Vec<u8>,
  dirty: bool,
  access: bool
}

impl CacheEntry {
  fn new() -> CacheEntry {
    CacheEntry {
      occupied: false,
      disk_sector: None,
      buffer: Vec::with_capacity(BLOCK_SECTOR_SIZE as usize),
      dirty: false,
      access: false
    }
  }

  pub fn flush_cache_entry(&mut self, block: &Block) -> Result<(), FsErrors> {
    if !self.occupied {
      todo!("Return some error here");
    }

    if self.dirty  {
      block.write_buffer_to_block(self.disk_sector.unwrap(), &self.buffer)?;
      self.dirty = false
    }
    Ok(())
  }
}

struct Cache {
  cache: Vec<CacheEntry>,
  clock: usize
}

impl Cache {
  pub fn new() -> Cache {
    Cache {
      cache: (0..CACHE_SIZE).map(|_| CacheEntry::new()).collect(),
      clock: 0
    }
  }

  pub fn close_cache(&mut self, block: &Block) -> Result<(), FsErrors> {
    for entry in self.cache.iter_mut() {
      if !entry.occupied {
        continue
      };
      entry.flush_cache_entry(block)?;
    }
    Ok(())
  }

  pub fn cache_lookup(&mut self, sector: BlockSectorT) -> Option<&mut CacheEntry> {
    for entry in self.cache.iter_mut() {
      if !entry.occupied {
        continue
      }
      if entry.disk_sector.unwrap() == sector {
        return Some(entry);
      }
    }
    None
  }

  fn evict_cache(&mut self, block: &Block) -> Result<&mut CacheEntry, FsErrors> {
    loop {
      if !self.cache[self.clock].occupied {
        return Ok(&mut self.cache[self.clock]);
      }

      if self.cache[self.clock].access {
        self.cache[self.clock].access = false;
      } else {
        break;
      }

      self.clock += 1;
      self.clock %= CACHE_SIZE as usize;
    }

    {
      let entry = &mut self.cache[self.clock];
      if entry.dirty {
        entry.flush_cache_entry(block)?;
      }
      entry.occupied = false;
    }

    return Ok(&mut self.cache[self.clock])
  }

  //Read a cache entry into memory
  pub fn read_cache_to_memory(&mut self, block: &Block, sector: BlockSectorT, buffer: &mut Vec<u8>) -> Result<(), FsErrors> {
    let slot = self.cache_lookup(sector);
    let entry: &mut CacheEntry;

    if let None = slot {
      entry = self.evict_cache(block)?;

      entry.occupied = true;
      entry.disk_sector = Some(sector);
      entry.dirty = false;
      block.read_block_to_buffer(sector, &mut entry.buffer)?;
    } else {
      entry = slot.unwrap();
    }

    entry.access = true;
    buffer.copy_from_slice(entry.buffer.as_slice());
    Ok(())
  }

  //Write a cache entry from memory
  pub fn write_cache_to_buffer(&mut self, block: &Block, sector: BlockSectorT, buffer: &Vec<u8>) -> Result<(), FsErrors> {
    let slot = self.cache_lookup(sector);
    let entry: &mut CacheEntry;

    if let None = slot {
      entry = self.evict_cache(block)?;

      entry.occupied = true;
      entry.disk_sector = Some(sector);
      entry.dirty = false;
      block.read_block_to_buffer(sector, &mut entry.buffer)?;
    } else {
      entry =slot.unwrap();
    }

    entry.access = true;
    entry.dirty = true;
    entry.buffer.copy_from_slice(buffer.as_slice());
    Ok(())
  }
}
