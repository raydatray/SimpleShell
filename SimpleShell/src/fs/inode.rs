use std::{
  cell::RefCell,
  rc::Rc
};

use crate::{
  fs::{
    block::{
      Block,
      BlockSectorT,
      BLOCK_SECTOR_SIZE
    },
    cache::Cache,
    fserrors::FSErrors
  }
};

use bytemuck::{
  from_bytes, Pod, Zeroable
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

  pub fn open_inode(&mut self, block: &Block, cache: &Cache, sector: BlockSectorT) -> Result<Rc<RefCell<MemoryInode>>, FSErrors> {
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

}

pub(crate) struct MemoryInode {
  data: DiskInode,
  deny_write_cnt: u32,
  removed: bool,
  sector: BlockSectorT
}

impl MemoryInode {
  fn new(block: &Block, cache: &Cache, sector: BlockSectorT) -> Result<Self, FSErrors> {
    let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];
    cache.read_to_buffer(block, sector, &mut buffer)?;

    let disk_inode = from_bytes::<DiskInode>(&buffer).to_owned();

    Ok(
      Self {
        data: disk_inode,
        deny_write_cnt: 0u32,
        removed: false,
        sector
      }
    )
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

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
struct IndirectBlockSector {
  inner: [BlockSectorT; INDIRECT_BLOCKS_PER_SECTOR as usize]
}
