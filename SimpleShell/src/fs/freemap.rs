use crate::fs::{
  bitmap::Bitmap,
  file_sys::{
    FREE_MAP_SECTOR,
    ROOT_DIR_SECTOR
  },
  file::File
};

pub(crate) struct Freemap {
  inner: Bitmap,
  file: Option<File>
}

impl Freemap {
  pub fn new(block_size: u32) -> Self {
    let bitmap = Bitmap::new(block_size - 1);
    bitmap.mark(FREE_MAP_SECTOR);
    bitmap.mark(ROOT_DIR_SECTOR);

    Self {
      inner: bitmap,
      file: None
    }
  }
}
