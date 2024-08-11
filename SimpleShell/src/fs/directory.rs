use std::{
  mem,
  rc::Rc,
  cell::RefCell
};

use bytemuck::{
  bytes_of, from_bytes, Pod, Zeroable
};

use crate::fs::block::BlockSectorT;
use crate::fs::file_sys::{ROOT_DIR_SECTOR, State};
use crate::fs::fs_errors::FsErrors;
use crate::fs::inode::{DiskInode, MemoryInode};

const NAME_MAX: usize = 31; //30 bytes + 1 byte of null terminator
const DIRECTORY_ENTRY_SIZE: u32 = {
  let directory_entry_size = mem::size_of::<DirectoryEntry>();
  directory_entry_size as u32
};

pub struct Directory {
  inode: Rc<RefCell<MemoryInode>>,
  position: u32,
  open_cnt: u32 //Redundant due to RC counting?
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct DirectoryEntry {
  inode_sector: BlockSectorT,
  name: [u8; NAME_MAX],
  in_use: u8 // This is really a bool, but we can't POD that. Use 0 and 1
}

///Takes a PATH, and outputs a tuple DIRECTORY, FILE_NAME
///
///Ex.
///"/home/user/documents/file.txt" -> "/home/user/documents/, file.txt"
///
///"file.txt" -> "", "file.txt"
pub fn split_path_filename(path: &str) -> (&str, &str) {
    let path = path.trim_end_matches('/');

    match path.rfind('/') {
        Some(index) => (&path[..=index], &path[index + 1..]),
        None => ("", path),
    }
}

impl Directory {
  pub fn new(state: &mut State, sector: BlockSectorT, entry_cnt: u32) -> Result<(), FsErrors> {
    DiskInode::new(state, sector, DIRECTORY_ENTRY_SIZE * entry_cnt, true)?;

    let sector_inode = state.inode_list.open_inode(sector)?;
    let dir = Self::open(sector_inode.clone());
    let dir_entry = DirectoryEntry::new(sector);

    if DIRECTORY_ENTRY_SIZE  != sector_inode.borrow_mut().write_at(state, bytes_of(&dir_entry), DIRECTORY_ENTRY_SIZE, 0)? {
      dir.close()?;
      return Err(todo!());
    }
    dir.close()
  }

  //This is really scuffed, any type that is wrapped within an RC probably doesn't need the open_cnt
  //In C implementation we use the open_cnt to keep track of open pointers to deallocate when 0
  //But RC already does this
  pub fn open_root(state: &mut State) -> Result<Rc<RefCell<Directory>>, FsErrors> {
    match &state.cwd {
      Some(directory) => {
        return Ok(directory.clone())
      },
      None => {
        let cwd = Self::open(state.inode_list.open_inode(ROOT_DIR_SECTOR)?);

        let wrapped_cwd = Rc::new(RefCell::new(cwd));
        wrapped_cwd.borrow_mut().open_cnt += 1;

        let return_dir = wrapped_cwd.clone();
        state.cwd = Some(wrapped_cwd);

        return Ok(return_dir);
      }
    }
  }

  pub fn open_path(state: &mut State, path: &str) -> Result<Rc<RefCell<Directory>>, FsErrors>{
    let mut curr = if path.starts_with("/") {
      Directory::open_root(state)?
    } else {
      match &state.cwd {
        Some(cwd) => {
          cwd.clone()
        },
        None => {
          Directory::open_root(state)?
        }
      }
    };

    for token in path.split("/").filter(|&x| !x.is_empty()) {
      let next_dir;
      match curr.borrow().lookup(state, token)? {
        Some((directory_entry, _)) => {
          let next_inode = state.inode_list.open_inode(directory_entry.inode_sector)?;
          next_dir = Directory::open(next_inode);
        },
        None => {
          return Err(todo!()) //Maybe return option of none type?
        }
      }
      curr = Rc::new(RefCell::new(next_dir));
    }

    if curr.borrow().get_inode().borrow().is_removed(){
      return Err(todo!()) //Removed directory
    }

    Ok(curr)
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

  pub fn search(&self, state: &mut State, name: &str) -> Result<Rc<RefCell<MemoryInode>>, FsErrors> {
    let mut buffer = [0u8; DIRECTORY_ENTRY_SIZE as usize];

    match name {
      "." => {
        return Ok(self.inode.clone())
      },
      ".."=> {
        self.inode.borrow_mut().read_at(&state.block, &state.cache, &mut buffer, DIRECTORY_ENTRY_SIZE, 0)?;
        let entry: &DirectoryEntry = bytemuck::from_bytes(&buffer);
        return state.inode_list.open_inode(entry.inode_sector)
      },
      _ => {
        match self.lookup(state, name)? {
          Some((directory_entry, _)) => {
            return state.inode_list.open_inode(directory_entry.inode_sector)
          },
          None => {
            return Err(todo!())
          }
        }
      }
    }
  }

  ///Searches DIRECTORY for a file with given NAME
  ///
  ///Returns the ENTRY containing the target file, and the OFFSET of the entry
  ///CHECK IF YOU ACTUALLY NEED TO RETURN OFFSET
  fn lookup(&self, state: &State, name: &str) -> Result<Option<(DirectoryEntry, u32)>, FsErrors> {
    let mut offset = DIRECTORY_ENTRY_SIZE;
    let mut buffer = [0u8; DIRECTORY_ENTRY_SIZE as usize];

    while self.inode.borrow_mut().read_at(&state.block, &state.cache, &mut buffer, DIRECTORY_ENTRY_SIZE, offset)? == DIRECTORY_ENTRY_SIZE {
      let entry = from_bytes::<DirectoryEntry>(&buffer).to_owned();
      let entry_name: &str = &entry.name_to_string();

      if entry.in_use == 1u8 && name == entry_name {
        return Ok(Some((entry, offset)));
      }
      offset += DIRECTORY_ENTRY_SIZE;
    }
    Ok(None) //We didn't find the entry
  }

  pub fn get_inode(&self) -> Rc<RefCell<MemoryInode>> {
    self.inode.clone()
  }

  pub fn is_empty(&self, state: &State) -> Result<bool, FsErrors> {
    let mut buffer = [0u8; DIRECTORY_ENTRY_SIZE as usize];
    let mut offset = DIRECTORY_ENTRY_SIZE;

    while self.inode.borrow_mut().read_at(&state.block, &state.cache, &mut buffer, DIRECTORY_ENTRY_SIZE, offset)? == DIRECTORY_ENTRY_SIZE {
      let entry: &DirectoryEntry = bytemuck::from_bytes(&buffer);
      if entry.in_use == 1u8 {
        return Ok(false);
      }

      offset += DIRECTORY_ENTRY_SIZE;
    }
    Ok(true)
  }

  pub fn add(&mut self, state: &mut State, name: &str, inode_sector: BlockSectorT, is_dir: bool) -> Result<(), FsErrors> {
    if name.is_empty() || name.len() > NAME_MAX {
      return Err(todo!("Invalid file name"));
    }

    match self.lookup(state, name)? {
      Some(_) => { //The file alr exists
        return Err(todo!("Name already exists"))
      },
      None => {}
    }

    let mut directory_entry = DirectoryEntry::new(0); //Careful, rewrite this inode sector

    if is_dir {
      let child_dir = Self::open(state.inode_list.open_inode(inode_sector)?);
      let child_dir_sector = child_dir.get_inode().borrow().get_inode_number();
      directory_entry.inode_sector = child_dir_sector;


      let buffer = bytes_of(&directory_entry);
      if child_dir.get_inode().borrow_mut().write_at(state, &buffer, DIRECTORY_ENTRY_SIZE, 0)? != DIRECTORY_ENTRY_SIZE {
        child_dir.close(); //We don't even need this?
        return Err(todo!());
      }
    }

    let mut offset = 0;
    let mut buffer = [0u8; DIRECTORY_ENTRY_SIZE as usize];

    while self.get_inode().borrow_mut().read_at(&state.block, &state.cache, &mut buffer, DIRECTORY_ENTRY_SIZE, offset)? == DIRECTORY_ENTRY_SIZE {
      let read_entry = from_bytes::<DirectoryEntry>(&buffer).to_owned();
      if read_entry.in_use == 0u8 {
        break;
      }
      offset += DIRECTORY_ENTRY_SIZE;
    }

    directory_entry.in_use = 1u8;
    directory_entry.name.copy_from_slice(name.as_bytes());
    directory_entry.inode_sector = inode_sector;

    match self.get_inode().borrow_mut().write_at(state, bytes_of(&directory_entry), DIRECTORY_ENTRY_SIZE, offset)? == DIRECTORY_ENTRY_SIZE {
      true => return Ok(()),
      false => return Err(todo!())
    }
  }

  pub fn remove(&mut self, state: &mut State, name: &str) -> Result<(), FsErrors> {
    match self.lookup(&state, name)? {
      Some((mut dir_entry, offset)) => {
        let inode = state.inode_list.open_inode(dir_entry.inode_sector)?;

        if inode.borrow().is_dir() {
          let target = Directory::open(inode.clone());
          if target.is_empty(&state)? {
            return Err(todo!());
          }
        }

        dir_entry.in_use = 0u8;
        let buffer = bytes_of(&dir_entry);

        if self.get_inode().borrow_mut().write_at(state, &buffer, DIRECTORY_ENTRY_SIZE, offset)? != DIRECTORY_ENTRY_SIZE {
          return Err(todo!());
        }
        return Ok(());
      },
      None => {
        return Err(todo!());
      }
    }
  }

  fn read_entry_names(&mut self, state: State) -> Result<String, FsErrors> {
    let mut buffer = [0u8; DIRECTORY_ENTRY_SIZE as usize];

    while self.inode.borrow_mut().read_at(&state.block, &state.cache, &mut buffer, DIRECTORY_ENTRY_SIZE, self.position)? == DIRECTORY_ENTRY_SIZE {
      self.position += DIRECTORY_ENTRY_SIZE;
      let directory_entry = from_bytes::<DirectoryEntry>(&buffer).to_owned();

      if directory_entry.in_use == 1u8 {
        return Ok(directory_entry.name_to_string())
      }
    }
    Err(todo!("Some error..."))
  }

  fn read_entry_inodes(&mut self, state: State) -> Result<(String, BlockSectorT), FsErrors> {
    let mut buffer = [0u8; DIRECTORY_ENTRY_SIZE as usize];

    while self.inode.borrow_mut().read_at(&state.block, &state.cache, &mut buffer, DIRECTORY_ENTRY_SIZE, self.position)? == DIRECTORY_ENTRY_SIZE {
      self.position += DIRECTORY_ENTRY_SIZE;
      let directory_entry: &DirectoryEntry = bytemuck::from_bytes(&buffer);

      if directory_entry.in_use == 1u8 {
        return Ok((directory_entry.name_to_string(), directory_entry.inode_sector))
      }
    }

    Err(todo!("Some error..."))
  }
}

impl DirectoryEntry {
  fn new(inode_sector: BlockSectorT) -> Self {
    Self {
      inode_sector,
      name: [0u8; NAME_MAX],
      in_use: 1u8 //"True"
    }
  }
  fn name_to_string(&self) -> String {
    String::from_utf8_lossy(&self.name).to_string()
  }
}
