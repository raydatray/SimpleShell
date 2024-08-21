use std::{
  error::Error,
  fmt::{
    Display,
    Formatter,
    Result
  }
};

use super::bitmap_errors::BitmapError;

#[derive(Debug)]
pub (crate) enum FreemapError {
  BitmapError(BitmapError)
}


impl Error for FreemapError {}

impl Display for FreemapError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match self {
      Self::BitmapError(e) => write!(f, "Bitmap Error: {:?}", e)
    }
  }
}

impl From<BitmapError> for FreemapError {
  fn from(e: BitmapError) -> Self {
    Self::BitmapError(e)
  }
}
