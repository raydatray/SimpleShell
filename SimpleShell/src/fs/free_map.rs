use crate::fs::bitmap::Bitmap;
use super::{block::BlockSectorT, file_sys::{FREE_MAP_SECTOR, ROOT_DIR_SECTOR}, fs_errors::FsErrors};

pub struct Freemap {
  inner: Bitmap
}

impl Freemap {
  pub fn new(block_size: u32) -> Self {
    let mut bitmap = Bitmap::new(block_size - 1);
    bitmap.bitmap_mark(FREE_MAP_SECTOR);
    bitmap.bitmap_mark(ROOT_DIR_SECTOR);

    Self {
      inner: bitmap
    }
  }

  fn num_free_sectors(&self) -> u32 {
    self.inner.bitmap_cnt(0, self.inner.get_bitmap_size(), false)
  }

  ///Allocates CNT consecutive sectors, and returns the first sector if successful
  pub fn allocate(&mut self, cnt: u32) -> Result<BlockSectorT, FsErrors> {
    let sector = self.inner.bitmap_scan_and_flip(0, cnt, false)?;

  }
}


pub fn free_map_release() -> () {
  todo!()
}
