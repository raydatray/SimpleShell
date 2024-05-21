use crate::errors::ShellErrors;
use crate::errors::ShellErrors::*;
#[derive(Clone, Debug)]
struct MemoryStruct {
  key: Option<String>,
  value: Option<String>
}

#[derive(Clone, Debug)]
pub struct ShellMemory {
  memory: Vec<MemoryStruct>,
  frame_store_size: usize,
  var_store_size: usize
}

impl MemoryStruct {
  fn new() -> MemoryStruct {
    MemoryStruct {
      key: None,
      value: None
    }
  }
}

impl ShellMemory {
  fn new(frame_store_size: usize, var_store_size: usize) -> ShellMemory {
    ShellMemory {
      memory: vec![MemoryStruct::new(); frame_store_size + var_store_size],
      frame_store_size,
      var_store_size
    }
  }

  pub fn free_at(&mut self, index: usize)  {
    self.memory[index].key = None;
    self.memory[index].value = None;
  }

  pub fn alloc_frame(&mut self, pid: String, index: &mut [usize; 3], valid_bit: &mut [bool; 3]) -> Result<(), ShellErrors> {
    for (i, mem) in self.memory[..self.frame_store_size].iter().enumerate() {
      let mut j = i;
      while j < i + 3 && j < self.frame_store_size {
        if let None = &mem.value {
          break
        }
        j += 1;
      }

      if j == i + 3 {
        for k in i..i + 3 {
          self.memory[k].key = Some(pid.clone());
          index[k - i] = k;
          valid_bit[k - i] = false;
        }
        return Ok(())
      }
    }
    Err(InitialFrameAllocationFailed)
  }

  pub fn set_value_at(&mut self, index: usize, pid: String, value: String, valid_bit: &mut bool) {
    self.memory[index].key = Some(pid);
    self.memory[index].value = Some(value);
    *valid_bit = true;
  }

  pub fn get_value_at(&self, index: usize) -> Option<String> {
    self.memory[index].value.clone()
  }

  pub fn set_value(&mut self, key: &String, value: &String) {
    for mem in self.memory[self.frame_store_size..].iter_mut() {
      if let Some(mem_key) = &mem.key {
        if mem_key == key {
          mem.value = Some(value.to_string());
        }
      }
    }

    for mem in self.memory[self.frame_store_size..].iter_mut() {
      if let None = &mem.key {
        mem.key = Some(key.to_string());
        mem.value = Some(value.to_string());
      }
    }
  }

  pub fn get_value(&self, key: &String) -> Option<String> {
    for mem in self.memory[self.frame_store_size..].iter() {
      if let Some(mem_key) = &mem.key {
        if mem_key == key {
          return mem.value.clone()
        }
      }
    }
    None
  }

  pub fn clear_variables(&mut self) {
    for mem in self.memory[self.frame_store_size..].iter_mut() {
      mem.key = None;
      mem.value = None;
    }
  }

  fn clear_frame(&mut self, index: usize) {
    for mem in self.memory[index..index + 3].iter_mut() {
      mem.key = None;
      mem.value = None;
    }
  }

  pub fn print_memory(&self) {
    let mut empty_count: usize = 0;
    for (i, mem) in self.memory.iter().enumerate() {
      match &mem.key {
        Some(key) => println!("Line: {}, Key: {}, Value: {}", i, key, mem.value.clone().unwrap_or(" ".to_string())),
        None => empty_count += 1
      }
    }
    println!{"Total lines: {}, Lines in use: {}, Lines free: {}", self.memory.len(), self.memory.len() - empty_count, empty_count}
  }
}

#[cfg(test)]
mod shellmemory_tests {
  use super::*;
  #[test]
  fn test_create() {
    let frame_store_size: usize = 5;
    let var_store_size: usize = 5;
    let total_size = frame_store_size + var_store_size;
    let shell_memory: ShellMemory = ShellMemory::new(frame_store_size, var_store_size);

    assert_eq!(shell_memory.frame_store_size, frame_store_size);
    assert_eq!(shell_memory.var_store_size, var_store_size);
    assert_eq!(shell_memory.memory.len(), total_size);
  }
  #[test]
  fn test_set_get_clear_value() {
    let mut shell_memory: ShellMemory = ShellMemory::new(5,5);
    assert_eq!(shell_memory.frame_store_size, 5);
    assert_eq!(shell_memory.var_store_size, 5);
    assert_eq!(shell_memory.memory.len(), 10);

    let test_key: String = "Test Key".to_string();
    let test_value: String = "Test Value".to_string();
    let test_key_2: String = "Test Key 2".to_string();

    shell_memory.print_memory();
    shell_memory.set_value(&test_key, &test_value);
    shell_memory.print_memory();

    assert_eq!(test_value, shell_memory.get_value(&test_key).unwrap());
    assert_eq!(None, shell_memory.get_value(&test_key_2));
  }
}
