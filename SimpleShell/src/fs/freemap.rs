use crate::fs::{
  bitmap::Bitmap,
  block::BlockSectorT,
  file_sys::{
    FREE_MAP_SECTOR,
    ROOT_DIR_SECTOR,
    FileSystem
  },
  file::File,
  fserrors::freemap_errors::FreemapError,
  inode::DiskInode
};

pub (crate) struct Freemap {
  pub inner: Bitmap,
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

  pub fn open_from_file(state: &mut FileSystem) -> Result<(), FreemapError> {
    assert!(state.freemap.file.is_none());

    let freemap_inode = state.inode_list.open_inode(&state.block, &state.cache, FREE_MAP_SECTOR)?;
    let mut freemap_file = File::open(freemap_inode);
    state.freemap.inner.read_from_file(&state.block, &state.cache, &mut freemap_file)?;
    state.freemap.file = Some(freemap_file);

    Ok(())
  }

  pub fn create_on_disk(state: &mut FileSystem) -> Result<(), FreemapError> {
    let size = state.freemap.inner.get_size();
    let _ = DiskInode::new(state, FREE_MAP_SECTOR, size, false)?;
    let freemap_inode = state.inode_list.open_inode(&state.block, &state.cache, FREE_MAP_SECTOR)?;
    let mut freemap_file = File::open(freemap_inode);

    Self::write_to_file(state)?;
    Ok(())
  }

  pub fn close(state: &mut FileSystem) -> Result<(), FreemapError> {
    if let None = state.freemap.file {
      return Err(FreemapError::NoFileAssigned())
    }

    let file = state.freemap.file.take().unwrap();
    Self::write_to_file(state)?;
    file.close(state)?;
    Ok(())
  }

  pub fn allocate(state: &mut FileSystem, cnt: u32) -> Result<BlockSectorT, FreemapError> {
    if let None = state.freemap.file {
      return Err(FreemapError::NoFileAssigned())
    }

    let sector = state.freemap.inner.scan_and_flip(0, cnt, false)?;
    Self::write_to_file(state)?;
    Ok(sector)
  }

  ///Releases CNT sectors starting from SECTOR, writing the result to file
  pub fn release(state: &mut FileSystem, sector: BlockSectorT, cnt: u32) -> Result<(), FreemapError> {
    if let None = state.freemap.file {
      return Err(FreemapError::NoFileAssigned())
    }

    assert!(state.freemap.inner.all(sector, cnt));
    state.freemap.inner.set_multiple(sector, cnt, false);

    Self::write_to_file(state)
  }

  ///Returns the numebr of free sectors on the FREEMAP
  pub fn num_free_sectors(&self) -> u32 {
    self.inner.count(0, self.inner.get_size(), false)
  }

  pub fn write_to_file(state: &mut FileSystem) -> Result<(), FreemapError> {
    let bits = state.freemap.inner.get_bits();
    let len = bits.len() as u32;

    let file = state.freemap.file.take().unwrap();
    let bytes_wrote = file.write_at(state, &bits, len, 0)?;

    assert_eq!(bytes_wrote, len);
    state.freemap.file = Some(file);
    Ok(())
  }
}
