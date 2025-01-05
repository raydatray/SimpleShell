use std::{
  error::Error,
  fmt::{
    Display,
    Formatter,
    Result
  }
};

use super::{
  bitmap_errors::BitmapError,
  file_errors::FileError,
  inode_errors::InodeError
};

#[derive(Debug)]
pub (crate) enum FreemapError {
  NoFileAssigned(),
  BitmapError(Box<BitmapError>),
  FileError(Box<FileError>),
  InodeError(Box<InodeError>)
}


impl Error for FreemapError {}

impl Display for FreemapError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match self {
      Self::NoFileAssigned() => write!(f, "Attempted to write/read to file when unassigned"),
      Self::BitmapError(e) => write!(f, "Bitmap Error: {:?}", e),
      Self::FileError(e) => write!(f, "File Error: {:?}", e),
      Self::InodeError(e) => write!(f, "Inode Error: {:?}", e)
    }
  }
}

impl From<BitmapError> for FreemapError {
  fn from(e: BitmapError) -> Self {
    Self::BitmapError(Box::new(e))
  }
}

impl From<FileError> for FreemapError {
  fn from(e: FileError) -> Self {
    Self::FileError(Box::new(e))
  }
}

impl From<InodeError> for FreemapError {
  fn from(e: InodeError) -> Self {
    Self::InodeError(Box::new(e))
  }
}
