use std::{
  error::Error,
  fmt::{
    Display,
    Formatter,
    Result
  }
};

use super::inode_errors::InodeError;

#[derive(Debug)]
pub (crate) enum FileError {
  FileNotFound(String),
  InodeError(Box<InodeError>)
}

impl Error for FileError {}

impl Display for FileError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match self {
      Self::FileNotFound(file_name) => write!(f, "File with name: {} not found", file_name),
      Self::InodeError(e) => write!(f, "Inode Error: {:?}", e)
    }
  }
}

impl From<InodeError> for FileError {
  fn from(e: InodeError) -> Self {
    Self::InodeError(Box::new(e))
  }
}
