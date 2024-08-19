use std::{
  error::Error,
  fmt::{
    Display,
    Formatter,
    Result
  }
};

#[derive(Debug)]
pub (crate) enum BitmapError {
  NoContiguousAllocationFound(u32)
}

impl Error for BitmapError {}

impl Display for BitmapError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match self {
      Self::NoContiguousAllocationFound(cnt) => write!(f, "No contiguous allocation of: {} bits found", cnt)
    }
  }
}
