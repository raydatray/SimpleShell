use std::{
  fs::{File, OpenOptions},
  os::unix::fs::FileExt
};

use crate::fs::block::{BLOCK_SECTOR_SIZE, BlockSectorT};
use crate::fs::fs_errors::FsErrors;

const CHANNEL_COUNT: usize = 2usize;
const DEVICE_COUNT: usize = 2usize;

//Top level controller, where we support two "legacy" ATA channels
struct Controller {
  channels: [Channel; CHANNEL_COUNT]
}

//An ATA channel, where we support up to two disks
struct Channel {
  name: String,
  devices: [Option<AtaDisk>; DEVICE_COUNT]
}

enum DiskType {
  Master,
  Slave
}

pub struct AtaDisk {
  name: String,
  channel: usize, //Store the channel index within the controller instead
  disk_type: DiskType,
  is_ata: bool,
  file_name: String,
  file_descriptor: File
}

impl Controller {
  fn new() -> Self {
    Self {
      channels: [
        Channel::new("ide0"),
        Channel::new("ide1")
      ]
    }
  }
}

impl Channel {
  fn new(name: &str) -> Self {
    Self {
      name: name.to_string(),
      devices: [None, None]
    }
  }
}

impl AtaDisk {
  fn new(hd: String, channel_num: usize, disk_suffix: usize) -> Result<Self, FsErrors> {
    let disk_name = format!("hd{}", disk_suffix);
    let file_descriptor = OpenOptions::new().read(true).write(true).open(&hd)?;

    Ok(
      Self {
        name: disk_name,
        channel: channel_num,
        disk_type: DiskType::Master,
        is_ata: true,
        file_name: hd,
        file_descriptor
      }
    )
  }

  fn identify_ata_device(&self) -> Result<(), FsErrors> {
    todo!()
  }

  pub fn ata_disk_read(&self, sector_num: BlockSectorT, buffer: &mut [u8]) -> Result<(), FsErrors> {
    self.file_descriptor.read_exact_at(buffer, (sector_num * BLOCK_SECTOR_SIZE as u32) as u64)?;
    Ok(())
  }

  pub fn ata_disk_write(&self, sector_num: BlockSectorT, buffer: &[u8]) -> Result<(), FsErrors> {
    self.file_descriptor.write_all_at(buffer, (sector_num * BLOCK_SECTOR_SIZE as u32) as u64)?;
    Ok(())
  }
}
