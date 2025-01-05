use std::cell::Cell;

use crate::fs::{
  ata::AtaDisk,
  fserrors::{
    block_errors::BlockError,
    controller_errors::ControllerError
  }
};

pub const BLOCK_SECTOR_SIZE: u32 = 512u32;
pub type BlockSectorT = u32;

pub struct Block<'disk> {
  name: String,
  file_name: String,
  size: BlockSectorT,
  write_cnt: Cell<u32>,
  read_cnt: Cell<u32>,
  hardware: HardwareOps<'disk>
}

impl<'disk> Block<'disk> {
  pub fn new(name: String, file_name: String, size: BlockSectorT, hardware: HardwareOps<'disk>) -> Self {
    Self {
      name,
      file_name,
      size,
      write_cnt: Cell::new(0u32),
      read_cnt: Cell::new(0u32),
      hardware
    }
  }

  pub fn get_size(&self) -> BlockSectorT {
    self.size
  }

  pub fn get_name(&self) -> &str {
    &self.name
  }

  fn check_sector(&self, sector: BlockSectorT) -> Result<(), BlockError> {
    if sector >= self.size {
      return Err(BlockError::SectorOutOfBounds(sector));
    }
    Ok(())
  }

  pub fn read_to_buffer(&self, sector: BlockSectorT, buffer: &mut [u8]) -> Result<(), BlockError> {
    assert_eq!(buffer.len(), BLOCK_SECTOR_SIZE as usize);

    self.check_sector(sector)?;
    self.hardware.read(sector, buffer)?;

    let curr_read_cnt = self.read_cnt.get();
    self.read_cnt.set(curr_read_cnt + 1);

    Ok(())
  }

  pub fn write_from_buffer(&self, sector: BlockSectorT, buffer: &[u8]) -> Result<(), BlockError> {
    assert_eq!(buffer.len(), BLOCK_SECTOR_SIZE as usize);

    self.check_sector(sector)?;
    self.hardware.write(sector, buffer)?;

    let curr_write_cnt = self.write_cnt.get();
    self.write_cnt.set(curr_write_cnt + 1);

    Ok(())
  }
}



pub struct HardwareOps<'disk> {
  read: Box<dyn Fn(BlockSectorT, &mut [u8]) -> Result<(), ControllerError> + 'disk>,
  write: Box<dyn Fn(BlockSectorT, &[u8]) -> Result<(), ControllerError> + 'disk>
}

impl<'disk> HardwareOps <'disk> {
  pub fn new(disk: &'disk AtaDisk) -> Self {
    Self {
      read: Box::new(move |sector: BlockSectorT, buffer: &mut [u8]| disk.read(sector, buffer)),
      write: Box::new(move |sector: BlockSectorT, buffer: &[u8]| disk.write(sector, buffer))
    }
  }

  pub fn read(&self, sector: BlockSectorT, buffer: &mut [u8]) -> Result<(), ControllerError> {
    (self.read)(sector, buffer)
  }

  pub fn write(&self, sector: BlockSectorT, buffer: &[u8]) -> Result<(), ControllerError> {
    (self.write)(sector, buffer)
  }
}
