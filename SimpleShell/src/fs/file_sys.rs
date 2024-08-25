use std::{cell::RefCell, rc::Rc};

use crate::fs::{
  block::Block,
  cache::Cache,
  directory::MemoryDirectory,
  file::FileTable,
  freemap::Freemap,
  fserrors::FSErrors,
  inode::InodeList
};

use super::{directory::split_path, inode::DiskInode};

pub const FREE_MAP_SECTOR: u32 = 0u32;
pub const ROOT_DIR_SECTOR: u32 = 1u32;
pub const MAX_FILES_PER_DIRECTORY: u32 = 1000u32;

pub struct FileSystem<'file_sys> {
  pub block: Block<'file_sys>,
  pub cache: Cache,
  pub file_table: FileTable,
  pub freemap: Freemap,
  pub inode_list: InodeList,
  pub cwd: Option<Rc<RefCell<MemoryDirectory>>>
}

impl<'file_sys> FileSystem<'file_sys> {
  pub fn new(block: Block<'file_sys>, format: bool) -> Result<Self, FSErrors> {
    let block_size = block.get_size();

    let mut file_sys = Self {
      block,
      cache: Cache::new(),
      freemap: Freemap::new(block_size),
      file_table: FileTable::new(),
      inode_list: InodeList::new(),
      cwd: None
    };

    if format {
      Self::format(&mut file_sys)?;
    }

    Freemap::open_from_file(&mut file_sys)?;
    println!("Number of free sectors: {}", file_sys.freemap.num_free_sectors());

    Ok(file_sys)
  }

  fn format(&mut self) -> Result<(), FSErrors> {
    println!("Formatting file system...");
    Freemap::create_on_disk(self)?;
    MemoryDirectory::new_on_disk(self, ROOT_DIR_SECTOR, MAX_FILES_PER_DIRECTORY)?;
    Freemap::close(self)?;
    Ok(())
  }


  pub fn close(&mut self) -> Result<(), FSErrors> {
    Freemap::close(self)?;
    Cache::close(&self.cache, &self.block)?;
    FileTable::close(self)?;
    Ok(())
  }

  pub fn create(&mut self, path: &str, init_size: u32, is_dir: bool) -> Result<(), FSErrors> {
    let (_, suffix) = split_path(path);
    let dir = MemoryDirectory::open_root(self)?;

    let sector = Freemap::allocate(self, 1)?;
    let _ = DiskInode::new(self, sector, init_size, is_dir)?;
    MemoryDirectory::add(dir.borrow_mut(), self, suffix, sector, is_dir)?;

    Ok(())
  }
}
