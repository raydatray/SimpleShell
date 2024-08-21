use std::{
  cell::RefCell, cmp::min, rc::Rc
};

use crate::{
  fs::{
    block::{
      Block,
      BlockSectorT,
      BLOCK_SECTOR_SIZE
    },
    cache::Cache,
    file_sys::FileSystem,
    freemap::Freemap,
    fserrors::inode_errors::InodeError
  }
};

use bytemuck::{
  bytes_of, from_bytes, Pod, Zeroable
};

const DIRECT_BLOCKS_CNT: u32 = 123u32;
const INDIRECT_BLOCKS_PER_SECTOR: u32 = 128u32;
const INODE_SIGNATURE: u32 = 0x494e4f44;

pub(crate) struct InodeList {
  inner: Vec<Rc<RefCell<MemoryInode>>>
}

impl InodeList {
  pub fn new() -> Self {
    Self {
      inner: Vec::new()
    }
  }

  ///Open an the INODE at a given SECTOR on BLOCK.
  ///
  ///Searches the given INODE LIST for an inode with provided SECTOR.
  ///If found, clones and returns an RC containing that inode.
  ///If not, opens that inode as an RC, places it into the INODE LIST and returns a cloned RC.
  pub fn open_inode(&mut self, block: &Block, cache: &Cache, sector: BlockSectorT) -> Result<Rc<RefCell<MemoryInode>>, InodeError> {
    match self.inner.iter().find(|inode| inode.borrow().sector == sector) {
      Some(inode) => {
        return Ok(inode.clone())
      },
      None => {
        let inode = MemoryInode::new(block, cache, sector)?;
        let celled_inode = Rc::new(RefCell::new(inode));
        let return_inode = celled_inode.clone();

        self.inner.push(celled_inode);
        return Ok(return_inode)
      }
    }
  }

  ///Closes the provided INODE.
  ///
  ///The caller needs to TAKE the RC containing the inode it wishes to close.
  ///The provided inode will be dropped. If the R-COUNT of that RC is 1 after that operation, we drop it from the INODE LIST
  pub fn close_inode(&mut self, state: &FileSystem, inode: Rc<RefCell<MemoryInode>>) -> Result<(), InodeError> {
    todo!()
  }

}

///Returns the number of sectors required to contain BYTES
#[inline(always)]
fn bytes_to_sectors(bytes: u32) -> u32 {
  bytes.div_ceil(BLOCK_SECTOR_SIZE)
}

pub(crate) struct MemoryInode {
  data: DiskInode,
  deny_write_cnt: u32,
  open_cnt: u32,
  removed: bool,
  sector: BlockSectorT
}

impl MemoryInode {
  ///Builds a new IN MEMORY INODE for the inode at SECTOR
  fn new(block: &Block, cache: &Cache, sector: BlockSectorT) -> Result<Self, InodeError> {
    let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
    cache.read_to_buffer(block, sector, &mut buffer)?;

    let disk_inode = from_bytes::<DiskInode>(&buffer).to_owned();

    Ok(
      Self {
        data: disk_inode,
        deny_write_cnt: 0u32,
        open_cnt: 1u32,
        removed: false,
        sector
      }
    )
  }

  pub fn is_dir(&self) -> bool {
    return match self.data.is_dir {
      1 => { true },
      _ => { false }
    }
  }

  pub fn removed(&self) -> bool {
    self.removed
  }

  pub fn inode_number(&self) -> BlockSectorT {
    self.sector
  }

  pub fn len(&self) -> u32 {
    self.data.len
  }

  pub fn allow_write(&mut self) {
    assert!(self.deny_write_cnt > 0);
    assert!(self.deny_write_cnt <= self.open_cnt);
    self.deny_write_cnt -= 1;
  }

  pub fn deny_write(&mut self) {
    self.deny_write_cnt += 1;
    assert!(self.deny_write_cnt <= self.open_cnt);
  }

  ///Returns a vector of BlockSectorT's that are allocated to SELF
  ///
  ///Return values are in-order of visitation
  pub fn data_sectors(&self, block: &Block, cache: &Cache) -> Result<Vec<BlockSectorT>, InodeError> {
    let data_len = self.data.len;

    let mut num_sectors = bytes_to_sectors(data_len);
    let mut sectors = Vec::<BlockSectorT>::with_capacity(num_sectors as usize);

    let mut curr_idx = 0usize;
    let mut limit = min(num_sectors, DIRECT_BLOCKS_CNT);

    //Direct blocks
    (0..limit).for_each(|i| {
      sectors[curr_idx] = self.data.direct_blocks[i as usize];
      curr_idx += 1;
    });

    if {num_sectors -= limit; num_sectors} == 0 {
      return Ok(sectors)
    }

    //Indirect block
    let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
    limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR);

    cache.read_to_buffer(block, self.data.indirect_block, &mut buffer)?;
    let indirect_block = from_bytes::<IndirectBlockSector>(&buffer);

    (0..limit).for_each(|i| {
      sectors[curr_idx] = indirect_block.inner[i as usize];
      curr_idx += 1;
    });

    if {num_sectors -= limit; num_sectors} == 0 {
      return Ok(sectors)
    }

    //Doubly indirect block
    limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR * INDIRECT_BLOCKS_PER_SECTOR);

    cache.read_to_buffer(block, self.data.doubly_indirect_block, &mut buffer)?;
    let doubly_indirect_block = from_bytes::<IndirectBlockSector>(&buffer).to_owned();
    let doubly_indirect_limit = limit.div_ceil(INDIRECT_BLOCKS_PER_SECTOR);

    let mut num_indirect_sectors = limit;

    (0..doubly_indirect_limit).try_for_each::<_, Result<(), InodeError>>(|i| {
      let subsize = min(num_indirect_sectors, INDIRECT_BLOCKS_PER_SECTOR);

      cache.read_to_buffer(block, doubly_indirect_block.inner[i as usize], &mut buffer)?;
      let indirect_block = from_bytes::<IndirectBlockSector>(&buffer);

      (0..subsize).for_each(|j| {
        sectors[curr_idx] = indirect_block.inner[j as usize];
        curr_idx += 1;
      });
      num_indirect_sectors -= subsize;
      Ok(())
    })?;

    if {num_sectors -= limit; num_sectors} == 0 {
      return Ok(sectors)
    }
    panic!("Number of sectors were not 0 at end of traversal");
  }

  fn deallocate(&self, state: &mut FileSystem) -> Result<(), InodeError> {
    let data_len = self.data.len;

    let mut num_sectors = bytes_to_sectors(data_len);
    let mut limit = min(num_sectors, DIRECT_BLOCKS_CNT);

    for i in 0..limit {
      Freemap::release(state, self.data.direct_blocks[i as usize], 1)?;
    }

    if {num_sectors -= limit; num_sectors} == 0 {
      return Ok(())
    }

    limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR);
    Self::deallocate_indirect(state, self.data.indirect_block, limit, 1)?;

    if {num_sectors -= limit; num_sectors} == 0 {
      return Ok(())
    }

    limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR * INDIRECT_BLOCKS_PER_SECTOR);
    Self::deallocate_indirect(state, self.data.doubly_indirect_block, limit, 2)?;

    assert_eq!(num_sectors, 0, "Number of sectors was not 0 at end of traversal");
    Ok(())
  }

  ///Deallocate INDIRECT BLOCKS (single or doubly),
  ///where the parent block is at SECTOR with NUM_SECTORS to be deallocated, and LVL degrees of indirection
  fn deallocate_indirect(state: &mut FileSystem, sector: BlockSectorT, mut num_sectors: u32, lvl: u32) -> Result<(), InodeError> {
    assert!(lvl <= 2, "Only double indirection is supported");

    //Base case
    if lvl == 0 {
      Freemap::release(state, sector, 1)?;
      return Ok(());
    }

    let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];

    state.cache.read_to_buffer(&state.block, sector, &mut buffer)?;
    let indirect_block = from_bytes::<IndirectBlockSector>(&buffer);

    //At single indirection (lvl 1), we only want to deallocate 1 sector
    //At double indirection (lvl 2), we want to deallocate either an entire section, or the remaing amount
    let unit = if lvl == 1 { 1 } else { INDIRECT_BLOCKS_PER_SECTOR };

    for sector in indirect_block.inner {
      let subsize = min(num_sectors, unit);
      Self::deallocate_indirect(state, sector, subsize, lvl - 1)?;
      num_sectors -= subsize;
    }

    assert_eq!(num_sectors, 0, "Number of sectors was not 0 at end of traversal");
    Freemap::release(state, sector, 1)?;
    Ok(())
  }

  ///Reads LENGTH bytes into BUFFER, starting at OFFSET. Returns the number of bytes read
  pub fn read_at(&self, block: &Block, cache: &Cache, buffer: &mut [u8], mut len: u32, mut ofst: u32) -> Result<u32, InodeError> {
    let mut bytes_read = 0usize;
    let mut bounce: Option<[u8; BLOCK_SECTOR_SIZE as usize]> = None;

    while len > 0 {
      let sector_idx = self.byte_to_sector(block, cache, ofst)?;
      let sector_ofst = (ofst % BLOCK_SECTOR_SIZE) as usize;

      let rmn_inode = self.len() - ofst;
      let rmn_sector = BLOCK_SECTOR_SIZE as usize - sector_ofst;
      let rmn_min = min(rmn_inode, rmn_sector as u32);

      let chunk_size = min(len, rmn_min) as usize;

      if chunk_size == 0 { break }

      //If we can read an entire SECTOR
      if sector_ofst == 0 && chunk_size == BLOCK_SECTOR_SIZE as usize {
        cache.read_to_buffer(block, sector_idx, &mut buffer[bytes_read..(bytes_read + BLOCK_SECTOR_SIZE as usize)])?;
      } else {
        if bounce.is_none() {
          bounce = Some([0u8; BLOCK_SECTOR_SIZE as usize]);
        }
        let bounce = bounce.as_mut().unwrap();
        cache.read_to_buffer(block, sector_idx,bounce)?;

        let buffer_slice = &mut buffer[bytes_read..(bytes_read + chunk_size)];
        let bounce_slice = &bounce[sector_ofst..(sector_ofst + chunk_size)];

        buffer_slice.copy_from_slice(bounce_slice);
      }

      len -= chunk_size as u32;
      ofst += chunk_size as u32;
      bytes_read += chunk_size;
    }

    Ok(bytes_read as u32)
  }

  pub fn write_at(&mut self, state: &mut FileSystem, buffer: &[u8], mut len: u32, mut ofst: u32) -> Result<u32, InodeError> {
    let mut bytes_wrote = 0usize;
    let mut bounce: Option<[u8; BLOCK_SECTOR_SIZE as usize]> = None;

    if self.deny_write_cnt > 0 { return Err(InodeError::WriteDenied()) }

    //If we need to extend the file
    if let Err(InodeError::OffsetOutOfBounds(_, _)) = self.byte_to_sector(&state.block, &state.cache, ofst + len - 1) {

    }

    Ok(bytes_wrote as u32)
  }

  ///Finds the SECTOR in which POS is located in on the given INODE
  fn byte_to_sector(&self, block: &Block, cache: &Cache, pos: u32) -> Result<BlockSectorT, InodeError> {
    if pos >= self.data.len {
      return Err(InodeError::OffsetOutOfBounds(pos, self.data.len))
    }

    let idx = pos / BLOCK_SECTOR_SIZE;
    self.data.idx_to_sector(block, cache, idx)
  }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C, packed)]
struct DiskInode {
  direct_blocks: [BlockSectorT; DIRECT_BLOCKS_CNT as usize],
  indirect_block: BlockSectorT,
  doubly_indirect_block: BlockSectorT,

  is_dir: u8, //We need this to satisfy bytemuck
  len: u32,
  sign: u32,
  _padding: [u8; 3] //We need this to satisfy bytemuck
}

impl DiskInode {
  ///Finds the SECTOR that IDX belongs to
  fn idx_to_sector(&self, block: &Block, cache: &Cache, idx: u32) -> Result<BlockSectorT, InodeError> {
    let mut idx_limit = DIRECT_BLOCKS_CNT;
    let mut idx_base = idx_limit;

    //Direct blocks (123)
    if idx < idx_limit {
      return Ok(self.direct_blocks[idx as usize])
    }

    //Indirect block
    idx_limit += INDIRECT_BLOCKS_PER_SECTOR;

    if idx < idx_limit {
      let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
      cache.read_to_buffer(block, self.indirect_block, &mut buffer)?;

      let indirect_block = from_bytes::<IndirectBlockSector>(&buffer);

      return Ok(indirect_block.inner[(idx - idx_base) as usize])
    }

    //Doubly indirect block
    idx_base = idx_limit;
    idx_limit += INDIRECT_BLOCKS_PER_SECTOR * INDIRECT_BLOCKS_PER_SECTOR;

    if idx < idx_limit {
      let idx_first = (idx - idx_base) / INDIRECT_BLOCKS_PER_SECTOR;
      let idx_scnd = (idx - idx_base) % INDIRECT_BLOCKS_PER_SECTOR;

      let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
      cache.read_to_buffer(block, self.doubly_indirect_block, &mut buffer)?;
      let indirect_block = from_bytes::<IndirectBlockSector>(&buffer);

      cache.read_to_buffer(block, indirect_block.inner[idx_first as usize], &mut buffer)?;
      let doubly_indirect_block = from_bytes::<IndirectBlockSector>(&buffer);

      return Ok(doubly_indirect_block.inner[idx_scnd as usize])
    }

    Err(InodeError::IndexOutOfBounds(idx))
  }

  fn allocate(&mut self, state: &mut FileSystem) -> Result<(), InodeError> {
    self.reserve(state, self.len)
  }

  fn reserve(&mut self, state: &mut FileSystem, len: u32) -> Result<(), InodeError> {
    const EMPTY_BUFFER: [u8; BLOCK_SECTOR_SIZE as usize] = [0u8; BLOCK_SECTOR_SIZE as usize];

    let mut num_sectors = bytes_to_sectors(len);
    let mut limit = min(num_sectors, DIRECT_BLOCKS_CNT);
    let mut idx;

    //Direct blocks
    for i in 0..limit {
      if self.direct_blocks[i as usize] == 0 {
        idx = Freemap::allocate(state, 1)?;
        self.direct_blocks[i as usize] = idx;
        state.cache.write_from_buffer(&state.block, idx, &EMPTY_BUFFER)?;
      }
    }

    if {num_sectors -= limit; num_sectors} == 0 {
      return Ok(())
    }

    //Indirect blocks
    limit = min(num_sectors,INDIRECT_BLOCKS_PER_SECTOR);
    idx = Self::reserve_indirect(state, limit, 1)?;
    self.indirect_block = idx;

    if {num_sectors -= limit; num_sectors} == 0 {
      return Ok(())
    }

    //Doubly indirect block
    limit = min(num_sectors, INDIRECT_BLOCKS_PER_SECTOR * INDIRECT_BLOCKS_PER_SECTOR);
    idx = Self::reserve_indirect(state, limit, 2)?;
    self.doubly_indirect_block = idx;

    if {num_sectors -= limit; num_sectors} == 0 {
      return Ok(())
    }

    panic!("Number of sectors were not 0 at end of traversal");
  }

  fn reserve_indirect(state: &mut FileSystem, mut num_sectors: u32, lvl: u32) -> Result<BlockSectorT, InodeError> {
    const EMPTY_BUFFER: [u8; BLOCK_SECTOR_SIZE as usize] = [0u8; BLOCK_SECTOR_SIZE as usize];
    assert!(lvl <= 2, "Only double indirection is supported");

    let idx;

    if lvl == 0 {
      idx = Freemap::allocate(state, 1)?;
      state.cache.write_from_buffer(&state.block, idx, &EMPTY_BUFFER)?;
      return Ok(idx)
    }

    idx = Freemap::allocate(state, 1)?;
    state.cache.write_from_buffer(&state.block, idx, &EMPTY_BUFFER)?;

    let mut indirect_block = IndirectBlockSector::new();
    let unit = if lvl == 1 { 1 } else { INDIRECT_BLOCKS_PER_SECTOR };
    let limit = num_sectors.div_ceil(unit);

    for i in 0..limit {
      let subsize = min(num_sectors, unit);
      let indirect_idx = Self::reserve_indirect(state, subsize, lvl - 1)?;

      indirect_block.inner[i as usize] = indirect_idx;
      num_sectors -= subsize;
    }

    assert_eq!(num_sectors, 0, "Number of sectors was not 0 at end of traversal");
    state.cache.write_from_buffer(&state.block, idx, bytes_of(&indirect_block))?;
    Ok(idx)
  }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
struct IndirectBlockSector {
  inner: [BlockSectorT; INDIRECT_BLOCKS_PER_SECTOR as usize]
}

impl IndirectBlockSector {
  ///Create an enpty IndirectBlockSector for creation and writing onto disk
  fn new() -> Self {
    Self {
      inner: [0u32; INDIRECT_BLOCKS_PER_SECTOR as usize]
    }
  }
}
