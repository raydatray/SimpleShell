use std::collections::VecDeque;

use super::{fs_errors::FsErrors, inode::{InodeKey, MemoryInode}};

struct FileTable {
  inner: VecDeque<FileTableEntry>
}

struct FileTableEntry {

}

struct File {
  inode: InodeKey,
  offset: u32,
  deny_write: bool
}

impl FileTable {
  pub fn new() -> Self {
    Self {
      inner: VecDeque::new()
    }
  }
}

impl FileTableEntry {

}

impl File {



  fn file_close(&mut self) {
    self.file_allow_write();

  }

  fn file_get_inode(&self) -> &MemoryInode {
    self.inode
  }

  fn file_read(&mut self, buffer: &mut [u8], size: u32) -> Result<u32, FsErrors> {
    todo!();
  }

  fn file_read_at(&self, buffer: &mut [u8], size: u32, offset: u32) -> Result<u32, FsErrors> {
    todo!();
  }

  fn file_write(&mut self, buffer: &[u8], size: u32) -> Result<u32, FsErrors> {
    todo!()
  }

  fn file_write_at(&self, buffer: &[u8], size: u32, offset: u32) -> Result<u32, FsErrors> {
    todo!()
  }

  fn file_deny_write(&mut self) {
    if !self.deny_write {
      self.deny_write = true;
      self.inode.deny_write();
    }
  }

  fn file_allow_write(&mut self) {
    if self.deny_write {
      self.deny_write = false;
      self.inode.allow_write();
    }
  }

  fn file_length(&self) -> u32 {
    self.inode.get_inode_length()
  }

  ///From the start of the file
  fn file_seek(&mut self, new_pos: u32) {
    self.offset = new_pos
  }

  fn file_tell(&self) -> u32 {
    self.offset
  }
}
