mod shellmemory;
mod interpreter;
mod kernel;
mod pcb;
mod cpu;

use std::{env, fs, process};

fn main() -> std::io::Result<()> {
  let args: Vec<String> = env::args().collect();

  if args.len() < 2 {
    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Error: SimpleShell must be called with the hard drive name as an argument."));
  }

  let hard_drive_name: &String = &args[1];
  let mut format: bool = false;

  if args.len() == 3 && args[2] == "-f" {
    format = true;
  }

  fs::remove_dir_all("backing_store")?;
  fs::create_dir("backing_store")?;

  Ok(())
}


pub fn parse_input(arguments: &Vec<String>, cwd: &String) -> String {

}
