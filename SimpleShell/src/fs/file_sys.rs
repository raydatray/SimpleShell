use crate::fs::{
  block::Block,
  cache::Cache,
  directory::Directory,
  file::FileTable,
  freemap::Freemap,
  fserrors::FSErrors,
  inode::InodeList
};

pub const FREE_MAP_SECTOR: u32 = 0u32;
pub const ROOT_DIR_SECTOR: u32 = 1u32;
pub const MAX_FILES_PER_DIRECTORY: u32 = 1000u32;

pub struct FileSystem<'file_sys> {
  pub block: Block<'file_sys>,
  pub cache: Cache,
  pub file_table: FileTable,
  pub freemap: Freemap,
  pub inode_list: InodeList,
  pub cwd: Option<Directory>
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

    Ok(file_sys)
  }
}
