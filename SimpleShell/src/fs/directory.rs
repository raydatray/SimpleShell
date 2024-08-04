use std::{mem, cell::RefCell, path::Path};
use std::mem;
use std::rc::Rc;
use bytemuck::Pod;
use crate::fs::bitmap::byte_cnt;
use crate::fs::block::{Block, BlockSectorT};
use crate::fs::file_sys::{ROOT_DIR_SECTOR, State};
use crate::fs::fs_errors::FsErrors;
use crate::fs::inode_dup::{DiskInode, InodeList, MemoryInode};

const NAME_MAX: usize = 30;
pub(crate) const DIRECTORY_ENTRY_SIZE: u32 = {
  let directory_entry_size = mem::size_of::<DirectoryEntry>();
  todo!("Assert its the right size or smthn");
  directory_entry_size as u32
};

pub struct Directory {
  inode: Rc<RefCell<MemoryInode>>,
  position: u32,
  open_cnt: u32 //Redundant due to RC counting?
}

#[derive(Pod)]
#[repr(C)]
struct DirectoryEntry {
  inode_sector: BlockSectorT,
  name: [u8; NAME_MAX + 1],
  in_use: bool
}

pub fn split_path_filename(path: &str) -> (&str, &str)  {
  let path = Path::new(path);

  let directory = path.parent().and_then(|p| p.to_str()).unwrap_or("");

  let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

  (directory, filename)
}

impl Directory {
  pub fn new(sector: BlockSectorT, entry_cnt: u32) -> Result<(), FsErrors> {
    DiskInode::new(block, cache, freemap, sector, directory_entry_size * entry_cnt, true)?;

  }

  pub fn open_root(state: &mut State) -> Result<&mut Directory, FsErrors> {
    match state.cwd {
      Some(directory) => {
        return Ok(&mut directory)
      },
      None => {
        let cwd = Self::open(state.inode_list.open_inode(ROOT_DIR_SECTOR)?);
        state.cwd = Some(cwd);
        return Ok(&mut cwd);
      }
    }
  }

  pub fn open_path(state: State, path: &str) -> Result<Rc<RefCell<Directory>>, FsErrors>{
    let mut curr = if path.starts_with("/") {
      Directory::open_root(state)?
    } else {
      match state.get_cwd() {
        Some(cwd) => {
          cwd
        },
        None => {
          Directory::open_root(state)?
        }
      }
    };


    for token in path.split("/").filter(|&x| !x.is_empty()) {

    }
    Ok(())
  }


  pub fn open(inode: Rc<RefCell<MemoryInode>>) -> Self {
    Self {
      inode,
      position: DIRECTORY_ENTRY_SIZE,
      open_cnt: 1u32
    }
  }

  pub fn close(&self) -> Result<(), FsErrors> {
    todo!();
  }

  pub fn lookup() -> Result<Rc<RefCell<MemoryInode>>, FsErrors> {todo!()}

  pub fn get_inode(&self) -> Rc<RefCell<MemoryInode>> {
    self.inode.clone()
  }

  pub fn is_empty(&self, state: State) -> bool {
    let mut buffer = [0u8; DIRECTORY_ENTRY_SIZE as usize];
    let mut offset = DIRECTORY_ENTRY_SIZE;

    while self.inode.borrow_mut().read_at(state, &mut buffer, DIRECTORY_ENTRY_SIZE, offset)? == DIRECTORY_ENTRY_SIZE {
      let entry: &DirectoryEntry = bytemuck::from_bytes(&buffer);
      if entry.in_use {
        return false
      }

      offset += DIRECTORY_ENTRY_SIZE;
    }
    true
  }

  pub fn search_for_file(&self, state: State, name: &str) -> Result<Rc<RefCell<MemoryInode>>, FsErrors> {
    let mut buffer = [0u8; DIRECTORY_ENTRY_SIZE as usize];

    match name {
      "." => {
        return Ok(self.inode.clone())
      },
      ".."=> {
        self.inode.borrow_mut().read_at(state, &mut buffer, DIRECTORY_ENTRY_SIZE, 0)?;
        let entry: &DirectoryEntry = bytemuck::from_bytes(&buffer);
        return state.inode_list.open_inode(entry.inode_sector)
      },
      _ => {
        match
      }
    }
  }

  pub fn add(&mut self, state: &mut State, name: &str, inode_sector: BlockSectorT, is_dir: bool) -> Result<(), FsErrors> {

  }
  pub fn remove(&mut self, state: State, pattern: String) -> Result<(), FsErrors> {

  }

  fn read_entry_names(&mut self, state: State) -> Result<String, FsErrors> {
    let mut buffer = [0u8; DIRECTORY_ENTRY_SIZE as usize];

    while self.inode.borrow_mut().read_at(state, &mut buffer, DIRECTORY_ENTRY_SIZE, self.position)? == DIRECTORY_ENTRY_SIZE {
      self.position += DIRECTORY_ENTRY_SIZE;
      let directory_entry: &DirectoryEntry = bytemuck::from_bytes(&buffer);

      if directory_entry.in_use {
        return Ok(directory_entry.name_to_string())
      }
    }
    Err(todo!("Some error..."))
  }

  fn read_entry_inodes(&mut self, state: State) -> Result<(String, BlockSectorT), FsErrors> {
    let mut buffer = [0u8; DIRECTORY_ENTRY_SIZE as usize];

    while self.inode.borrow_mut().read_at(state, &mut buffer, DIRECTORY_ENTRY_SIZE, self.position)? == DIRECTORY_ENTRY_SIZE {
      self.position += DIRECTORY_ENTRY_SIZE;
      let directory_entry: &DirectoryEntry = bytemuck::from_bytes(&buffer);

      if directory_entry.in_use {
        return Ok((directory_entry.name_to_string(), directory_entry.inode_sector))
      }
    }

    Err(todo!("Some error..."))
  }
}

impl DirectoryEntry {
  fn name_to_string(&self) -> String {
    String::from_utf8_lossy(&self.name).to_string()
  }
}
