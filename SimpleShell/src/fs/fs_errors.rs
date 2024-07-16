use std::error::Error;
use std::fmt::{format, Display, Formatter};
use std::io;

use super::block::BlockSectorT;

#[derive(Debug, Eq, PartialEq)]
pub enum FsErrors {
  SectorOutOfBounds(BlockSectorT),
  NotATADevice(String),
  InvalidPartitionTableSignature(String),
  InvalidExtendedPartitionTable(String,BlockSectorT),
  PartitionStartPastEOD(BlockSectorT),
  PartitionEndPastEOD(BlockSectorT),
  IoError(String)
}

impl Error for FsErrors {}

impl Display for FsErrors {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let message: String = match self {
      Self::SectorOutOfBounds(sector) => format!("Sector {} is out of bounds", sector),
      Self::NotATADevice(device_name) => format!("Device {} is not ATA compliant", device_name),
      Self::InvalidPartitionTableSignature(device_name) => format!("Device {} has an invalid partition table signature", device_name),
      Self::InvalidExtendedPartitionTable(devuice_name, sector) => format!("Device {} has an invalid extended partition table in sector {}", device_name, sector),
      Self::PartitionStartPastEOD(sector) => format!("Partition starts past EOD: Sector {}", sector),
      Self::PartitionEndPastEOD(sector) => format!("Partition end past EOD: Sector {}", sector),
      Self::IoError(v) => format!("{}", v)
    };

    write!(f, "Error: {message}")
  }
}

impl From<io::Error> for FsErrors {
  fn from(value: io::Error) -> Self {
    FsErrors::IoError(value.to_string())
  }
}
