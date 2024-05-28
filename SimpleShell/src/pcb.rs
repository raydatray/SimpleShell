use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek};

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
  fn eq(&self, other: &Self) -> bool { self.pid == other.pid }
}

impl PCB {
  pub fn new(shell_memory: &mut ShellMemory, pid: &usize, file_name: &String) -> Result<PCB, ShellErrors> {
    let source_file = File::open(file_name)?;
    let program_size = Self::count_lines(&source_file)?;
    let page_table_size = if program_size % 3 == 0 { program_size / 3 } else { program_size / 3 + 1 };
    let mut page_table: Vec<PAGE> = (0..page_table_size).map(|i| PAGE::new(&pid, &i)).collect();

    let pages_to_load = if page_table_size < 2 { page_table_size } else { 2 };
    let mut reader = BufReader::new(&source_file);
    reader.rewind()?;

    for i in 0..pages_to_load {
      let curr_page = &mut page_table[i];
      if let Err(e) = shell_memory.alloc_frame(&pid.to_string(), &mut curr_page.index, &mut curr_page.valid_bit) {
          shell_memory.print_memory();
          println!("{}", e);
          return Err(e.into())
      }

      for j in 0..3 {
        let mut line = String::new();
        if reader.read_line(&mut line)? != 0  {
          shell_memory.set_value_at(curr_page.index[j], &pid.to_string(), &line.clone(), &mut curr_page.valid_bit[j]);
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
    let count= lines.try_fold(0, |acc, line| line.map(|_| acc + 1))?;
    Ok(count)
  }

  pub fn load_page(&mut self, shell_memory: &mut ShellMemory, page_index: usize) -> Result<(), ShellErrors> {
    let curr_page = &mut self.page_table[page_index];
    if let Err(e) = shell_memory.alloc_frame(&self.pid.to_string(), &mut curr_page.index, &mut curr_page.valid_bit) {
      shell_memory.print_memory();
      println!("{}", e);
      return Err(ShellErrors::NoFreePages)
    }

    let mut reader = BufReader::new(&self.source_file);
    for i in 0..3 {
      let mut line = String::new();
      if reader.read_line(&mut line)? != 0  {
        shell_memory.set_value_at(curr_page.index[i], &self.pid.to_string(), &line.clone(), &mut curr_page.valid_bit[i]);
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
  use crate::errors::ShellErrors::InitialFrameAllocationFailed;
  use super::*;
  pub const FRAME_STORE_SIZE: usize = 12;
  pub const VAR_STORE_SIZE: usize =  4;
  pub const TOTAL_SIZE: usize = FRAME_STORE_SIZE + VAR_STORE_SIZE;

  #[test]
  fn test_create_page() {
    let page_pid = 0usize;
    let page_index = 0usize;

    let page = PAGE::new(&page_pid, &page_index);

    assert_eq!(page.page_pid, page_pid);
    assert_eq!(page.page_index, page_index);

    for i in 0usize..3usize {
      assert_eq!(page.index[i], 1000);
      assert_eq!(page.valid_bit[i], false)
    }
  }

  #[test]
  fn test_eq_page() {
    let page_pid1 = 0usize;
    let page_pid2 = 1usize;
    let page_pid3 = 2usize;

    let page_index1 = 0usize;
    let page_index2 = 1usize;
    let page_index3 = 2usize;

    let page1 = PAGE::new(&page_pid1, &page_index1);
    let page2 = PAGE::new(&page_pid2, &page_index2);
    let mut page3 = PAGE::new(&page_pid3, &page_index3);

    page3.index[0] = 999usize;

    assert_eq!(page1, page2);
    assert_ne!(page3, page1);
    assert_ne!(page3, page2);
  }


  //Case 1: A perfect allocation of exactly 2 pages (6 lines)
  #[test]
  fn test_create_pcb_1() {
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    let pid = 0usize;
    let test_file_path = "examples/test1.txt".to_string();

    let pcb = PCB::new(&mut shell_memory, &pid, &test_file_path);
    assert!(pcb.is_ok());

    let created_pcb = pcb.unwrap();

    assert_eq!(created_pcb.pid, pid);
    assert_eq!(created_pcb.program_size, 6);
    assert_eq!(created_pcb.program_counter, 0);
    assert_eq!(created_pcb.pages_executed, 0);
    assert_eq!(created_pcb.frames_executed, 0);
    assert_eq!(created_pcb.page_table_size, 2);
    assert_eq!(created_pcb.page_table.len(), 2);

    //We expect every frame of every page to be allocated and set to true
    for (i, page) in created_pcb.page_table.iter().enumerate() {
      for (j, frame) in page.index.iter().enumerate()  {
        let frame_line = shell_memory.get_value_at(*frame);
        let expected_line = format!("line{}\r\n", (((i * 3) + j) + 1));

        assert_eq!(frame_line, Some(expected_line));
        assert_eq!(page.valid_bit[j], true);
      }
    }
  }


  //Case 2: A less than 2 pages initial allocation (2 lines)
  #[test]
  fn test_create_pcb_2() {
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    let pid = 0usize;
    let test_file_path = "examples/test2.txt".to_string();

    let pcb = PCB::new(&mut shell_memory, &pid, &test_file_path);
    assert!(pcb.is_ok());

    let created_pcb = pcb.unwrap();

    assert_eq!(created_pcb.pid, pid);
    assert_eq!(created_pcb.program_size, 2);
    assert_eq!(created_pcb.program_counter, 0);
    assert_eq!(created_pcb.pages_executed, 0);
    assert_eq!(created_pcb.frames_executed, 0);
    assert_eq!(created_pcb.page_table_size, 1);
    assert_eq!(created_pcb.page_table.len(), 1);

    //We expect the last frame of the page to unallocated and false
    for j in created_pcb.page_table[0].index.iter() {
      let frame_line = shell_memory.get_value_at(*j);
      match j {
        0 | 1 => {
          let expected_line = format!("line{}\r\n", (j + 1));
          assert_eq!(frame_line, Some(expected_line));
          assert_eq!(created_pcb.page_table[0].valid_bit[*j], true)
        },
        2 => {
          assert_eq!(frame_line, None);
          assert_eq!(created_pcb.page_table[0].valid_bit[*j], false)
        }
        _ => panic!("Out of bounds")
      }
    }
  }

  //Case 3: A more than 2 page initial allocation
  #[test]
  fn test_create_pcb_3() {
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    let pid = 0usize;
    let test_file_path = "examples/test3.txt".to_string();

    let pcb = PCB::new(&mut shell_memory, &pid, &test_file_path);
    assert!(pcb.is_ok());

    let created_pcb = pcb.unwrap();

    assert_eq!(created_pcb.pid, pid);
    assert_eq!(created_pcb.program_size, 8);
    assert_eq!(created_pcb.program_counter, 0);
    assert_eq!(created_pcb.pages_executed, 0);
    assert_eq!(created_pcb.frames_executed, 0);
    assert_eq!(created_pcb.page_table_size, 3);
    assert_eq!(created_pcb.page_table.len(), 3);


    for (i, page) in created_pcb.page_table.iter().enumerate() {
      for (j, frame) in page.index.iter().enumerate() {
        match i {
          0..=1 => {
            let frame_line = shell_memory.get_value_at(*frame);
            let expected_line = format!("line{}\r\n", (((i * 3) + j) + 1));
            assert_eq!(frame_line, Some(expected_line));
            assert_eq!(page.valid_bit[j], true);
          },
          2 => {
            assert_eq!(*frame, 1000usize);
            assert_eq!(page.valid_bit[j], false);
          }
          _ => panic!("Out of bounds")
        }
      }
    }
  }

  //We test the fail case due to being unable to allocate inital frames
  #[test]
  fn test_fail_create_pcb() {
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    let dummy_pid = "dummyPid".to_string();
    let dummy_value = "dummyValue".to_string();

    //We fill the frame store such that it is impossible to allocate the initial frames
    //(We don't actually care about these values)
    for i in 0usize..9usize {
      shell_memory.set_value_at(i, &dummy_pid, &dummy_value, &mut false)
    }

    let pid = 0usize;
    let test_file_path = "examples/test1.txt".to_string();

    let pcb = PCB::new(&mut shell_memory, &pid, &test_file_path);
    assert!(pcb.is_err());
    assert_eq!(pcb.unwrap_err(), InitialFrameAllocationFailed)
  }

  #[test]
  fn test_count_lines() {
    let test_file_path = "examples/test1.txt".to_string();
    let test_file = File::open(test_file_path).unwrap();

    let count = PCB::count_lines(&test_file);
    assert!(count.is_ok());
    assert_eq!(count.unwrap(), 6);
  }

  #[test]
  fn test_load_pages_success() {
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    let pid = 0usize;
    let test_file_path = "examples/test3.txt".to_string();

    let pcb = PCB::new(&mut shell_memory, &pid, &test_file_path);
    assert!(pcb.is_ok());

    let mut created_pcb = pcb.unwrap();
    //Same as created_3 up to this point.
    //We "fail" @index 2 of the pagetable and need to load in a new page

    let result = created_pcb.load_page(&mut shell_memory, 2);
    assert!(result.is_ok());

    //We expect the new pages to be loaded in properly



  }

  #[test]
  fn test_load_pages_fail() {

  }

  #[test]
  fn test_evict_page() {

  }

  #[test]
  fn test_run_process() {

  }

  #[test]
  fn test_increment_pc() {

  }
}