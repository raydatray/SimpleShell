use std::{array::from_fn, cell::{Cell, RefCell, RefMut}};

use crate::fs::block::Block;
use super::{block::{BlockSectorT, BLOCK_SECTOR_SIZE}, fs_errors::FsErrors};

const CACHE_SIZE: usize = 64usize;

struct CacheEntry {
  occupied: bool,
  disk_sector: Option<BlockSectorT>,
  buffer: [u8; BLOCK_SECTOR_SIZE as usize],
  dirty: bool,
  access: bool
}

impl CacheEntry {
  fn new() -> CacheEntry {
    CacheEntry {
      occupied: false,
      disk_sector: None,
      buffer: [0; BLOCK_SECTOR_SIZE as usize],
      dirty: false,
      access: false
    }
  }

  pub fn flush_cache_entry(&mut self, block: &Block) -> Result<(), FsErrors> {
    if !self.occupied {
      todo!("Return some error here");
    }

    if self.dirty {
      block.write_buffer_to_block(self.disk_sector.unwrap(), &self.buffer)?;
      self.dirty = false
    }
    Ok(())
  }
}

pub struct Cache {
  cache: [RefCell<CacheEntry>; CACHE_SIZE],
  clock: Cell<u32>
}

impl Cache {
  pub fn new() -> Cache {
    Cache {
      cache: from_fn::<_, CACHE_SIZE, _>(|_| RefCell::new(CacheEntry::new())),
      clock: Cell::new(0u32)
    }
  }

  pub fn close_cache(&self, block: &Block) -> Result<(), FsErrors> {
    for entry in self.cache.iter(){
      let mut entry = entry.borrow_mut();
      if entry.occupied {
        entry.flush_cache_entry(block)?;
      }
    }
    Ok(())
  }

  pub fn cache_lookup(&self, sector: BlockSectorT) -> Option<RefMut<CacheEntry>> {
    for entry in self.cache.iter() {
      let entry = entry.borrow_mut();
      if entry.occupied && entry.disk_sector == Some(sector) {
        return Some(entry)
      }
    }
    None
  }

  fn evict_cache(&self, block: &Block) -> Result<RefMut<CacheEntry>, FsErrors> {
    loop {
      let clock = self.clock.get();
      let mut entry = self.cache[clock as usize].borrow_mut();

      if !entry.occupied {
        return Ok(entry)
      }

      if entry.access {
        entry.access = false;
      } else {
        break
      }

      self.clock.set((clock + 1) % CACHE_SIZE as u32);
    }

    let clock = self.clock.get();
    let mut entry = self.cache[clock as usize].borrow_mut();

    if entry.dirty {
      entry.flush_cache_entry(block)?;
    }
    entry.occupied = false;
    Ok(entry)
  }

  //Read a cache entry into memory
  pub fn read_cache_to_buffer(&self, block: &Block, sector: BlockSectorT, buffer: &mut [u8]) -> Result<(), FsErrors> {
    let mut entry;
    let slot = self.cache_lookup(sector);

    if let None = slot {
      entry = self.evict_cache(block)?;

      entry.occupied = true;
      entry.disk_sector = Some(sector);
      entry.dirty = false;
      block.read_block_to_buffer(sector, &mut entry.buffer)?;
    } else {
      entry = slot.unwrap()
    }

    entry.access = true;
    buffer.copy_from_slice(entry.buffer.as_slice());

    Ok(())
  }

  //Write a cache entry from memory
  pub fn write_cache_from_buffer(&self, block: &Block, sector: BlockSectorT, buffer: &[u8]) -> Result<(), FsErrors> {
    let mut entry;
    let slot = self.cache_lookup(sector);

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
    entry.dirty = true;
    entry.buffer.copy_from_slice(buffer);
    Ok(())
  }
}
