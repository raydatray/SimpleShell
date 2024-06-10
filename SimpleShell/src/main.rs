mod shellmemory;
mod interpreter;
mod kernel;
mod pcb;
mod errors;

use crate::shellmemory::ShellMemory;
use crate::kernel::Kernel;

fn main() -> std::io::Result<()> {
  //Temporary values until we can capture these from compile or @ runtime
  const FRAME_STORE_SIZE: usize = 18;
  const VAR_STORE_SIZE: usize = 10;

  let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
  let mut kernel = Kernel::new();



  Ok(())
}
