mod shellmemory;
mod pcb;
mod cpu;

use std::collections::{HashMap, VecDeque};
use std::error::Error;


use crate::cpu::{process_complete, run_cpu};
use crate::pcb::PCB;
use crate::shellmemory::ShellMemory;

struct Kernel {
  all_pcb: HashMap<usize, PCB>,
  process_queue: VecDeque<usize>,
  lru_cache: VecDeque<(usize, usize)>,
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

    self.all_pcb.insert(self.pid_counter, new_pcb);
    self.process_queue.push_back(self.pid_counter);

    for i in 0..page_limit {
      let page_index = self.all_pcb.get(&self.pid_counter).unwrap().page_table[i].page_index;
      self.lru_cache.push_front((self.pid_counter, page_index))
    }

    self.pid_counter += 1;
    Ok(())
  }

  fn run_process(&mut self, shell_memory: &mut ShellMemory, pcb: &mut PCB, page_index: &usize, start_pos: &usize, program_counter: &mut usize, size: &mut usize, valid_bit: &mut usize, cwd: &String) -> Result<(), Box<dyn Error>> {
    if let Err(_) = run_cpu(shell_memory, start_pos, program_counter, size, valid_bit, cwd) {
      if let Err(_) = pcb.load_page(shell_memory, *page_index) {
        let victim_page = self.lru_cache.pop_back().unwrap();

        match self.all_pcb.iter_mut().find(|(pid, _)| **pid != victim_page.0) {
          Some((_, found_pcb)) => {
            found_pcb.evict_page(shell_memory, victim_page.1);
            pcb.load_page(shell_memory, *page_index)?;
          },
          None => return Err("Could not find PCB to be deleted".into())
        }
      }
      self.lru_cache.push_front((pcb.pid, pcb.page_table[*page_index].page_index));
      self.process_queue.push_back(pcb.pid);
      return Err("Page fault and loaded".into())
    }

    match self.lru_cache.binary_search(&(pcb.pid, *page_index)) {
      Ok(i) => {
        let page = self.lru_cache.remove(i).unwrap(); //We can guarantee this does not panic
        self.lru_cache.push_front(page);
        Ok(())
      },
      Err(_) => {
        return Err("Cached Page not found (How did this happen??)".into())
      }
    }
  }

  pub fn run_process_rr(&mut self, shell_memory: &mut ShellMemory, cwd: &String) -> Result<(), Box<dyn Error>> {
    'scheduler: while !self.queue_done() {
      let mut curr_pcb = self.all_pcb.get_mut(&self.process_queue.pop_front().unwrap()).unwrap();







    }
    self.process_queue.clear();
    self.lru_cache.clear();
    self.all_pcb.clear();
    Ok(())
  }

  fn queue_done(&self) -> bool {
    self.process_queue.iter().all(|pid| self.all_pcb.get(pid).unwrap().pcb_complete())
  }
}