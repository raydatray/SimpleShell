use std::{
  error::Error,
  fmt::{
    Display,
    Formatter,
    Result
  },
  io,
};

#[derive(Debug)]
pub(crate) enum ControllerError {
  ChannelOccupied(usize),
  IOError(io::Error)
}

impl Display for ControllerError {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match self {
      Self::ChannelOccupied(channel_num) => write!(f, "Channel #: {} fully occupied", channel_num),
      Self::IOError(e) => write!(f, "IO error: {:?}", e)
    }
  }
}

impl Error for ControllerError {}

impl From<io::Error> for ControllerError {
  fn from(e: io::Error) -> Self {
    Self::IOError(e)
  }
}
