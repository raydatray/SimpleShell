use std::cell::{RefCell, RefMut};
use super::{free_map::Freemap, fs_errors::FsErrors, inode_dup::{InodeList, MemoryInode}};

pub struct FileTable {
  inner: Vec<FileTableEntry>
}

struct FileTableEntry {
  file_name: String,
  file: RefCell<File>,
  index: FileTableKey
}

pub struct File {
  inode: RefMut<MemoryInode>,
  position: u32,
  deny_write: bool
}

impl FileTable {
  pub fn new() -> Self {
    Self {
      inner: SlotMap::with_key()
    }
  }

  pub fn add_by_name(&mut self, file: File, file_name: String) {
    if let Some(_) = self.get_by_name(file_name) {
      return
    }
  }

  pub fn get_by_name(&self, file_name: String) -> Option<FileTableKey> {
    self.inner.iter().find_map(|(key, entry)| {
      if entry.file_name == file_name {
        Some(key)
      } else {
        None
      }
    })
  }
}

impl FileTableEntry {
}

impl File {
  pub fn open(inode: RefMut<MemoryInode>) -> Self {
    Self {
      inode,
      position: 0u32,
      deny_write: false
    }
  }

  pub fn close(&mut self, inode_list: &mut InodeList) -> Result<(), FsErrors>{
    self.file_allow_write();
    inode_list.close_inode(inode)
    //free(self)???
  }

  fn get_inode(&self) -> RefMut<MemoryInode> {
    self.inode
  }

  pub fn read(&mut self, buffer: &mut [u8], size: u32) -> Result<u32, FsErrors> {
    let bytes_read = self.inode.read_at(cache, block, buffer, size, self.position)?;
    self.position += bytes_read;
    Ok(bytes_read)
  }

  pub fn read_at(&self, buffer: &mut [u8], size: u32, offset: u32) -> Result<u32, FsErrors> {
    self.inode.read_at(cache, block, buffer, size, offset)
  }

  pub fn write(&mut self, freemap: &mut Freemap, buffer: &[u8], size: u32) -> Result<u32, FsErrors> {
    let bytes_written = self.inode.write_at(cache, block, freemap, buffer, size, self.position)?;
    self.position += bytes_written;
    Ok(bytes_written)
  }

  pub fn write_at(&self, freemap: &mut Freemap, buffer: &[u8], size: u32, offset: u32) -> Result<u32, FsErrors> {
    self.inode.write_at(cache, block, freemap, buffer, size, offset)
  }

  fn deny_write(&mut self) {
    if !self.deny_write {
      self.deny_write = true;
      self.inode.deny_write();
    }
  }

  fn allow_write(&mut self) {
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
