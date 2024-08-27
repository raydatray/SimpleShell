use std::{
  cell::{
    Cell,
    RefCell
  },
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
  deny_write: Cell<bool>,
  pos: Cell<u32>
}

impl File {
  pub fn open(inode: Rc<RefCell<MemoryInode>>) -> Self {
    Self {
      inode,
      deny_write: Cell::new(false),
      pos: Cell::new(0u32)
    }
  }

  pub fn close(&self, state: &mut FileSystem) -> Result<(), FileError> {
    self.allow_write();
    let inode_num = self.inode.borrow().inode_num();
    InodeList::close_inode(state, inode_num)?;
    Ok(())
  }

  fn allow_write(&self) {
    let inner = self.deny_write.take();
    if inner {
      self.deny_write.set(false);
      self.inode.borrow_mut().allow_write()
    }
  }

  fn deny_write(&self) {
    let inner = self.deny_write.take();

    if !inner {
      self.deny_write.set(true);
      self.inode.borrow_mut().deny_write()
    }
  }

  pub fn len(&self) -> u32 {
    self.inode.borrow().len()
  }

  pub fn seek(&self, ofst: u32) {
    self.pos.set(ofst)
  }

  pub fn tell(&self) -> u32 {
    self.pos.get()
  }

  pub fn read(&self, block: &Block, cache: &Cache, buffer: &mut [u8], len: u32) -> Result<u32, FileError> {
    let pos = self.pos.get();

    let bytes_read = self.inode.borrow_mut().read_at(block, cache, buffer, len, pos)?;
    self.pos.set(pos + bytes_read);
    Ok(bytes_read)
  }

  pub fn read_at(&self, block: &Block, cache: &Cache, buffer: &mut [u8], len: u32, ofst: u32) -> Result<u32, FileError> {
    let bytes_read = self.inode.borrow_mut().read_at(block, cache, buffer, len, ofst)?;
    Ok(bytes_read)
  }

  pub fn write(&self, state: &mut FileSystem, buffer: &[u8], len: u32) -> Result<u32, FileError> {
    let pos = self.pos.get();

    let bytes_wrote = self.inode.borrow_mut().write_at(state, buffer, len, pos)?;
    self.pos.set(pos + bytes_wrote);
    Ok(bytes_wrote)
  }

  pub fn write_at(&self, state: &mut FileSystem, buffer: &[u8], len: u32, ofst: u32) -> Result<u32, FileError> {
    let bytes_wrote = self.inode.borrow_mut().write_at(state, buffer, len, ofst)?;
    Ok(bytes_wrote)
  }
}
