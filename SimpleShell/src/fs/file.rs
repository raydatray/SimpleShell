use std::{
  cell::RefCell,
  rc::Rc
};

use crate::fs::{
  block::Block,
  cache::Cache,
  fserrors::file_errors::FileError,
  inode::MemoryInode,
};

pub(crate) struct FileTable {
  inner: Vec<FileTableEntry>
}

impl FileTable {
  pub(crate) fn new() -> Self {
    Self {
      inner: Vec::new()
    }
  }
}

struct FileTableEntry {
  inner: Rc<RefCell<File>>,
  name: String
}

pub(crate) struct File {
  inode: Rc<RefCell<MemoryInode>>,
  pos: u32,
  deny_write: bool
}

impl File {
  pub fn open(inode: Rc<RefCell<MemoryInode>>) -> Self {
    Self {
      inode,
      pos: 0u32,
      deny_write: false
    }
  }

  pub fn close() { todo!() }


  pub fn read(&mut self, block: &Block, cache: &Cache, buffer: &mut [u8], size: u32) -> Result<u32, FileError> {
    todo!()
  }
}
