use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use crate::errors::ShellErrors::PageFault;

use crate::pcb::PCB;
use crate::shellmemory::ShellMemory;

struct Kernel {
  all_pcb: HashMap<usize, RefCell<PCB>>, //PID -> PCB
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

  pub fn add_new_process(&mut self, shell_memory: &mut ShellMemory, script_source: &String) -> Result<(), Box<dyn Error>> {
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

  fn run_process(&mut self, shell_memory: &mut ShellMemory, pid: &usize, cwd: &String) -> Result<(), Box<dyn Error>> {
    let mut pcb = self.all_pcb.get(pid).unwrap().borrow_mut();

    match pcb.run_process(shell_memory, cwd) {
      Ok(page_index) => {
       let index = self.lru_cache.iter().enumerate().find(|(i,(_, page_id))|{
         *page_id == page_index
       })
         .map(|(i,_)| i);
        self.lru_cache.remove(index.unwrap());
        self.lru_cache.push_front((*pid, page_index))
      },
      Err(e) => { //If we page fault
        match e {
          PageFault(page_index) => {
            if let Err(_) = pcb.load_page(shell_memory, page_index) {
              let victim_page = self.lru_cache.pop_back().unwrap(); //Get LRU page
              self.all_pcb.get(&victim_page.0).unwrap().borrow_mut().evict_page(shell_memory, victim_page.1); //Evict that page
              pcb.load_page(shell_memory, victim_page.1)? //Load page @ evicted page location
            }
            self.lru_cache.push_front((*pid, page_index)); //Place page into LRU
            self.process_queue.push_back(*pid); //Place process back of queue
          },
          _ => return Err(Box::try_from(e).unwrap())
        }
      }
    }
    Ok(())
  }

  pub fn run_process_fifo(&mut self, shell_memory: &mut ShellMemory, cwd: &String) -> Result<(), Box<dyn Error>> {
    while !self.queue_done(){
      let pid = self.process_queue.pop_front().unwrap();
      self.run_process(shell_memory, &pid, cwd).expect("TODO: panic message");
    }
    Ok(())
  }

  fn queue_done(&self) -> bool {
    self.process_queue.iter().all(|pid| self.all_pcb.get(pid).unwrap().borrow().pcb_complete())
  }
}