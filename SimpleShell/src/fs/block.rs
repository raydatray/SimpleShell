use std::cell::Cell;

use crate::fs::fs_errors::FsErrors;
use crate::fs::ata::AtaDisk;

pub const BLOCK_SECTOR_SIZE: u32 = 512u32;
pub type BlockSectorT = u32;

///A struct that contains closures to the write and read operations on the physical hardware of the block
///
///These may reference a raw, unpartitioned block, or be "presectored" to partitions on a block
pub struct HardwareOperations<'a> {
  hardware_read: Box<dyn Fn(BlockSectorT, &mut [u8]) -> Result<(), FsErrors> + 'a>,
  hardware_write: Box<dyn Fn(BlockSectorT, &[u8]) -> Result<(), FsErrors> + 'a>
}

///Emulation of a block device, with associated hardware write and read operations
///
///A BLOCK is to be used immutably, as it does not require mutable references to perform any of its operations
///
///The block interfaces can be considered STABlE
pub struct Block<'a > {
  name: String,
  file_name : String,
  size : BlockSectorT,
  write_count : Cell<u64>,
  read_count : Cell<u64>,
  hardware_ops: HardwareOperations<'a>,
}

impl<'a> HardwareOperations<'a> {
  ///Capture closures to a "pure" block devices
  pub fn new(disk: &'a AtaDisk) -> HardwareOperations<'a> {
    HardwareOperations {
      hardware_read: Box::new(move |sector_num, buffer: &mut [u8]| disk.ata_disk_read(sector_num, buffer)),
      hardware_write: Box::new(move |sector_num, buffer: &[u8]| disk.ata_disk_write(sector_num, buffer))
    }
  }

  ///Interface to read to the underlying hardware
  pub fn read_block_to_buffer(&self, sector: BlockSectorT, buffer: &mut [u8]) -> Result<(), FsErrors> {
    (self.hardware_read)(sector, buffer)
  }

  ///Interface to write to the underlying hardware
  pub fn write_buffer_to_block(&self, sector: BlockSectorT, buffer: &[u8]) -> Result<(), FsErrors> {
    (self.hardware_write)(sector, buffer)
  }
}

impl<'a> Block<'a> {
  ///Create a new emulated block device
  pub fn new(name: String, file_name: String, size: BlockSectorT, hardware_ops: HardwareOperations<'a>) -> Self {
    Self {
      name,
      file_name,
      size,
      write_count: Cell::new(0u64),
      read_count: Cell::new(0u64),
      hardware_ops
    }
  }

  ///Check if a given SECTOR is in bounds of given BLOCK
  fn check_sector(&self, sector: BlockSectorT) ->  Result<(), FsErrors> {
    if sector >= self.size {
      return Err(FsErrors::SectorOutOfBounds(sector));
    }
    Ok(())
  }

  ///Read from the BLOCK into a BUFFER
  pub fn read_block_to_buffer(&self, sector: BlockSectorT, buffer: &mut [u8]) -> Result<(), FsErrors> {
    self.check_sector(sector)?;
    self.hardware_ops.read_block_to_buffer(sector, buffer)?;

    let current_read_count = self.read_count.get();
    self.read_count.set(current_read_count + 1);

    Ok(())
  }

  ///Write from the BLOCK into a BUFFER
  pub fn write_buffer_to_block(&self, sector: BlockSectorT, buffer: &[u8]) -> Result<(), FsErrors> {
    self.check_sector(sector)?;
    self.hardware_ops.write_buffer_to_block(sector, buffer)?;

    let current_write_count = self.write_count.get();
    self.write_count.set(current_write_count + 1);

    Ok(())
  }

  ///Return the size of the BLOCK
  pub fn get_block_size(&self) -> BlockSectorT {
    self.size
  }

  ///Return the name of the BLOCK
  pub fn get_block_name(&self) -> &str {
    &self.name
  }
}
