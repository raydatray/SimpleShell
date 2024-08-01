use std::cell::{Ref, RefCell};
use std::rc::Rc;
use crate::fs::block::Block;
use crate::fs::cache::Cache;
use crate::fs::directory::Directory;
use crate::fs::free_map::Freemap;
use crate::fs::inode_dup::InodeList;

pub const FREE_MAP_SECTOR: u32 = 0u32;
pub const ROOT_DIR_SECTOR: u32 = 1u32;
pub const MAX_FILES_PER_DIRECTORY: u32 = 1000u32;

pub struct State {
  block: Block,
  cache: Cache,
  freemap: Freemap,
  pub inode_list: InodeList,
  cwd: Option<Rc<RefCell<Directory>>>,
}

impl State {
  pub fn get_cwd(&self) -> Option<Rc<RefCell<Directory>>> {
    self.cwd.clone()
  }

}