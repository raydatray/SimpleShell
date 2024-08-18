use std::{
  rc::Rc,
  cell::RefCell
};

use crate::fs::block::Block;
use crate::fs::cache::Cache;
use crate::fs::free_map::Freemap;
use crate::fs::fs_errors::FsErrors;
use crate::fs::file_sys::State;
use crate::fs::inode::{
  InodeList,
  MemoryInode
};

pub struct FileTable {
  inner: Vec<FileTableEntry>
}

struct FileTableEntry {
  file_name: String,
  file: Rc<RefCell<File>>,
}

pub struct File {
  inode: Rc<RefCell<MemoryInode>>,
  position: u32,
  deny_write: bool
}

impl FileTable {
  pub fn new() -> Self {
    Self {
      inner: Vec::new()
    }
  }

  pub fn add_by_name(&mut self, file: Rc<RefCell<File>>, file_name: String) {
    if let Some(_) = self.get_by_name(&file_name) {
      return
    } else {
      let file_table_entry = FileTableEntry::new(file_name, file);
      self.inner.push(file_table_entry);
    }
  }

  pub fn get_by_name(&self, file_name: &String) -> Option<Rc<RefCell<File>>> {
    self.inner.iter().find_map(|entry| {
      if entry.file_name == *file_name {
        Some(entry.file.clone())
      } else {
        None
      }
    })
  }

  pub fn close(&self, inode_list: &mut InodeList) -> Result<(), FsErrors> {
    self.inner.iter().for_each(|entry| {
      entry.file.borrow_mut().close(inode_list)?;
    });

    Ok(())
  }
}

impl FileTableEntry {
  pub fn new(file_name: String, file: Rc<RefCell<File>>) -> Self {
    Self {
      file_name,
      file
    }
  }
}

impl File {
  pub fn open(inode: Rc<RefCell<MemoryInode>>) -> Self {
    Self {
      inode,
      position: 0u32,
      deny_write: false
    }
  }

  pub fn close(&mut self, inode_list: &mut InodeList) -> Result<(), FsErrors>{
    self.allow_write();

    if let None = self.inode {
      return Err(todo!())
    }

    inode_list.close_inode(self.inode.take())?;
    //We're supposed to deallocate the file here too....
  }

  fn get_inode(&self) -> Rc<RefCell<MemoryInode>> {
    return match &self.inode {
      Some(inode) => inode.clone(),
      None => todo!("Some error")
    }
  }

  pub fn read(&mut self, block: &Block, cache: &Cache, buffer: &mut [u8], size: u32) -> Result<u32, FsErrors> {
    let bytes_read = self.inode.borrow_mut().read_at(block, cache, buffer, size, self.position)?;
    self.position += bytes_read;
    Ok(bytes_read)
  }

  pub fn read_at(&self, block: &Block, cache: &Cache, buffer: &mut [u8], size: u32, offset: u32) -> Result<u32, FsErrors> {
    self.inode.borrow_mut().read_at(block, cache, buffer, size, offset)
  }

  pub fn write(&mut self, state: &mut State, buffer: &[u8], size: u32) -> Result<u32, FsErrors> {
    let bytes_written = self.inode.clone().borrow_mut().write_at(state, buffer, size, self.position)?;
    self.position += bytes_written;
    Ok(bytes_written)
  }

  pub fn write_at(&self, state: &mut State, buffer: &[u8], size: u32, offset: u32) -> Result<u32, FsErrors> {
    self.inode.clone().borrow_mut().write_at(state, buffer, size, offset)
  }

  fn deny_write(&mut self) {
    if !self.deny_write {
      self.deny_write = true;
      self.inode.borrow_mut().deny_write();
    }
  }

  fn allow_write(&mut self) {
    if self.deny_write {
      self.deny_write = false;
      self.inode.borrow_mut().allow_write();
    }
  }

  fn file_length(&self) -> u32 {
    self.inode.borrow().get_length()
  }

  ///From the start of the file
  fn file_seek(&mut self, new_pos: u32) {
    self.position = new_pos
  }

  fn file_tell(&self) -> u32 {
    self.position
  }
}
