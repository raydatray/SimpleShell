use std::cell::Cell;

use crate::fs::fs_errors::FsErrors;
use crate::fs::ata::AtaDisk;

pub const BLOCK_SECTOR_SIZE: u32 = 512u32;
pub type BlockSectorT = u32;

pub struct HardwareOperations<'a> {
  hardware_read: Box<dyn Fn(BlockSectorT, &mut [u8]) -> Result<(), FsErrors> + 'a>,
  hardware_write: Box<dyn Fn(BlockSectorT, &[u8]) -> Result<(), FsErrors> + 'a>
}

impl<'a> HardwareOperations<'a> {
  pub fn new(disk: &'a AtaDisk) -> HardwareOperations<'a> {
    HardwareOperations {
      //We "capture" &disk to "prebind" it to our function calls
      hardware_read: Box::new(move |sector_num, buffer: &mut [u8]| disk.ata_disk_read(sector_num, buffer)),
      hardware_write: Box::new(move |sector_num, buffer: &[u8]| disk.ata_disk_write(sector_num, buffer))
    }
  }

  pub fn read_block_to_buffer(&self, sector: BlockSectorT, buffer: &mut [u8]) -> Result<(), FsErrors> {
    (self.hardware_read)(sector, buffer)
  }

  pub fn write_buffer_to_block(&self, sector: BlockSectorT, buffer: &[u8]) -> Result<(), FsErrors> {
    (self.hardware_write)(sector, buffer)
  }
}

pub struct Block<'a> {
  name: String,
  file_name : String,
  size : BlockSectorT,
  write_count : Cell<u64>, //Allows all block methods to &self
  read_count : Cell<u64>,
  hardware_ops: HardwareOperations<'a>,
  //todo: aux data as referencE (Either partition or ata disk)
  //Can't use weak unless we use RC-allocated(dont want to!!)
}

impl<'a> Block<'a> {
  pub fn new(name: String, file_name: String, size: BlockSectorT, hardware_ops: HardwareOperations<'a>) -> Block<'a> {
    Block {
      name,
      file_name,
      size,
      write_count: Cell::new(0u64),
      read_count: Cell::new(0u64),
      hardware_ops
    }
  }
  fn check_sector(&self, sector: BlockSectorT) ->  Result<(), FsErrors> {
    if sector >= self.size {
      return Err(FsErrors::SectorOutOfBounds(sector));
    }
    Ok(())
  }

  pub fn read_block_to_buffer(&self, sector: BlockSectorT, buffer: &mut [u8]) -> Result<(), FsErrors> {
    self.check_sector(sector)?;
    self.hardware_ops.read_block_to_buffer(sector, buffer)?;

    let current_read_count = self.read_count.get();
    self.read_count.set(current_read_count + 1);

    Ok(())
  }

  pub fn write_buffer_to_block(&self, sector: BlockSectorT, buffer: &[u8]) -> Result<(), FsErrors> {
    self.check_sector(sector)?;
    self.hardware_ops.write_buffer_to_block(sector, buffer)?;

    let current_write_count = self.write_count.get();
    self.write_count.set(current_write_count + 1);

    Ok(())
  }

  pub fn get_block_size(&self) -> BlockSectorT {
    self.size
  }

  pub fn get_block_name(&self) -> String {
    self.name.clone()
  }
}
