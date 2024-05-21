#![allow(warnings)] //Suppress warnings while devving

mod shellmemory;
mod interpreter;
mod kernel;
mod pcb;
mod errors;

use std::env;

fn main() -> std::io::Result<()> {
  let args = env::args();

  if args.len() < 2 {
    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Error: SimpleShell must be called with the hard drive name as an argument."));
  }
  Ok(())
}
