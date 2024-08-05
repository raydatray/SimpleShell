use crate::fs::{bitmap::Bitmap, inode_dup::MemoryInode};
use crate::fs::inode::DiskInode;
use crate::fs::file_sys::State;
use super::{block::BlockSectorT, file::File, file_sys::{FREE_MAP_SECTOR, ROOT_DIR_SECTOR}, fs_errors::FsErrors, inode_dup::InodeList};

pub struct Freemap {
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

  pub fn num_free_sectors(&self) -> u32 {
    let bitmap = self.inner.borrow();
    bitmap.count(0, bitmap.get_size(), false)
  }

  ///Allocates CNT consecutive sectors, and returns the first sector if successful
  pub fn allocate(&self, cnt: u32) -> Result<BlockSectorT, FsErrors> {
    let sector = self.inner.scan_and_flip(0, cnt, false)?;

    if let Some(file) = &mut self.file {
      match self.inner.write_to_file(&mut self, file) {
        Ok(_) => { return Ok(sector) },
        Err(_) => {
          self.inner.set_multiple(sector, cnt, false);
          return Err(todo!())
        }
      }
    }
    Err(todo!())
  }

  pub fn release(&mut self, sector: BlockSectorT, cnt: u32) -> Result<(), FsErrors> {
    assert!(self.inner.all(sector, cnt));

    self.inner.set_multiple(sector, cnt, false);

    if let Some(file) = &mut self.file {
      self.inner.write_to_file(&mut self, file)?;
      Ok(())
    } else {
      todo!("Err")
    }
  }

  pub fn open_from_file(&mut self, inode_list: &mut InodeList) -> Result<(), FsErrors> {
    assert!(self.file.is_none());

    let freemap_inode = inode_list.open_inode(FREE_MAP_SECTOR)?;
    let mut freemap_file = File::open(freemap_inode);
    self.inner.read_from_file(&mut freemap_file)?;
    self.file = Some(freemap_file);
    Ok(())
  }

  pub fn close(&mut self, inode_list: &mut InodeList) -> Result<(), FsErrors> {
    if let Some(mut file) = self.file.take(){
      file.close(inode_list)
    } else {
      Err(todo!())
    }
  }

  ///Creates a new FREEMAP on the provided BLOCK. Only used when formatting the filesystem
  ///
  ///Existing files will not be overwritten, however they will be all marked as FREE
  ///FREE_MAP_SECTOR (Sector 0) will be overwritten
  pub fn create_on_disk(state: &mut State) -> Result<(), FsErrors> {
    let size = state.freemap.inner.get_size();
    let _ = DiskInode::new(state, FREE_MAP_SECTOR, size, false)?;
    let freemap_inode =  state.inode_list.open_inode(FREE_MAP_SECTOR)?;
    let mut freemap_file = File::open(freemap_inode);

    state.freemap.inner.write_to_file(&mut state.freemap, &mut freemap_file)?;
    Ok(())
  }
}
