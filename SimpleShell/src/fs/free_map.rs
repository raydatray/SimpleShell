use crate::fs::{bitmap::Bitmap, inode_dup::MemoryInode};
use super::{block::BlockSectorT, file::File, file_sys::{FREE_MAP_SECTOR, ROOT_DIR_SECTOR}, fs_errors::FsErrors, inode_dup::InodeList};

pub struct Freemap {
  inner: Bitmap,
  file: Option<File>
}

impl Freemap {
  pub fn new(block_size: u32) -> Self {
    let mut bitmap = Bitmap::new(block_size - 1);
    bitmap.mark(FREE_MAP_SECTOR);
    bitmap.mark(ROOT_DIR_SECTOR);

    Self {
      inner: bitmap,
      file: None
    }
  }

  fn num_free_sectors(&self) -> u32 {
    self.inner.count(0, self.inner.get_bitmap_size(), false)
  }

  ///Allocates CNT consecutive sectors, and returns the first sector if successful
  pub fn allocate(&mut self, cnt: u32) -> Result<BlockSectorT, FsErrors> {
    let sector = self.inner.scan_and_flip(0, cnt, false)?;

    self.inner.
    self.inner.set_multiple(sector, cnt, false);


  }

  pub fn release(&mut self, sector: BlockSectorT, cnt: u32) -> Result<(), FsErrors> {
    assert!(self.inner.all(sector, cnt));

    self.inner.set_multiple(sector, cnt, false);

    if let Some(file) = self.file {
      return self.inner.write_to_file(&mut self, &mut file)
    } else {
      todo!("Err")
    }
  }

  fn open_from_file(&mut self, inode_list: &mut InodeList) {
    assert!(self.file.is_none());

    let mut free_map_file = File::open(inode_list.open_inode(FREE_MAP_SECTOR));
    self.file = Some(free_map_file);
    self.inner.read_from_file(&mut free_map_file);

  }

  pub fn close(&self, inode_list: &mut InodeList) -> Result<(), FsErrors>{
    return match self.file {
      Some(mut file) => {
        file.close(inode_list)
      },
      None => Err(todo!())
    }
  }

  pub fn create_on_disk(&self, cache: &Cache, block: &Block) {
    let inode = MemoryInode::new(block, cache)
  }
}
