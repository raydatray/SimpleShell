use std::error::Error;
use crate::shellmemory::ShellMemory;

pub fn run_cpu(shell_memory: &mut ShellMemory, start_pos: &usize, program_counter: &mut usize, size: &mut usize, valid_bit: &usize, cwd: &String) -> Result<(), Box<dyn Error>> {
  if *valid_bit == 0 {
    return Err("Page fault".into());
  }
  //parse_input(shell_memory.get_value_at(*start_pos).unwrap().split_whitespace().collect(), cwd);
  *program_counter += 1;
  *size -= 1;
  Ok(())
}

pub fn process_complete(size: usize) -> bool {
  return size == 0
}