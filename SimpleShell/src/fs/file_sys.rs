use std::cell::{Ref, RefCell};
use std::rc::Rc;
use crate::fs::block::Block;
use crate::fs::cache::Cache;
use crate::fs::directory::Directory;
use crate::fs::free_map::Freemap;
use crate::fs::inode::InodeList;

use super::block::BlockSectorT;
use super::directory::split_path_filename;
use super::file::{File, FileTable};
use super::fs_errors::FsErrors;
use super::inode::{DiskInode, MemoryInode};

pub const FREE_MAP_SECTOR: u32 = 0u32;
pub const ROOT_DIR_SECTOR: u32 = 1u32;
pub const MAX_FILES_PER_DIRECTORY: u32 = 1000u32;

pub struct State<'file_sys> {
  pub block: Block<'file_sys>,
  pub cache: Cache,
  pub freemap: Freemap,
  pub inode_list: InodeList,
  pub file_table: FileTable,
  pub cwd: Option<Directory>,
}

impl <'file_sys> State <'file_sys> {
  pub fn file_sys_init(block: Block, format: bool) -> Result<Self, FsErrors> {
    //Inode, freemap, cache, format, open freemap
    let mut inode_list = InodeList::new();
    let mut freemap = Freemap::new(block.get_block_size());
    let cache = Cache::new();

    if format {
      todo!()
      Self::format()
    }

    freemap.open_from_file(&mut inode_list)?;
    println!("Number of free sectors: {}", freemap.num_free_sectors());

    let file_table = FileTable::new();

    Ok(
      Self {
        block,
        cache,
        freemap,
        inode_list,
        file_table,
        cwd: None //Sus, where do we get this from???
      }
    )
  }

  fn format(freemap: &) {

  }

  pub fn file_sys_close(&mut self) -> Result<(), FsErrors> {
    self.freemap.close(&mut self.inode_list)?;
    self.cache.close(&self.block)?;
    self.file_table.close(&mut self.inode_list)?;

    //Everything else will dropped when the State struct is dropped
    Ok(())
  }

  pub fn file_sys_create(&mut self, path: &str, initial_size: u32, is_dir: bool) -> Result<(), FsErrors> {
    let (_, file_name) = split_path_filename(path);
    let dir = Directory::open_root(self)?;

    let inode_sector =  self.freemap.allocate(1)?;
    let inode_result = DiskInode::new(self, inode_sector, initial_size, is_dir);
    let create_result = dir.add(self, file_name, inode_sector, is_dir);

    match (inode_result, create_result) {
      (Ok(_), Ok(_)) => {},
      (_, _) => {
        self.freemap.release(inode_sector,1)?;
        return Err(todo!());
      }
    }
    Ok(())
  }

  pub fn file_sys_open(&mut self, path: &str) -> Result<Rc<RefCell<File>>, FsErrors> {
    let (directory, file_name) = split_path_filename(path);
    let dir = Directory::open_root(self)?;
    let inode: Rc<RefCell<MemoryInode>>;

    match file_name.len() {
      0 => {
        inode = dir.get_inode();
      },
      _ => {
        inode = dir.lookup()?;
        dir.close()?;
      }
    }

    if inode.is_removed() {
      return Err(todo!());
    }

    File::open(inode)
  }

  pub fn file_sys_remove(path: &str) -> Result<(), FsErrors> {

  }

  pub fn file_sys_chdir(path: &str) -> Result<(), FsErrors> {

  }

  pub fn get_cwd(&self) -> Option<Rc<RefCell<Directory>>> {
    self.cwd.clone()
  }
}
