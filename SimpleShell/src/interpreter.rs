use crate::errors::ShellErrors;
use crate::kernel::Kernel;
use crate::shellmemory::ShellMemory;

pub fn parser(mut kernel: Option<&mut Kernel>, shell_memory: &mut ShellMemory, user_input: &mut String, cwd: &String) -> Result<(), ShellErrors> {
  let tokens: Vec<&str> = user_input.split(';').collect();

  for token in tokens.iter() {
    let token = token.trim();
    let arguments: Vec<String> = token.split_whitespace().map(|s|s.to_string()).collect();
    let num_of_args = arguments.len();

    match kernel.as_deref_mut() {
      Some(x) => {
        top_level_interpreter(x, shell_memory, &arguments, num_of_args, cwd)?
      },
      None => {
        interpreter(shell_memory, &arguments, num_of_args, cwd)?
      }
    }
  }
  Ok(())
}

pub fn top_level_interpreter(kernel: &mut Kernel, shell_memory: &mut ShellMemory, arguments: &Vec<String>, num_of_args: usize, cwd: &String) -> Result<(), ShellErrors> {
  if num_of_args < 1 {
    println!("Error: interpreter must be called with at least one argument");
  }

  match arguments.first().unwrap().as_str() {
    "run" => {
      if num_of_args != 2 {
        println!("Error: run must be called with two arguments");
      }
      kernel.add_new_process(shell_memory, &arguments[1])?;
      kernel.run_process_fifo(shell_memory, cwd)?;
      Ok(())
    },
    "exec" => {
      if num_of_args < 2 {
        println!("Error: exec must be called with at least two arguments");
      }
      for script_source in arguments[1..].iter() {
        kernel.add_new_process(shell_memory, script_source)?;
      }
      kernel.run_process_fifo(shell_memory, cwd)?;
      Ok(())
    },
    _ => {
      return interpreter(shell_memory, arguments, num_of_args, cwd);
    }
  }
}

pub fn interpreter(shell_memory: &mut ShellMemory, arguments: &Vec<String>, num_of_args: usize, _cwd: &String) -> Result<(), ShellErrors> {
  match arguments.first().unwrap().as_str() {
    "help" => {
      println!("Help!");
      Ok(())
    },
    "quit" => {
      std::process::exit(0);
    },
    "set" => {
      if num_of_args < 3 {
        println!("Error: set command must be called with at least three arguments");
      }

      let key: String = arguments[1].clone();
      let value: String = arguments[2..].join(" ");

      shell_memory.set_var(&key, &value);
      Ok(())
    },
    "print" => {
      if num_of_args < 2 {
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
    },
    _ => {
      Ok(())
    }
  }
}
