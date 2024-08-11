use std::{
  array::from_fn,
  cell::{
    Cell,
    RefCell,
    RefMut
  }
};

use crate::fs::block::{Block, BlockSectorT, BLOCK_SECTOR_SIZE};
use crate::fs::fs_errors::FsErrors;

const CACHE_SIZE: usize = 64usize;

///A CACHE that holds CACHE_SIZE amount of entries, with a built in clock for replacement
///
///A CACHE is to be used immutably, as it does not require a mutable references to perform any of its operations
///
///The cache interfaces can be considered STABLE
pub struct Cache {
  cache: [RefCell<CacheEntry>; CACHE_SIZE],
  clock: Cell<u32>
}

///A CACHE holds CACHE_ENTRIES, which contains a buffer of bytes BLOCK_SECTOR_SIZE long
///
///Auxiliary information is held in the entry to perform writeback and allocations
///
///
struct CacheEntry {
  buffer: [u8; BLOCK_SECTOR_SIZE as usize],
  disk_sector: Option<BlockSectorT>,
  occupied: bool,
  dirty: bool,
  access: bool
}

impl Cache {
  ///Initiate a new CACHE with a 0 clock
  pub fn new() -> Cache {
    Cache {
      cache: from_fn::<_, CACHE_SIZE, _>(|_| RefCell::new(CacheEntry::new())),
      clock: Cell::new(0u32)
    }
  }

  ///Close a CACHE, writing all ENTRIES back to the provided BLOCK device
  pub fn close(&self, block: &Block) -> Result<(), FsErrors> {
    self.cache.iter()
      .filter(|entry| {
        entry.borrow().occupied
      })
      .try_for_each(|entry| {
        entry.borrow_mut().flush_cache_entry(block)
      })?;
      Ok(())
  }

  ///Looks for a CACHE ENTRY @ SECTOR in the CACHE
  ///
  ///Returns Some(RefMut<CacheEntry>) if found, None if not
  pub fn lookup(&self, sector: BlockSectorT) -> Option<RefMut<CacheEntry>> {
    self.cache.iter().find_map(|entry| {
      let entry = entry.borrow_mut();
      if entry.occupied && entry.disk_sector == Some(sector) {
        Some(entry)
      } else {
        None
      }
    })
  }

  ///Evict a CACHE ENTRY from CACHE with the built in algorithm
  ///
  ///Returns a mutable reference to cleared CACHE ENTRY for immediate use
  fn evict(&self, block: &Block) -> Result<RefMut<CacheEntry>, FsErrors> {
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

  ///Read a CACHE ENTRY into a provided buffer
  ///
  ///Safety: provided buffer must be of size BLOCK_ENTRY_SIZE, panics if otherwise
  pub fn read_to_buffer(&self, block: &Block, sector: BlockSectorT, buffer: &mut [u8]) -> Result<(), FsErrors> {
    assert_eq!(buffer.len(), BLOCK_SECTOR_SIZE as usize);

    let mut entry;
    let slot = self.lookup(sector);

    if let None = slot {
      entry = self.evict(block)?;

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

  ///Write a CACHE ENTRY from a provided buffer
  ///
  ///Safety: provided buffer must be of size BLOCK_ENTRY_SIZE, panics if otherwise
  pub fn write_from_buffer(&self, block: &Block, sector: BlockSectorT, buffer: &[u8]) -> Result<(), FsErrors> {
    assert_eq!(buffer.len(), BLOCK_SECTOR_SIZE as usize);

    let mut entry;
    let slot = self.lookup(sector);

    if let None = slot {
      entry = self.evict(block)?;

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

impl CacheEntry {
  fn new() -> CacheEntry {
    CacheEntry {
      buffer: [0u8; BLOCK_SECTOR_SIZE as usize],
      disk_sector: None,
      occupied: false,
      dirty: false,
      access: false
    }
  }

  ///Flushes the given CACHE ENTRY back to BLOCK @ the internally held DISK_SECTOR
  ///
  ///Can error in the case of:
  ///Attempting to flush an unoccupied entry
  ///Attempting to flush an entry with a "NONE" disk_sector
  pub fn flush_cache_entry(&mut self, block: &Block) -> Result<(), FsErrors> {
    if !self.occupied {
      return Err(FsErrors::UnoccupiedCacheEntry())
    }

    match self.dirty {
      Some(disk_sector) => {
        block.write_buffer_to_block(self.disk_sector.unwrap(), &self.buffer)?;
        self.dirty = false;
        Ok(())
      },
      None => {
        return Err(FsErrors::CacheEntryUnspecifiedSector())
      }
    }
  }
}
