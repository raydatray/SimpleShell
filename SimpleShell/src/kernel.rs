use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::error::Error;

use crate::errors::ShellErrors;
use crate::errors::ShellErrors::{CacheError, PageFault, NoFreePages};

use crate::pcb::PCB;
use crate::shellmemory::ShellMemory;

pub struct Kernel {
  all_pcb: HashMap<usize, RefCell<PCB>>, //The hashmap owns the PCBs in RefCells (for interior mutability) me > borrow checker
  process_queue: VecDeque<usize>, //PIDs
  lru_cache: VecDeque<(usize, usize)>, //PID, Page_index
  pid_counter: usize
}

impl Kernel {
  pub fn new() -> Kernel {
    Kernel {
      all_pcb: HashMap::new(),
      process_queue: VecDeque::new(),
      lru_cache: VecDeque::new(),
      pid_counter: 1,
    }
  }

  pub fn add_new_process(&mut self, shell_memory: &mut ShellMemory, script_source: &String) -> Result<(), ShellErrors> {
    let new_pcb = PCB::new(shell_memory, &self.pid_counter, script_source)?;
    let page_limit =  if new_pcb.page_table_size < 2 { new_pcb.page_table_size } else { 2 };

    self.all_pcb.insert(self.pid_counter, RefCell::new(new_pcb));
    self.process_queue.push_back(self.pid_counter);

    for i in 0..page_limit {
      let page_index = self.all_pcb.get(&self.pid_counter).unwrap().borrow().page_table[i].page_index;
      self.lru_cache.push_front((self.pid_counter, page_index))
    }

    self.pid_counter += 1;
    Ok(())
  }

  fn run_process(&mut self, shell_memory: &mut ShellMemory, pid: &usize, cwd: &String) -> Result<(), ShellErrors> {
    let mut pcb = self.all_pcb.get(pid).unwrap().borrow_mut();

    match pcb.run_process(shell_memory, cwd) {
      Ok(page_index) => {
        let target_entry = (*pid, page_index);

        return match self.lru_cache.iter().position(|lru_entry|  { *lru_entry ==  target_entry }) {
          Some(i)  => {
            let lru_entry = self.lru_cache.remove(i).unwrap();
            self.lru_cache.push_front(lru_entry);
            Ok(())
          },
          None => {
            Err(CacheError) //This should never happen
          }
        }
      },
      Err(e) => { //If we page fault
        match e {
          PageFault(page_index) => {
            if let Err(NoFreePages) = pcb.load_page(shell_memory, page_index) { //If we need  to evict LRU
              let victim_page = self.lru_cache.pop_back().unwrap(); //Get LRU page

              if victim_page.0 == *pid {
                pcb.evict_page(shell_memory, victim_page.1);
              } else {
                self.all_pcb.get(&victim_page.0).unwrap().borrow_mut().evict_page(shell_memory, victim_page.1); //Evict that page
              }

              pcb.load_page(shell_memory, page_index).unwrap(); //Load page @ evicted page location
              return Err(e)
            }
            self.lru_cache.push_front((*pid, page_index)); //Place page into LRU
            self.process_queue.push_back(*pid); //Place process back of queue
            Err(e)
          },
          _ => Err(*Box::try_from(e).unwrap())
        }
      }
    }
  }

  pub fn run_process_fifo(&mut self, shell_memory: &mut ShellMemory, cwd: &String) -> Result<(), ShellErrors> {
    while !self.queue_done(){
      let pid = self.process_queue.pop_front().unwrap();
      while !self.all_pcb.get(&pid).unwrap().borrow().pcb_complete() {
        //Any type of error other than PageFault should be propagated
        match self.run_process(shell_memory, &pid, cwd) {
          Ok(()) | Err(PageFault(_)) => {
            continue;
          },
          Err(e) => {
            return Err(e);
          }
        }
      }
    }
    Ok(())
  }

  fn queue_done(&self) -> bool {
    self.process_queue.iter().all(|pid| self.all_pcb.get(pid).unwrap().borrow().pcb_complete())
  }
}

#[cfg(test)]
mod kernel_tests {
  use super::*;
  pub const FRAME_STORE_SIZE: usize = 18; //3 files, 2 pages, 3 line each = 18 frame store at a minimum
  pub const VAR_STORE_SIZE: usize =  4;
  pub const TOTAL_SIZE: usize = FRAME_STORE_SIZE + VAR_STORE_SIZE;
  pub const TEST_FILE_1: &str = "examples/test1.txt";
  pub const TEST_FILE_2: &str = "examples/test2.txt";
  pub const TEST_FILE_3: &str = "examples/test3.txt";

  #[test]
  fn test_create_and_add() {
    let script_paths = vec![TEST_FILE_1, TEST_FILE_2, TEST_FILE_3];
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    let mut kernel = Kernel::new();

    for (i, script_path) in script_paths.iter().enumerate() {
      let result = kernel.add_new_process(&mut shell_memory, &script_path.to_string());
      assert!(result.is_ok());
      assert_eq!(kernel.all_pcb.len(), i + 1);
      assert!(kernel.all_pcb.contains_key(&(i + 1)));
      assert_eq!(*kernel.process_queue.back().unwrap(), i + 1);

      //We expect the LRU to be added contiguously (3, 2, 1)
      //We expect 2, 1, 2 pages to be allocated in form pid, page#
      match i {
        0 | 2 => {
          let _ = (1usize..0usize).for_each(|j| {
            let lru = kernel.lru_cache.front();
            assert!(lru.is_some());

            let expected_lru = ((i + 1), j);
            assert_eq!(lru, Some(&expected_lru));
          });
        },
        1 => {
          let lru = kernel.lru_cache.front();
          assert!(lru.is_some());

          let expected_lru = ((i + 1), 0usize);
          assert_eq!(lru, Some(&expected_lru));
        },
        _ => {
          panic!("Out of bounds");
        }
      }
    }
  }

  #[test]
  fn test_run_process_success() {
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    let mut kernel = Kernel::new();

    let dummy_cwd = "dummyCwd".to_string();

    //We add one process and run it
    let result = kernel.add_new_process(&mut shell_memory, &TEST_FILE_1.to_string());
    assert!(result.is_ok());

    //We have 6 lines,  so we run the process 6 times
    //We expect no loading, nor page faults
    for i in 0usize..6 {
      let result = kernel.run_process(&mut shell_memory, &1, &dummy_cwd);
      assert!(result.is_ok());

      //Each time we expect LRU cache to be updated
      //Initial state: 1,1 - 1,0
      match i {
        0..=2 => {
          //We are in page 0 here, we expect (1,0) to be at the front of LRU
          let expected_lru = (1usize, 0usize);
          let lru = kernel.lru_cache.front();
          assert!(lru.is_some());
          assert_eq!(lru, Some(&expected_lru));
        },
        3..=5 => {
          //We are in page 1 here, we expect (1,1) to be at the front of LRU here
          let expected_lru = (1usize, 1usize);
          let lru = kernel.lru_cache.front();
          assert!(lru.is_some());
          assert_eq!(lru, Some(&expected_lru));
        }
        _ => {
          panic!("Out of bounds");
        }
      }
    }
  }
  #[test]
  fn test_run_process_page_fault() {
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    let mut kernel = Kernel::new();

    let dummy_cwd = "dummyCwd".to_string();

    //We add one process and run it
    let result = kernel.add_new_process(&mut shell_memory, &TEST_FILE_3.to_string());
    assert!(result.is_ok());

    //We manually remove the pid from the deque to assert proper behavior
    kernel.process_queue.pop_front();

    //This process is 8 lines long, so we can expect a page fault and a re-load
    //On i = 6 we will fault. We do 9 iterations due to 1 skipped
    for i in 0usize..9 {
      let result = kernel.run_process(&mut shell_memory, &1, &dummy_cwd);
      match i {
        0..=5 | 7..=8 => {
          assert!(result.is_ok());

          let expected_lru = (1usize, i / 3usize);
          let lru = kernel.lru_cache.front();

          assert!(lru.is_some());
          assert_eq!(lru, Some(&expected_lru));
        },
        6 => {
          assert!(result.is_err());
          assert_eq!(result.unwrap_err(), PageFault(2));

          //After a page fault, we expect the pid to be readded to the  back  of the process q
          assert_eq!(kernel.process_queue.len(), 1);
          assert_eq!(kernel.process_queue[0], 1);

          let expected_lru = (1usize, 2usize);
          let lru = kernel.lru_cache.front();

          assert!(lru.is_some());
          assert_eq!(lru, Some(&expected_lru));
        },
        _ => {
          panic!("Out of bounds");
        }
      }
    }
  }
  #[test]
  fn test_cache_replacement() {
    //We deliberately reduce to 2 pages so that we force a cache replacement
    let mut shell_memory = ShellMemory::new(6, VAR_STORE_SIZE);
    let mut kernel = Kernel::new();

    let dummy_cwd = "dummyCwd".to_string();

    //We add one process and run it
    let result = kernel.add_new_process(&mut shell_memory, &TEST_FILE_3.to_string());
    assert!(result.is_ok());

    //We expect LRU to be (1,1), (1,0) in its init state
    //1,0 - 1,1 -> 1,1 - 1,0. We evict 1,0 and expect 1,2 - 1,1

    for i in 0usize..9 {
      let result = kernel.run_process(&mut shell_memory, &1, &dummy_cwd);
      match i {
        0..=5  => {
          assert!(result.is_ok());

          let expected_lru = (1usize, i / 3usize);
          let lru = kernel.lru_cache.front();

          assert!(lru.is_some());
          assert_eq!(lru, Some(&expected_lru));
        },
        6 => {
          assert!(result.is_err());
          assert_eq!(result.unwrap_err(), PageFault(2));


        }
        7..=8 => {

        },
        _ => {
          panic!("Out of bounds");
        }
      }
    }
  }

  //Check the output on this test manually (we can't capture stdout yet)
  #[test]
  fn test_fifo() {
    let script_paths = vec![TEST_FILE_1, TEST_FILE_2, TEST_FILE_3];
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    let mut kernel = Kernel::new();

    let dummy_cwd = "dummyCwd".to_string();

    script_paths.iter().for_each(|script_path| {
      kernel.add_new_process(&mut shell_memory, &script_path.to_string()).unwrap()
    });

    let result = kernel.run_process_fifo(&mut shell_memory, &dummy_cwd);
  }
}