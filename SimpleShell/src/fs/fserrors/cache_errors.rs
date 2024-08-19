use std::{
  error::Error,
  fmt::{
    Display,
    Formatter,
    Result
  }
};

use crate::fs::{
  fserrors::block_errors::BlockError
};

#[derive(Debug)]
pub (crate) enum CacheError {
  FlushUnoccupiedEntry(),
  FlushNullDiskSector(),
  BlockError(BlockError)
}

impl Error for CacheError {}

impl Display for CacheError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match self {
      Self::FlushUnoccupiedEntry() => write!(f, "Attempted to flush unoccupied entry"),
      Self::FlushNullDiskSector() => write!(f, "Attempted to flush entry with no disk sector"),
      Self::BlockError(e) => write!(f, "Block error: {:?}", e)
    }
  }
}

impl From<BlockError> for CacheError {
  fn from(e: BlockError) -> Self {
    Self::BlockError(e)
  }
}
