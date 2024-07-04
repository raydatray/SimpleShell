use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

use super::block::BlockSectorT;

#[derive(Debug, Eq, PartialEq)]
pub enum FsErrors {
  SectorOutOfBounds(BlockSectorT),
  NotATADevice,
  IoError(String)
}

impl Error for FsErrors {}

impl Display for FsErrors {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let message: String = match self {
      Self::SectorOutOfBounds(index) => format!("Sector {} is out of bounds", index),
      Self::NotATADevice => "Device is not ATA compliant".parse().unwrap(),
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
