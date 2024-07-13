use std::{fs::{metadata, File, Metadata}, io::Write, os::{macos::fs::MetadataExt, unix::fs::{FileExt, MetadataExt}}, rc::{Rc, Weak}};

use super::{block::{Block, BlockSectorT, HardwareOperations, BLOCK_SECTOR_SIZE}, fs_errors::FsErrors};

const CHANNEL_COUNT: u8 = 2u8;
const DEVICE_COUNT: u8 = 2u8;

//Top level controller, where we support two "legacy" ATA channels
struct Controller {
  channels: [Channel; CHANNEL_COUNT as usize]
}

//An ATA channel, where we support up to two disks
struct Channel {
  name: String,
  devices: [Rc<AtaDisk>; DEVICE_COUNT as usize]
}

enum DiskType {
  Master,
  Slave
}

pub struct AtaDisk {
  name: String,
  channel: Weak<Channel>,
  disk_type: DiskType,
  is_ata: bool,
  file_path: String,
  file_descriptor: File
}

impl Controller {

}



impl AtaDisk {
  fn identify_and_register_ata_device(&self) -> Result<(), FsErrors> {
    if !self.is_ata {
      todo!();
    }

    let meta = metadata(self.file_path)?;
    let capacity = meta.st_size() / BLOCK_SECTOR_SIZE as u64;
    let block_operations = HardwareOperations::new(&self);

    let block = Block::new(self.name, self.file_path, capacity as BlockSectorT, hardware_ops);
    //partition scan


  }

  pub fn ata_disk_read(&self, sector_num: BlockSectorT, buffer: &mut Vec<u8>) -> Result<(), FsErrors> {
    self.file_descriptor.read_exact_at(buffer, (sector_num * BLOCK_SECTOR_SIZE as u32) as u64)?;
    Ok(())
  }

  pub fn ata_disk_write(&self, sector_num: BlockSectorT, buffer: &Vec<u8>) -> Result<(), FsErrors> {
    self.file_descriptor.write_all_at(buffer, (sector_num * BLOCK_SECTOR_SIZE as u32) as u64)?;
    Ok(())
  }
}
