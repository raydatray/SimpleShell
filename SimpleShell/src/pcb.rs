mod shellmemory;

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::shellmemory::ShellMemory;

#[derive(Debug)]
pub struct PCB {
  pid: usize,
  program_size: usize,
  program_counter: Vec<usize>,
  page_table: Vec<PAGE>,
  page_table_size: usize,
  source_file: File
}

#[derive(Debug, Default)]
struct PAGE {
  page_pid: usize,
  page_index: usize,
  index: [usize; 3],
  valid_bit: [usize; 3]
}

impl PartialEq for PAGE {
  fn eq(&self, other: &Self) -> bool {
    self.index[0] == other.index[0]
  }
}

impl PAGE {
  fn new(page_pid: &usize, page_table_index: &usize) -> PAGE {
    PAGE {
      page_pid: *page_pid,
      page_index: *page_table_index,
      index: [1000; 3],
      valid_bit: [0; 3]
    }
  }
}

impl PartialEq for PCB {
  fn eq(&self, other: &Self) -> bool {
    self.pid == other.pid
  }
}

impl PCB {
  pub fn new(shell_memory: &mut ShellMemory, pid: &usize, file_name: String) -> Result<PCB, Box<dyn Error>> {
    let source_file = File::open(file_name)?;
    let program_size = Self::count_lines(&source_file)?;
    let page_table_size = if program_size % 3 == 0 { program_size / 3 } else { program_size / 3 + 1 };
    let mut page_table: Vec<PAGE> = (0..page_table_size).map(|i| PAGE::new(&pid, &i)).collect();
    let program_counter = vec![0; program_size];

    let pages_to_load = if program_size < 2 { program_size } else { 2 };
    let mut reader = BufReader::new(&source_file);

    for i in 0..pages_to_load {
      let curr_page = &mut page_table[i];
      if let Err(e) = shell_memory.alloc_frame(pid.to_string(), &mut curr_page.index, &mut curr_page.valid_bit) {
        shell_memory.print_memory();
        println!("{}", e);
        return Err(e.into())
      }

      for j in 0..3 {
        let mut line = String::new();
        if reader.read_line(&mut line)? != 0  {
          shell_memory.set_value_at(curr_page.index[j], pid.to_string(), line, &mut curr_page.valid_bit[j]);
          line.clear();
        } else {
          break;
        }
      }
    }

    Ok(PCB {
      pid: *pid,
      program_size,
      program_counter,
      page_table,
      page_table_size,
      source_file
    })
  }

  fn count_lines(file: &File) -> Result<usize, std::io::Error> {
    let mut lines = BufReader::new(file).lines();
    let count: usize = lines.try_fold(0, |acc, line| line.map(|_| acc + 1))?;
    Ok(count)
  }

  pub fn load_page(&mut self, shell_memory: &mut ShellMemory, page_index: usize) -> Result<(), Box<dyn Error>> {
    let curr_page= &mut self.page_table[page_index];

    if let Err(e) = shell_memory.alloc_frame(self.pid.to_string(), &mut curr_page.index, &mut curr_page.valid_bit) {
      shell_memory.print_memory();
      println!("{}", e);
      return Err(e.into())
    }

    let mut reader = BufReader::new(&self.source_file);
    for i in 0..3 {
      let mut line = String::new();

      if reader.read_line(&mut line)? != 0  {
        shell_memory.set_value_at(self.page_table[page_index].index[i], self.pid.to_string(), line, &mut self.page_table[page_index].valid_bit[i]);
        line.clear();
      } else {
        break;
      }
    }
    Ok(())
  }

  pub fn evict_page(&mut self, shell_memory: &mut ShellMemory, page_index: usize) {
    println!("Page fault! Victim page contents");
    for i in 0..3 {
      println!("{}", shell_memory.get_value_at(self.page_table[page_index].index[i as usize]).unwrap()); //We can guarantee this will not panic
      shell_memory.free_at(self.page_table[page_index].index[i as usize]);
      self.page_table[page_index].index[i] = 0;
      self.page_table[page_index].valid_bit[i] = 0;
    }
    println!("End of victim page contents");
  }

  pub fn pcb_complete(&self) -> bool {
    self.program_counter.iter().all(|pc| *pc == 3)
  }
}

#[cfg(test)]
mod pcb_tests {
  use super::*;

  #[test]
  fn test_create() {
    todo!()
  }
}