use std::{
  error::Error,
  fmt::{
    Display,
    Formatter,
    Result
  }
};

use super::file_errors::FileError;

#[derive(Debug)]
pub (crate) enum BitmapError {
  NoContiguousAllocationFound(u32),
  FileError(FileError)
}

impl Error for BitmapError {}

impl Display for BitmapError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match self {
      Self::NoContiguousAllocationFound(cnt) => write!(f, "No contiguous allocation of: {} bits found", cnt),
      Self::FileError(e) => write!(f, "Failed to read/write to bitmap file, File Error: {:?}", e)
    }
  }
}

impl From<FileError> for BitmapError {
  fn from(e: FileError) -> Self {
    Self::FileError(e)
  }
}
