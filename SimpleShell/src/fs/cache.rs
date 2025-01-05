use std::{
  array::from_fn,
  cell::{
    Cell,
    RefCell,
    RefMut
  }
};

use crate::fs::{
  block::{
    Block,
    BlockSectorT,
    BLOCK_SECTOR_SIZE
  },
  fserrors::cache_errors::CacheError
};

const CACHE_SIZE: usize = 64usize;


///A simple write-back cache with CACHE_SIZE (64) entries
pub(crate) struct Cache {
  inner: [RefCell<CacheEntry>; CACHE_SIZE],
  clock: Cell<usize>
}

impl Cache {
  pub fn new() -> Self {
    Self {
      inner: from_fn::<_, CACHE_SIZE, _>(|_| RefCell::new(CacheEntry::new())),
      clock:Cell::new(0usize)
    }
  }

  ///Closes the cache, flushing all filled entries back to disk
  ///
  ///You MUST call this before dropping CACHE
  pub fn close(&self, block: &Block) -> Result<(), CacheError> {
    self.inner.iter()
      .filter(|entry| {
        entry.borrow().occupied
      })
      .try_for_each(|entry| {
        entry.borrow_mut().flush(block)
      })
  }

  ///Reads a cache entry with tag SECTOR on BLOCK to BUFFER
  ///
  ///Caller must guarantee BUFFER to be of length BLOCK_SECTOR_SIZE
  pub fn read_to_buffer(&self, block: &Block, sector: BlockSectorT, buffer: &mut [u8]) -> Result<(), CacheError> {
    assert_eq!(buffer.len(), BLOCK_SECTOR_SIZE as usize);

    let mut entry;
    let slot = self.lookup(sector);

    if let None = slot {
      entry = self.evict(block)?;

      entry.disk_sector = Some(sector);
      entry.dirty = false;
      entry.occupied = true;
      block.read_to_buffer(sector, &mut entry.inner)?;
    } else {
      entry = slot.unwrap();
    }

    entry.access = true;
    buffer.copy_from_slice(entry.inner.as_slice());

    Ok(())
  }

  ///Writes a cache entry with tag SECTOR on BLOCK from BUFFER
  ///
  ///Caller must gurantee BUFFER to be of length BLOCK_SECTOR_SIZE
  pub fn write_from_buffer(&self, block: &Block, sector: BlockSectorT, buffer: &[u8]) -> Result<(), CacheError> {
    assert_eq!(buffer.len(), BLOCK_SECTOR_SIZE as usize);

    let mut entry;
    let slot = self.lookup(sector);

    if let None = slot {
      entry = self.evict(block)?;

      entry.disk_sector = Some(sector);
      entry.dirty = false;
      entry.occupied = true;
      block.read_to_buffer(sector, &mut entry.inner)?;
    } else {
      entry = slot.unwrap();
    }

    entry.access = true;
    entry.dirty = true;
    entry.inner.copy_from_slice(buffer);

    Ok(())
  }

  ///Evict a cache entry back to BLOCK, and returns a mutable reference to that entry for immediate use
  fn evict(&self, block: &Block) -> Result<RefMut<CacheEntry>, CacheError> {
    let mut entry;

    loop {
      let clock = self.clock.get();
      entry = self.inner[clock].borrow_mut();

      if !entry.occupied {
        return Ok(entry);
      }

      if entry.access {
        entry.access = false;
      } else {
        break;
      }

      self.clock.set((clock + 1) % CACHE_SIZE);
    }

    let clock = self.clock.get();
    entry = self.inner[clock].borrow_mut();

    if entry.occupied {
      entry.flush(block)?;
    }
    entry.occupied = false;
    Ok(entry)
  }

  ///Searches for a cache entry with tag SECTOR
  fn lookup(&self, sector: BlockSectorT) -> Option<RefMut<CacheEntry>> {
    self.inner.iter().find_map(|entry| {
      let entry = entry.borrow_mut();
      if entry.disk_sector == Some(sector) && entry.occupied {
        Some(entry)
      } else {
        None
      }
    })
  }
}

///A simple cache entry with inner byte buffer of size BLOCK_SECTOR_SIZE and tags
struct CacheEntry {
  inner: [u8; BLOCK_SECTOR_SIZE as usize],
  disk_sector: Option<BlockSectorT>,
  access: bool,
  dirty: bool,
  occupied: bool
}

impl CacheEntry {
  fn new() -> Self {
    Self {
      inner: [0u8; BLOCK_SECTOR_SIZE as usize],
      disk_sector: None,
      access: false,
      dirty: false,
      occupied: false
    }
  }

  ///Writes entry back to tagged sector on BLOCK
  fn flush(&mut self, block: &Block) -> Result<(), CacheError> {
    if !self.occupied {
      return Err(CacheError::FlushUnoccupiedEntry())
    }

    if self.dirty {
      match self.disk_sector {
        Some(disk_sector) => {
          block.write_from_buffer(disk_sector, &self.inner)?;
          self.dirty = false;
        },
        None => {
          return Err(CacheError::FlushNullDiskSector())
        }
      }
    }
    Ok(())
  }
}
