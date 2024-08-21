use crate::fs::{
  bitmap::Bitmap,
  block::{
    BlockSectorT
  },
  file_sys::{
    FREE_MAP_SECTOR,
    ROOT_DIR_SECTOR,
    FileSystem
  },
  file::File,
  fserrors::freemap_errors::FreemapError,
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

  pub fn allocate(state: &mut FileSystem, cnt: u32) -> Result<BlockSectorT, FreemapError> {
    todo!()
  }

  ///Releases CNT sectors starting from SECTOR, writing the result to file
  pub fn release(state: &mut FileSystem, sector: BlockSectorT, cnt: u32) -> Result<(), FreemapError> {
    assert!(state.freemap.inner.all(sector, cnt));

    state.freemap.inner.set_multiple(sector, cnt, false);

    todo!()
  }
}
