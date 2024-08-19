use std::{
  cell::RefCell,
  rc::Rc
};

use crate::fs::{
  inode::MemoryInode
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
