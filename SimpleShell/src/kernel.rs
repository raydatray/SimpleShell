mod shellmemory;
mod pcb;
use crate::pcb::PCB;
struct Kernel {
  process_queue: Vec<PCB>,
  lru_cache: Vec<PCB>,
  all_pcb: Vec<PCB>,
  pid_counter: usize
}

impl Kernel {
  fn new() -> Kernel {
    Kernel {
      process_queue: Vec::new(),
      lru_cache: Vec::new(),
      all_pcb: Vec::new(),
      pid_counter: 1
    }
  }

  fn run_process() -> Result<(), Box<dyn Err>> {

  }
}