use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug, Eq, PartialEq)]
pub enum ShellErrors {
  PageFault(usize),
  InitialFrameAllocationFailed,
  NoFreePages,
  IoError(String)
}

impl Error for ShellErrors {}

impl Display for ShellErrors {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let message = match self {
      Self::PageFault(page_index) => format!("Page fault occurred @ index: {}", page_index),
      Self::InitialFrameAllocationFailed => "Insufficient memory to allocate initial pages".parse().unwrap(),
      Self::NoFreePages => "No free pages".parse().unwrap(),
      Self::IoError(v) => format!("{}",v)
    };

    write!(f, "Error: {message}")
  }
}

impl From<io::Error> for ShellErrors {
  fn from(value: io::Error) -> Self {
    ShellErrors::IoError(value.to_string())
  }
}
