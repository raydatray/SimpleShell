use std::{
  fs::{
    File,
    OpenOptions,
  },
  os::unix::fs::FileExt
};

use crate::fs::{
  block::{
    BlockSectorT,
    BLOCK_SECTOR_SIZE
  },
  fserrors::controller_errors::ControllerError
};

const CHANNEL_COUNT: usize = 2usize;
const DEVICE_COUNT: usize = 2usize;

///Top level controller of the file system
pub struct AtaController {
  channels: [Channel; CHANNEL_COUNT]
}

impl AtaController {
  pub fn new() -> Self {
    Self {
      channels: [Channel::new("ide0"), Channel::new("ide1")]
    }
  }

  pub fn init(file_name: &str) -> Result<Self, ControllerError> {
    let mut controller = Self::new();

    controller.add_device(file_name, 0usize)?;

    Ok(controller)
  }

  ///Add a new device given a file path at the given CHANNEL
  pub fn add_device(&mut self, file_name: &str, channel_num: usize) -> Result<(), ControllerError> {
    self.channels[channel_num].add_device(channel_num, file_name)
  }
}

struct Channel {
  name: String,
  devices: [Option<AtaDisk>; DEVICE_COUNT]
}

impl Channel {
  fn new(name: &str) -> Self {
    Self {
      name: name.to_owned(),
      devices: [None, None]
    }
  }

  pub fn add_device(&mut self, channel_num: usize, file_name: &str) -> Result<(), ControllerError> {
    for (i, device) in self.devices.iter_mut().enumerate() {
      if let None = device {
        let new_device = AtaDisk::new(file_name, channel_num, i)?;
        let _ = device.insert(new_device);
        return Ok(());
      }
    }

    Err(ControllerError::ChannelOccupied(channel_num))
  }
}

enum DiskType {
  Master,
  Slave
}

pub struct AtaDisk {
  channel_num: usize,
  disk_type: DiskType,
  file_descriptor: File,
  file_name: String,
  is_ata: bool,
  name: String
}

impl AtaDisk {
  fn new (file_name: &str, channel_num: usize, disk_num: usize) -> Result<Self, ControllerError> {
    let disk_name = format!("hd{}", (channel_num * 2) + disk_num);
    let file_descriptor = OpenOptions::new().read(true).write(true).open(&file_name)?;

    Ok(
      Self {
        channel_num,
        disk_type: DiskType::Master ,//Todo: make this dynamic
        file_descriptor,
        file_name: file_name.to_owned(),
        is_ata: true,
        name: disk_name
      }
    )
  }

  ///Reads exactly SIZE OF BUFFER from SECTOR_NUM into BUFFER
  ///
  ///Functionally, we use this to read an entire sector of a time from the file
  ///
  ///Result: Error if SIZE OF BUFFER not read
  pub fn read(&self, sector: BlockSectorT, buffer: &mut [u8]) -> Result<(), ControllerError>{
    assert_eq!(buffer.len(), BLOCK_SECTOR_SIZE as usize);
    self.file_descriptor.read_exact_at(buffer, (sector * BLOCK_SECTOR_SIZE) as u64)?;
    Ok(())
  }

  ///Writes exactly SIZE OF BUFFER from SECTOR_NUM into BUFFER
  ///
  ///Functionally, we use this to write an entire sector of a time to the file
  ///
  ///Result: Error if SIZE OF BUFFER not written to
  pub fn write(&self, sector: BlockSectorT, buffer: &[u8]) -> Result<(), ControllerError>{
    assert_eq!(buffer.len(), BLOCK_SECTOR_SIZE as usize);
    self.file_descriptor.write_all_at(buffer, (sector * BLOCK_SECTOR_SIZE) as u64)?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::io::Write;
  use super::*;
  use tempfile::{Builder, NamedTempFile, tempfile};
  use rand::{random, Rng};

  fn setup_test_file(len: usize) -> (NamedTempFile, Vec<u8>) {
    let mut temp_file = NamedTempFile::new().unwrap();

    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..len).map(|_| rng.gen()).collect();

    temp_file.write_all(&random_bytes).unwrap();

    (temp_file, random_bytes)
  }

  #[test]
  fn generate_file() {
    let (file, gen_buffer) = setup_test_file(1024);
    let path = file.path().to_path_buf();
    let opened_file = File::open(path).unwrap();
    println!("{}", opened_file.metadata().unwrap().len());
  }


  #[test]
  #[should_panic]
  fn test_non_conforming_buffer_read() {
    let (file_name, generated_buffer) = setup_test_file(1024);
    let disk = AtaDisk::new(file_name.path().to_path_buf().to_str().unwrap(), 0, 0).unwrap();
    let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize - 1];

    let _ = disk.read(0, &mut buffer);
  }

  #[test]
  #[should_panic]
  fn test_non_conforming_buffer_write() {
    let (file_name, generated_buffer) = setup_test_file(1024);
    let disk = AtaDisk::new(file_name.path().to_path_buf().to_str().unwrap(), 0, 0).unwrap();
    let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize - 1];

    let _ = disk.write(0, &mut buffer);
  }

  #[test]
  fn test_read_oob() {
    let (file, generated_buffer) = setup_test_file(1024);
    let disk = AtaDisk::new(file.path().to_path_buf().to_str().unwrap(), 0, 0).unwrap();
    let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];

    let read = disk.read(2, &mut buffer);
    assert!(read.is_err());
  }

  #[test]
  fn test_read() {
    let (file, generated_buffer) = setup_test_file(1024);
    let disk = AtaDisk::new(file.path().to_path_buf().to_str().unwrap(), 0, 0).unwrap();
    let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];

    let read = disk.read(1, &mut buffer);
    assert!(read.is_ok());

    assert_eq!(buffer, generated_buffer[1 * BLOCK_SECTOR_SIZE as usize..])
  }

}
