use crate::fs::fs_errors::FsErrors;
use crate::fs::ata::AtaDisk;

pub const BLOCK_SECTOR_SIZE: u16 = 512u16;
pub type BlockSectorT = u32;

struct HardwareOperations<'a> {
  hardware_read: Box<dyn Fn(BlockSectorT, &mut Vec<u8>) -> Result<(), FsErrors> + 'a>,
  hardware_write: Box<dyn Fn(BlockSectorT, &Vec<u8>) -> Result<(), FsErrors> + 'a>
}

impl<'a> HardwareOperations<'a> {
  fn new(disk: &'a AtaDisk) -> HardwareOperations<'a> {
    HardwareOperations {
      hardware_read: Box::new(move |sector_num, buffer: &mut Vec<u8>| disk.ata_disk_read(sector_num, buffer)),
      hardware_write: Box::new(move |sector_num, buffer: &Vec<u8>| disk.ata_disk_write(sector_num, buffer))
    }
  }

  pub fn read_block_to_buffer(&self, sector: BlockSectorT, buffer: &mut Vec<u8>) -> Result<(), FsErrors> {
    (self.hardware_read)(sector, buffer)
  }

  pub fn write_buffer_to_block(&self, sector: BlockSectorT, buffer: &Vec<u8>) -> Result<(), FsErrors> {
    (self.hardware_write)(sector, buffer)
  }
}

pub struct Block<'a> {
  name: String,
  file_name : String,
  size : BlockSectorT,
  write_count : u64,
  read_count : u64,
  hardware_ops: HardwareOperations<'a>
  //todo: aux data?
}

impl<'a> Block<'a> {
  fn new(name: String, file_name: String, size: BlockSectorT, hardware_ops: HardwareOperations<'a>) -> Block<'a> {
    Block {
      name,
      file_name,
      size,
      write_count: 0u64,
      read_count: 0u64,
      hardware_ops
    }
  }
  fn check_sector(&self, sector: BlockSectorT) ->  Result<(), FsErrors> {
    if sector >= self.size {
      return Err(FsErrors::SectorOutOfBounds(sector));
    }
    Ok(())
  }

  fn read_block_to_buffer(&mut self, sector: BlockSectorT, buffer: &mut Vec<u8>) -> Result<(), FsErrors> {
    self.check_sector(sector)?;
    self.hardware_ops.read_block_to_buffer(sector, buffer)?;
    self.read_count += 1;
    Ok(())
  }

  fn write_buffer_to_block(&mut self, sector: BlockSectorT, buffer: &Vec<u8>) -> Result<(), FsErrors> {
    self.check_sector(sector)?;
    self.hardware_ops.write_buffer_to_block(sector, buffer)?;
    self.write_count += 1;
    Ok(())
  }

  pub fn get_block_size(&self) -> BlockSectorT {
    self.size
  }

  pub fn get_block_name(&self) -> String {
    self.name.clone()
  }
}