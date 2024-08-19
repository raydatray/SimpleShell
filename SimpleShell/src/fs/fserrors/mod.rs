use std::{
  error::Error,
  fmt::{
    Display,
    Formatter,
    Result
  }
};

use cache_errors::CacheError;

pub(crate) mod bitmap_errors;
pub(crate) mod block_errors;
pub(crate) mod cache_errors;
pub(crate) mod controller_errors;

#[derive(Debug)]
pub enum FSErrors {
  BitmapError(bitmap_errors::BitmapError),
  BlockError(block_errors::BlockError),
  CacheError(cache_errors::CacheError),
  ControllerError(controller_errors::ControllerError)
}

impl Error for FSErrors {}

impl Display for FSErrors {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match self {
      Self::BitmapError(e) => write!(f, "Bitmap Error: {:?}", e),
      Self::CacheError(e) => write!(f, "Cache Error: {:?}", e),
      Self::BlockError(e) => write!(f, "Block Error: {:?}", e),
      Self::ControllerError(e) => write!(f, "Controller Error: {:?}", e)
    }
  }
}

impl From<CacheError> for FSErrors {
  fn from(e: CacheError) -> Self {
    Self::CacheError(e)
  }
}
