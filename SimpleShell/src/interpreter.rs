use crate::errors::ShellErrors;
use crate::shellmemory::ShellMemory;

pub fn parser(shell_memory: &mut ShellMemory, user_input: &mut String, cwd: &String) -> Result<(), ShellErrors> {
  let tokens: Vec<&str> = user_input.split(';').collect();

  for token in tokens.iter() {
    let token = token.trim();
    let arguments: Vec<String> = token.split_whitespace().map(|s|s.to_string()).collect();
    let num_of_args = arguments.len();
    interpreter(shell_memory, &arguments, &num_of_args, cwd)?
  }
  Ok(())
}

pub fn interpreter(shell_memory: &mut ShellMemory, arguments: &Vec<String>, num_of_args: &usize, cwd: &String) -> Result<(), ShellErrors> {
  if *num_of_args < 1 {
    println!("Error: interpreter must be called with at least one argument.");
  }

  match arguments.first().unwrap().as_str() {
    "help" => {
      println!("Help!");
      Ok(())
    },
    "quit" => {
      std::process::exit(0);
    },
    "set" => {
      if *num_of_args < 3 {
        println!("Error: set command must be called with at least three arguments");
      }

      let key: String = arguments[1].clone();
      let value: String = arguments[2..].join(" ");

      shell_memory.set_var(&key, &value);
      Ok(())
    },
    "print" => {
      if *num_of_args != 2 {
        println!("Error: print command must be called with at least two arguments")
      }

      println!("{}", arguments[1..].join(" "));
      Ok(())
    },
    "echo" => {
      match arguments[1].as_str() {
        "$" => {
          println!("{}", shell_memory.get_var_by_key(&arguments[2]).unwrap_or(" ".to_string()));
          Ok(())
        },
        _ => {
          println!("{}", arguments[1..].join(" "));
          Ok(())
        }
      }
    },
    "resetvars" => {
      shell_memory.clear_variables();
      Ok(())
    }
    "run" => {
      todo!();
    },
    "exec" => {
      todo!();
    },
    _ => {
      Ok(())
    }
  }
}

#[cfg(test)]
mod interpreter_tests {
  use super::*;
  pub const FRAME_STORE_SIZE: usize = 6;
  pub const VAR_STORE_SIZE: usize =  4;
  pub const TOTAL_SIZE: usize = FRAME_STORE_SIZE + VAR_STORE_SIZE;
}
