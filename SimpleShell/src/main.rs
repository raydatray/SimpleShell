mod shellmemory;
mod interpreter;
mod kernel;
mod pcb;
mod errors;

use std::io;
use std::io::Write;
use crate::errors::ShellErrors;
use crate::interpreter::parser;
use crate::shellmemory::ShellMemory;
use crate::kernel::Kernel;

fn main() -> Result<(), ShellErrors> {
  //Temporary values until we can capture these from compile or @ runtime
  const FRAME_STORE_SIZE: usize = 18;
  const VAR_STORE_SIZE: usize = 10;

  let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
  let mut kernel = Kernel::new();

  let mut buffer = String::new();
  let dummy_cwd = "dummyCwd".to_string();

  loop {
    print!("$ ");
    io::stdout().flush().expect("TODO: panic message");
    io::stdin().read_line(&mut buffer).expect("TODO: panic message");
    parser(Some(&mut kernel), &mut shell_memory, &mut buffer, &dummy_cwd)?;
    buffer.clear();
  }
}
