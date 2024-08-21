use std::{
  error::Error,
  fmt::{
    Display,
    Formatter,
    Result
  }
};

use super::{
  CacheError,
  freemap_errors::FreemapError
};

#[derive(Debug)]
pub (crate) enum InodeError {
  OffsetOutOfBounds(u32, u32),
  IndexOutOfBounds(u32),
  WriteDenied(),
  CacheError(CacheError),
  FreemapError(FreemapError)
}

impl Error for InodeError {}

impl Display for InodeError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match self {
      Self::OffsetOutOfBounds(offset, actual) => write!(f, "Offset: {} past inode length: {}", offset, actual),
      Self::IndexOutOfBounds(idx) => write!(f, "Index: {} past max inode length", idx),
      Self::WriteDenied() => write!(f, "Write denied for given inode"),
      Self::CacheError(e) => write!(f, "Cache Error: {:?}", e),
      Self::FreemapError(e) => write!(f, "Freemap Error: {:?}", e)
    }
  }
}

impl From<CacheError> for InodeError {
  fn from(e: CacheError) -> Self {
    Self::CacheError(e)
  }
}

impl From<FreemapError> for InodeError {
  fn from(e: FreemapError) -> Self {
    Self::FreemapError(e)
  }
}
