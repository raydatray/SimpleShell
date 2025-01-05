use std::{
  error::Error,
  fmt::{
    Display,
    Formatter,
    Result
  }
};

use crate::fs::{
  block::BlockSectorT,
  fserrors::controller_errors::ControllerError
};

#[derive(Debug)]
pub(crate) enum BlockError {
  SectorOutOfBounds(BlockSectorT),
  ControllerError(Box<ControllerError>)
}

impl Error for BlockError {}

impl Display for BlockError {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match self {
      Self::SectorOutOfBounds(sector) => write!(f, "Sector: {} out of bounds", sector),
      Self::ControllerError(e) => write!(f, "Controller Error: {:?}", e)
    }
  }
}

impl From<ControllerError> for BlockError {
  fn from(e: ControllerError) -> Self {
    Self::ControllerError(Box::new(e))
  }
}
