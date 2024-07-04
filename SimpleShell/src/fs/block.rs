use crate::fs::fs_errors::FsErrors;
use crate::fs::ata::AtaDisk;

pub const BLOCK_SECTOR_SIZE: u8 = 512;
pub type BlockSectorT = u32;

struct HardwareOperations {
  read: fn(&AtaDisk, BlockSectorT, &mut Vec<u8>),
  write: fn(&AtaDisk, BlockSectorT, &Vec<u8>)
}

impl HardwareOperations {
  fn new(disk: &AtaDisk) -> Self {
    Self {
      read: AtaDisk::ada_disk_read,
      write: AtaDisk::ada_disk_write
    }
  }
}


pub struct Block {
  name: String,
  file_name : String,
  size : BlockSectorT,
  write_count : u64,
  read_count : u64
}

impl Block {
  fn check_sector(&self, sector: BlockSectorT) ->  Result<(), FsErrors> {
    if sector >= self.size {
      return Err(FsErrors::SectorOutOfBounds(sector));
    }
    Ok(())
  }

  fn read_block_to_buffer() -> Result<(), FsErrors> {
    todo!();
  }

  fn write_buffer_to_block() -> Result<(), FsErrors> {
    todo!();
  }

  pub fn get_block_size(&self) -> BlockSectorT {
    self.size
  }

  pub fn get_block_name(&self) -> String {
    self.name.clone()
  }
}
