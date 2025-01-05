use std::{
  error::Error,
  fmt::{
    Display,
    Formatter,
    Result
  }
};

use cache_errors::CacheError;
use dir_errors::DirError;
use file_errors::FileError;
use freemap_errors::FreemapError;
use inode_errors::InodeError;

pub(crate) mod bitmap_errors;
pub(crate) mod block_errors;
pub(crate) mod cache_errors;
pub(crate) mod controller_errors;
pub(crate) mod dir_errors;
pub(crate) mod file_errors;
pub(crate) mod freemap_errors;
pub(crate) mod inode_errors;

#[derive(Debug)]
pub enum FSErrors {
  BlockError(block_errors::BlockError),
  CacheError(cache_errors::CacheError),
  ControllerError(controller_errors::ControllerError),
  DirError(dir_errors::DirError),
  FileError(file_errors::FileError),
  FreemapError(freemap_errors::FreemapError),
  InodeError(inode_errors::InodeError),
  InvalidName(String, usize),
  IOError(std::io::Error)
}

impl Error for FSErrors {}

impl Display for FSErrors {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match self {
      Self::BlockError(e) => write!(f, "Block Error: {:?}", e),
      Self::CacheError(e) => write!(f, "Cache Error: {:?}", e),
      Self::ControllerError(e) => write!(f, "Controller Error: {:?}", e),
      Self::DirError(e) => write!(f, "Dir Error: {:?}", e),
      Self::FileError(e) => write!(f, "File Error: {:?}", e),
      Self::FreemapError(e) => write!(f, "Freemap Error: {:?}", e),
      Self::InodeError(e) => write!(f, "Inode Error: {:?}", e),
      Self::InvalidName(name, len) => write!(f, "Invalid name: {}, len: {}, max len 256", name, len),
      Self::IOError(e) => write!(f, "IO Error: {}", e)
    }
  }
}

impl From<CacheError> for FSErrors {
  fn from(e: CacheError) -> Self {
    Self::CacheError(e)
  }
}

impl From<DirError> for FSErrors {
  fn from(e: DirError) -> Self {
    Self::DirError(e)
  }
}

impl From<FreemapError> for FSErrors {
  fn from(e: FreemapError) -> Self {
    Self::FreemapError(e)
  }
}

impl From<FileError> for FSErrors {
  fn from(e: FileError) -> Self {
    Self::FileError(e)
  }
}

impl From<InodeError> for FSErrors {
  fn from(e: InodeError) -> Self {
    Self::InodeError(e)
  }
}

impl From<std::io::Error> for FSErrors {
  fn from(e: std::io::Error) -> Self {
    Self::IOError(e)
  }
}
