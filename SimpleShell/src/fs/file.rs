use std::{
  cell::RefCell,
  rc::Rc
};

use crate::fs::{
  block::Block,
  cache::Cache,
  file_sys::FileSystem,
  fserrors::file_errors::FileError,
  inode::{InodeList, MemoryInode},
};

pub (crate) struct FileTable {
  inner: Vec<FileTableEntry>
}

impl FileTable {
  pub (crate) fn new() -> Self {
    Self {
      inner: Vec::new()
    }
  }

  ///Closes the FILE TABLE, freeing all FILES and associated INODEs
  ///
  ///SAFETY: This fn replaces the inner value of FILETABLE using mem::take, you should only call this
  ///when you will not be using the existing entries in FILETABLE again
  pub (crate) fn close(state: &mut FileSystem) -> Result<(), FileError> {
    let entries = std::mem::take(&mut state.file_table.inner);
    for entry in entries {
      entry.inner.borrow_mut().close(state)?;
    }
    Ok(())
  }

  pub (crate) fn add_by_name(&mut self, file: Rc<RefCell<File>>, file_name: &str) {
    if let None = self.get_by_name(file_name) {
      let file_table_entry = FileTableEntry::new(file, file_name);
      self.inner.push(file_table_entry);
    }
  }

  pub (crate) fn get_by_name(&self, file_name: &str) -> Option<Rc<RefCell<File>>> {
    self.inner.iter().find_map(|entry| {
      if entry.name == file_name {
        Some(entry.inner.clone())
      } else {
        None
      }
    })
  }

  pub (crate) fn remove_by_name(state: &mut FileSystem, file_name: &str) -> Result<(), FileError> {
    let idx = state.file_table.inner.iter().position(|entry| entry.name == file_name);

    match idx {
      Some(idx) => {
        let entry = state.file_table.inner.remove(idx);
        entry.inner.borrow_mut().close(state)?;
        return Ok(())
      },
      None => return Err(FileError::FileNotFound(file_name.to_string()))
    }
  }
}

struct FileTableEntry {
  inner: Rc<RefCell<File>>,
  name: String
}

impl FileTableEntry {
  fn new(file: Rc<RefCell<File>>, file_name: &str) -> Self {
    Self {
      inner: file,
      name: file_name.to_string()
      }
  }
}

pub(crate) struct File {
  inode: Rc<RefCell<MemoryInode>>,
  deny_write: bool,
  pos: u32
}

impl File {
  pub fn open(inode: Rc<RefCell<MemoryInode>>) -> Self {
    Self {
      inode,
      deny_write: false,
      pos: 0u32,
    }
  }

  pub fn close(&mut self, state: &mut FileSystem) -> Result<(), FileError> {
    self.allow_write();
    let inode_num = self.inode.borrow().inode_num();
    InodeList::close_inode(state, inode_num)?;
    Ok(())
  }

  fn allow_write(&mut self) {
    if self.deny_write {
      self.deny_write = false;
      self.inode.borrow_mut().allow_write()
    }
  }

  fn deny_write(&mut self) {
    if !self.deny_write {
      self.deny_write = true;
      self.inode.borrow_mut().deny_write()
    }
  }

  fn len(&self) -> u32 {
    self.inode.borrow().len()
  }

  fn seek(&mut self, ofst: u32) {
    self.pos = ofst
  }

  fn tell(&self) -> u32 {
    self.pos
  }

  pub fn read(&mut self, block: &Block, cache: &Cache, buffer: &mut [u8], len: u32) -> Result<u32, FileError> {
    let bytes_read = self.inode.borrow_mut().read_at(block, cache, buffer, len, self.pos)?;
    self.pos += bytes_read;
    Ok(bytes_read)
  }

  pub fn read_at(&mut self, block: &Block, cache: &Cache, buffer: &mut [u8], len: u32, ofst: u32) -> Result<u32, FileError> {
    let bytes_read = self.inode.borrow_mut().read_at(block, cache, buffer, len, ofst)?;
    Ok(bytes_read)
  }

  pub fn write(&mut self, state: &mut FileSystem, buffer: &[u8], len: u32) -> Result<u32, FileError> {
    let bytes_wrote = self.inode.borrow_mut().write_at(state, buffer, len, self.pos)?;
    self.pos += bytes_wrote;
    Ok(bytes_wrote)
  }

  pub fn write_at(&mut self, state: &mut FileSystem, buffer: &[u8], len: u32, ofst: u32) -> Result<u32, FileError> {
    let bytes_wrote = self.inode.borrow_mut().write_at(state, buffer, len, ofst)?;
    Ok(bytes_wrote)
  }
}
