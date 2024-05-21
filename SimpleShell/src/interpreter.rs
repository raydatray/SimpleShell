use std::error::Error;
use crate::errors::ShellErrors;
use crate::shellmemory::ShellMemory;

pub fn parser(shell_memory: &mut ShellMemory, user_input: &mut String, cwd: &String) -> Result<(), ShellErrors> {
  let tokens: Vec<&str>= user_input.split(';').collect();


  for token in tokens.iter() {
    let token = token.trim();
    let arguments: Vec<_> = token.split_whitespace().map(|s|s.to_string()).collect();
    let num_of_args = arguments.len();

    if let Err(e) = interpreter(shell_memory, &arguments, &num_of_args, cwd) {
      return Err(e)
    }
  }

  Ok(())
}


pub fn interpreter(shell_memory: &mut ShellMemory, arguments: &Vec<String>, num_of_args: &usize, cwd: &String) -> Result<(), ShellErrors> {
  if *num_of_args < 1 {
    println!("Error: interpreter must be called with at least one argument.");
  }

  match arguments.first().unwrap().as_str() {
    "help" => {
      todo!();
    },
    "quit" => {
      todo!();
    },
    "set" => {
      if *num_of_args < 3 {
        println!("Error: set command must be called with at least three arguments");
      }

      let key: String = arguments[1].clone();
      let value: String = arguments[2..].join(" ");

      shell_memory.set_value(&key, &value);
      Ok(())
    },
    "print" => {
      if *num_of_args != 2 {
        println!("Error: print command must be called with two arguments")
      }

      println!("{}", arguments[1]);
      Ok(())
    },
    "echo" => {
      todo!()
    },
    "resetvars" => {
      todo!()
    }
    "run" => {
      todo!();

    },
    "exec" => {
      todo!();
    },
    _ => {
      println!("Invalid command");
      Ok(())
    }
  }
}