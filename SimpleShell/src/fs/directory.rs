use std::{
  cell::{Ref, RefCell, RefMut}, mem::size_of, rc::Rc, str::SplitAsciiWhitespace
};

use crate::fs::{
  block::BlockSectorT,
  file_sys::FileSystem,
  fserrors::dir_errors::DirError,
  inode::{DiskInode, InodeList, MemoryInode}
};

use bytemuck::{from_bytes, bytes_of, Pod, Zeroable};

use super::{file_sys::ROOT_DIR_SECTOR};


const NAME_MAX: usize = 31;
const DIR_ENTRY_SIZE: u32 = size_of::<DiskDirectory>() as u32;

pub(crate) struct MemoryDirectory {
  inode: Rc<RefCell<MemoryInode>>,
  pos: u32,
  open_cnt: u32
}

pub fn split_path(path: &str) -> (&str, &str) {
  let path = path.trim_end_matches('/');

  match path.rfind('/') {
    Some(idx) => (&path[..=idx], &path[idx + 1..]),
    None => ("", path)
  }
}

impl MemoryDirectory {
  fn new(inode: Rc<RefCell<MemoryInode>>) -> Self {
    Self {
      inode,
      pos: DIR_ENTRY_SIZE,
      open_cnt: 1u32
    }
  }

  pub fn get_inode(&self) -> Rc<RefCell<MemoryInode>> {
    self.inode.clone()
  }

  pub fn new_on_disk(state: &mut FileSystem, sector: BlockSectorT, entry_cnt: u32) -> Result<(), DirError> {
    let _ = DiskInode::new(state, sector, DIR_ENTRY_SIZE * entry_cnt, true)?;

    let inode = state.inode_list.open_inode(&state.block, &state.cache, sector)?;
    let dir_entry = DiskDirectory::new(sector);

    let bytes_wrote = inode.borrow_mut().write_at(state, bytes_of(&dir_entry), DIR_ENTRY_SIZE, 0)?;

    if bytes_wrote != DIR_ENTRY_SIZE {
      return Err(DirError::CreationFailedBytesMissing())
    }
    Ok(())
  }

  pub fn open_root(state: &mut FileSystem) -> Result<Rc<RefCell<MemoryDirectory>>, DirError> {
    if state.cwd.is_none() {
      let root_inode = state.inode_list.open_inode(&state.block, &state.cache, ROOT_DIR_SECTOR)?;
      let root_dir = Self::new(root_inode);
      state.cwd = Some(Rc::new(RefCell::new(root_dir)));
    }

    match &state.cwd {
      Some(dir) => Ok(dir.clone()),
      None => panic!("Root directory still none after assignment")
    }
  }

  pub fn open_path(state: &mut FileSystem, path: &str) -> Result<Rc<RefCell<MemoryDirectory>>, DirError> {
    let mut curr_dir = if path.starts_with("/") {
      Self::open_root(state)?
    } else {
      match &state.cwd {
        Some(cwd) => {
          cwd.clone()
        },
        None => {
          Self::open_root(state)?
        }
      }
    };

    for token in path.split("/").filter(|&x| !x.is_empty()) {
      let next_dir;

      match curr_dir.borrow().lookup(state, token)? {
        Some((entry, _)) => {
          let next_inode = state.inode_list.open_inode(&state.block, &state.cache, entry.sector)?;
          next_dir = MemoryDirectory::new(next_inode);
        },
        None => return Err(DirError::EntryNotFound(path.to_string()))
      }

      curr_dir = Rc::new(RefCell::new(next_dir));
    }
    Ok(curr_dir)
  }


  ///Searches a given DIRECTORY for a DIRECTORY ENTRY with the given PAT
  ///
  ///Returns a tuple containing the DISK ENTRY and its OFST within DIRECTORY if found, None if not
  fn lookup(&self, state: &FileSystem, pat: &str) -> Result<Option<(DiskDirectory, u32)>, DirError> {
    let mut ofst = DIR_ENTRY_SIZE;
    let mut buffer = [0u8; DIR_ENTRY_SIZE as usize];

    while self.inode.borrow_mut().read_at(&state.block, &state.cache, &mut buffer, DIR_ENTRY_SIZE, ofst)? == DIR_ENTRY_SIZE {
      let entry = from_bytes::<DiskDirectory>(&buffer);
      let entry_name = entry.name_to_string();

      if entry.in_use == 1u8 && entry_name == pat {
        return Ok(Some((entry.to_owned(), ofst)))
      }

      ofst += DIR_ENTRY_SIZE;
    }
    Ok(None)
  }


  pub fn search(&self, state: &mut FileSystem, pat: &str) -> Result<Rc<RefCell<MemoryInode>>, DirError> {
    let mut buffer = [0u8; DIR_ENTRY_SIZE as usize];

    match pat {
      "." => {
        let sector = self.inode.borrow().inode_num();
        return Ok(state.inode_list.open_inode(&state.block, &state.cache, sector)?)
      },
      ".." => {
        self.inode.borrow().read_at(&state.block, &state.cache, &mut buffer, DIR_ENTRY_SIZE, 0)?;
        let entry = from_bytes::<DiskDirectory>(&buffer);
        return Ok(state.inode_list.open_inode(&state.block, &state.cache, entry.sector)?)
      },
      _ => {
        match self.lookup(state, pat)? {
          Some((entry, _)) => return Ok(state.inode_list.open_inode(&state.block, &state.cache, entry.sector)?),
          None => return Err(DirError::EntryNotFound(pat.to_string()))
        }
      }
    }
  }

  pub fn is_empty(&self, state: &FileSystem) -> Result<bool, DirError> {
    let mut buffer = [0u8; DIR_ENTRY_SIZE as usize];
    let mut ofst = DIR_ENTRY_SIZE;

    while self.inode.borrow_mut().read_at(&state.block, &state.cache, &mut buffer, DIR_ENTRY_SIZE, ofst)? == DIR_ENTRY_SIZE {
      let entry = from_bytes::<DiskDirectory>(&buffer);
      if entry.in_use == 1u8 {
        return Ok(false)
      }

      ofst += DIR_ENTRY_SIZE;
    }
    return Ok(true)
  }

  pub fn add(dir: RefMut<Self>, state: &mut FileSystem, name: &str, sector: BlockSectorT, is_dir: bool) -> Result<(), DirError> {
    if name.is_empty() || name.len() > NAME_MAX {
      return Err(DirError::InvalidName())
    }

    if let Some(_) = dir.lookup(state, name)? {
      return Err(DirError::EntryAlreadyExists(name.to_string()))
    }

    let mut entry = DiskDirectory::new(0);

    if is_dir {
      let child_inode = state.inode_list.open_inode(&state.block, &state.cache, sector)?;
      let child_inode_sector = child_inode.borrow().inode_num();
      entry.sector = child_inode_sector;

      let bytes_wrote = child_inode.borrow_mut().write_at(state, bytes_of(&entry), DIR_ENTRY_SIZE, 0)?;

      if bytes_wrote != DIR_ENTRY_SIZE {
        InodeList::close_inode(state, child_inode_sector)?;
        return Err(DirError::CreationFailedBytesMissing())
      }
    }

    let mut ofst = 0;
    let mut buffer = [0u8; DIR_ENTRY_SIZE as usize];

    while dir.inode.borrow_mut().read_at(&state.block, &state.cache, &mut buffer, DIR_ENTRY_SIZE, ofst)? == DIR_ENTRY_SIZE {
      let read_entry = from_bytes::<DiskDirectory>(&buffer);
      if read_entry.in_use == 1u8 {
        break;
      }
      ofst += DIR_ENTRY_SIZE;
    }

    entry.in_use = 1u8;
    entry.name.copy_from_slice(name.as_bytes());
    entry.sector = sector;

    let bytes_wrote = dir.inode.borrow_mut().write_at(state, bytes_of(&entry), DIR_ENTRY_SIZE, ofst)?;
    if bytes_wrote != DIR_ENTRY_SIZE {
      return Err(DirError::CreationFailedBytesMissing())
    }

    Ok(())
  }

  ///Removes an entry with NAME in DIR
  pub fn remove(dir: RefMut<Self>, state: &mut FileSystem, name: &str) -> Result<(), DirError> {
    match dir.lookup(state, name)? {
      Some((mut sub_entry, ofst)) => {
        let sub_inode = state.inode_list.open_inode(&state.block, &state.cache, sub_entry.sector)?;

        if sub_inode.borrow().is_dir() {
          let sub_dir = Self::new(sub_inode);
          if !sub_dir.is_empty(state)? {
            return Err(DirError::CannotDeleteNonEmptyDir(name.to_string()))
          }
        }

        sub_entry.in_use = 0u8;
        let bytes_wrote = dir.inode.borrow_mut().write_at(state, bytes_of(&sub_entry), DIR_ENTRY_SIZE, ofst)?;
        if bytes_wrote != DIR_ENTRY_SIZE {
          return Err(DirError::CreationFailedBytesMissing())
        }

        return Ok(())
      },
      None => return Err(DirError::EntryNotFound(name.to_string()))
    }
  }

  ///Reads all directory entries in the given DIR and returns in Vec
  pub fn read_names(&self, state: &mut FileSystem) -> Result<Vec<String>, DirError> {
    let mut buffer = [0u8; DIR_ENTRY_SIZE as usize];
    let mut ofst = DIR_ENTRY_SIZE;
    let mut result = Vec::<String>::new();

    while self.inode.borrow().read_at(&state.block, &state.cache, &mut buffer, DIR_ENTRY_SIZE, ofst)? == DIR_ENTRY_SIZE {
      let entry = from_bytes::<DiskDirectory>(&buffer);
      if entry.in_use == 1u8 {
        result.push(entry.name_to_string());
      }
      ofst += DIR_ENTRY_SIZE;
    }
    Ok(result)
  }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C, packed)]
struct DiskDirectory {
  name: [u8; NAME_MAX],
  in_use: u8,
  sector: BlockSectorT
}

impl DiskDirectory {
  fn new(sector: BlockSectorT) -> Self {
    Self {
      name: [0u8; NAME_MAX],
      in_use: 0u8, //false
      sector
    }
  }

  fn name_to_string(&self) -> String {
    String::from_utf8_lossy(&self.name).to_string()
  }
}
