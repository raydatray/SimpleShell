mod shellmemory;
mod pcb;

use std::error::Error;
use crate::pcb::PCB;
fn run_cpu(start_pos: &usize, program_counter: &usize, size: &usize, valid_bit: &usize, cwd: &String) -> Result<(), Box<dyn Error>> {
  if *valid_bit == 0 {
    return Err("Page fault".into());
  }


}