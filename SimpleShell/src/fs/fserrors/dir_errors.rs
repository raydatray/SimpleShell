use std::{
  error::Error,
  fmt::{
    Display,
    Formatter,
    Result
  }
};

use super::{
  inode_errors::InodeError
};

#[derive(Debug)]
pub enum DirError {
  CannotDeleteNonEmptyDir(String),
  CreationFailedBytesMissing(),
  EntryNotFound(String),
  EntryAlreadyExists(String),
  InvalidName(),
  InodeError(Box<InodeError>)
}

impl Error for DirError {}

impl Display for DirError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match self {
      Self::CannotDeleteNonEmptyDir(pat) => write!(f, "Cannot delete non-empty directory: {}", pat),
      Self::CreationFailedBytesMissing() => write!(f, "Creation failed, full directory not written to disk"),
      Self::EntryNotFound(pat) => write!(f, "Entry not found with path: {}", pat),
      Self::EntryAlreadyExists(pat) => write!(f, "Entry with name: {}, already exists", pat),
      Self::InvalidName() => write!(f, "Invalid name provided, max 30 chars"),
      Self::InodeError(e) => write!(f, "Inode Error: {:?}", e)
    }
  }
}

impl From<InodeError> for DirError {
  fn from(e: InodeError) -> Self {
    Self::InodeError(Box::new(e))
  }
}
