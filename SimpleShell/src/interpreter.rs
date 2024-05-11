use crate::shellmemory::ShellMemory;

pub fn interpreter(shell_memory: &mut ShellMemory, arguments: &Vec<String>, num_of_args: &usize, cwd: &String) {
  if *num_of_args < 1 {
    println!("Error: interpreter must be called with at least one argument.");
  }

  match arguments.first().unwrap().as_str() {
    "help" => {
      todo!();
      Ok(Some("help".to_string()))
    },
    "quit" => {
      todo!();
      Ok(Some("Quitting Simple Shell".to_string()))
    },
    "set" => {
      if *num_of_args < 3 {
        println!("Error: set command must be called with at least three arguments");
      }

      let key: String = arguments[1].clone();
      let value: String = arguments[2..].join(" ");

      shell_memory.set_value(&key, &value)?;
    },
    "print" => {
      if *num_of_args != 2 {
        println!("Error: print command must be called with two arguments")
      }

      println!("{}", arguments[1]);
    },
    "echo" => {
      if *num_of_args > 2 {
        println!("Error: echo command must be called with at least two arguments")
      }

      if arguments[1].starts_with('$') {
        let key: String = arguments[1].chars().skip(1).collect();

        match shell_memory.get_value(&key) {
          Ok(value) => {
            println!("{}", value);
          },
          Err(e) => println!("{}", e)
        }
      } else {
        println!("{}", arguments[1]);
      }
    },
    "resetvars" => {
      shell_memory.clear_variables();
    }
    "run" => {
      todo!();
      if *num_of_args != 2 {
        println!("Error: run command must be called with at least 2 arguments")
      }

    },
    "exec" => {
      todo!();
    },
    "ls" => {
      todo!();

    },
    "cat" => {
      todo!();
    },
    "rm" => {
      todo!();
    },
    "create" => {
      todo!();
    },
    "write" => {
      todo!();
    },
    "find_file" => {
      todo!();
    },
    "read" => {
      todo!();
    },
    "copy_in" => {
      todo!();
    },
    "copy_out" => {
      todo!();
    },
    "size" => {
      todo!();
    },
    "seek" => {
      todo!();
    },
    "freespace" => {
      todo!();
    },
    "frag_degree" => {
      todo!();
    },
    "defragment" => {
      todo!();
    },
    "recover" => {
      todo!();
    },
    _ => {
      println!("Invalid command")
    }
  }
}