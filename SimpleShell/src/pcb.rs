use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::interpreter::parser;
use crate::shellmemory::ShellMemory;
use crate::errors::ShellErrors;
use crate::errors::ShellErrors::PageFault;

#[derive(Debug)]
pub struct PCB {
  pub pid: usize,
  pub program_size: usize,
  pub program_counter: usize,
  pub pages_executed: usize,
  pub frames_executed: usize,
  pub page_table: Vec<PAGE>,
  pub page_table_size: usize,
  source_file: File
}

#[derive(Clone, Debug, Default)]
pub struct PAGE {
  pub page_pid: usize,
  pub page_index: usize,
  pub index: [usize; 3],
  pub valid_bit: [bool; 3]
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
      valid_bit: [false; 3]
    }
  }
}

impl PartialEq for PCB {
  fn eq(&self, other: &Self) -> bool {
    self.pid == other.pid
  }
}

impl PCB {
  pub fn new(shell_memory: &mut ShellMemory, pid: &usize, file_name: &String) -> Result<PCB, Box<dyn Error>> {
    let source_file = File::open(file_name)?;
    let program_size = Self::count_lines(&source_file)?;
    let page_table_size = if program_size % 3 == 0 { program_size / 3 } else { program_size / 3 + 1 };
    let mut page_table: Vec<PAGE> = (0..page_table_size).map(|i| PAGE::new(&pid, &i)).collect();

    let pages_to_load = if program_size < 2 { program_size } else { 2 };
    let mut reader = BufReader::new(&source_file);

    for i in 0..pages_to_load {
      let curr_page = &mut page_table[i];
      if let Err(e) = shell_memory.alloc_frame(pid.to_string(),&mut curr_page.index, &mut curr_page.valid_bit) {
          shell_memory.print_memory();
          println!("{}", e);
          return Err(e.into())
      }

      for j in 0..3 {
        let mut line = String::new();
        if reader.read_line(&mut line)? != 0  {
          shell_memory.set_value_at(curr_page.index[j], pid.to_string(), line.clone(), &mut curr_page.valid_bit[j]);
          line.clear();
        } else {
          break;
        }
      }
    }

    Ok(PCB {
      pid: *pid,
      program_size,
      program_counter: 0,
      pages_executed: 0,
      frames_executed: 0,
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

  pub fn load_page(&mut self, shell_memory: &mut ShellMemory, page_index: usize) -> Result<(), ShellErrors> {
    let curr_page = &mut self.page_table[page_index];
    if let Err(e) = shell_memory.alloc_frame(self.pid.to_string(), &mut curr_page.index, &mut curr_page.valid_bit) {
      shell_memory.print_memory();
      println!("{}", e);
      return Err(ShellErrors::NoFreePages)
    }

    let mut reader = BufReader::new(&self.source_file);
    for i in 0..3 {
      let mut line = String::new();
      if reader.read_line(&mut line)? != 0  {
        shell_memory.set_value_at(curr_page.index[i], self.pid.to_string(), line.clone(), &mut curr_page.valid_bit[i]);
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
      println!("{}", shell_memory.get_value_at(self.page_table[page_index].index[i]).unwrap()); //We can guarantee this will not panic
      shell_memory.free_at(self.page_table[page_index].index[i]);
      self.page_table[page_index].index[i] = 0;
      self.page_table[page_index].valid_bit[i] = false;
    }
    println!("End of victim page contents");
  }

  pub fn run_process(&mut self, shell_memory: &mut ShellMemory, cwd: &String) -> Result<usize, ShellErrors> {
    if self.page_table[self.pages_executed].valid_bit[self.frames_executed] == false {
      return Err(PageFault(self.page_table[self.pages_executed].index[self.frames_executed]));
    }
    parser(shell_memory, &mut shell_memory.get_value_at(self.page_table[self.pages_executed].index[self.frames_executed]).unwrap(), cwd)?;
    self.increment_pc();
    Ok(self.page_table[self.pages_executed].index[self.frames_executed])
  }

  fn increment_pc(&mut self) {
    self.frames_executed += 1;

    if self.frames_executed % 3 == 0 {
      self.frames_executed = 0;
      self.pages_executed += 1;
    }

    self.program_size -= 1;
    self.program_counter += 1;
  }

  pub fn pcb_complete(&self) -> bool { self.program_size == 0 }
}

#[cfg(test)]
mod pcb_tests {
  use super::*;

  #[test]
  fn test_create() {
    todo!()
  }
}