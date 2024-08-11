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
  pub fn new(frame_store_size: usize, var_store_size: usize) -> ShellMemory {
    ShellMemory {
      memory: vec![MemoryStruct::new(); frame_store_size + var_store_size],
      frame_store_size,
      var_store_size
    }
  }

  pub fn alloc_frame(&mut self, pid: &String, index: &mut [usize; 3], valid_bit: &mut [bool; 3]) -> Result<(), ShellErrors> {
    for (i, mem) in self.memory[..self.frame_store_size].iter().enumerate() {
      let mut j = i;
      while j < i + 3 && j < self.frame_store_size {
        if let Some(_) = &mem.key {
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

  fn clear_frame(&mut self, index: usize) {
    for mem in self.memory[index..index + 3].iter_mut() {
      mem.key = None;
      mem.value = None;
    }
  }

  pub fn set_value_at(&mut self, index: usize, pid: &String, value: &String, valid_bit: &mut bool) {
    self.memory[index].key = Some(pid.to_string());
    self.memory[index].value = Some(value.to_string());
    *valid_bit = true;
  }

  pub fn get_value_at(&self, index: usize) -> Option<String> {
    self.memory[index].value.clone()
  }

  pub fn free_at(&mut self, index: usize)  {
    self.memory[index].key = None;
    self.memory[index].value = None;
  }

  pub fn set_var(&mut self, key: &String, value: &String) {
    for mem in self.memory[self.frame_store_size..].iter_mut() {
      if let Some(mem_key) = &mem.key {
        if mem_key == key {
          mem.value = Some(value.to_string());
          return
        }
      }
    }

    for mem in self.memory[self.frame_store_size..].iter_mut() {
      if let None = &mem.key {
        mem.key = Some(key.to_string());
        mem.value = Some(value.to_string());
        return
      }
    }
  }

  pub fn get_var_by_key(&self, key: &String) -> Option<String> {
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


  pub fn print_memory(&self) {
    let mut empty_count: usize = 0;
    for (i, mem) in self.memory.iter().enumerate() {
      println!("Line: {}, Key: {} | Value: {}", i, mem.key.clone().unwrap_or(" ".to_string()), mem.value.clone().unwrap_or(" ".to_string()));
      if mem.key == None {
        empty_count += 1;
      }
    }
    println!{"Total lines: {}, Lines in use: {}, Lines free: {}", self.memory.len(), self.memory.len() - empty_count, empty_count}
  }
}

#[cfg(test)]
mod shellmemory_tests {
  use super::*;
  pub const FRAME_STORE_SIZE: usize = 6;
  pub const VAR_STORE_SIZE: usize =  4;
  pub const TOTAL_SIZE: usize = FRAME_STORE_SIZE + VAR_STORE_SIZE;

  #[test]
  fn test_create() {
    let _total_size = FRAME_STORE_SIZE + VAR_STORE_SIZE;
    let shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    assert_eq!(shell_memory.frame_store_size, FRAME_STORE_SIZE);
    assert_eq!(shell_memory.var_store_size, VAR_STORE_SIZE);
    assert_eq!(shell_memory.memory.len(), TOTAL_SIZE);

    for mem in shell_memory.memory {
      assert_eq!(mem.value, None);
      assert_eq!(mem.key, None);
    }
  }

  #[test]
  fn test_set_get_clear_value() {
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);

    let test_key = "Test Key".to_string();
    let test_value = "Test Value".to_string();
    let test_key_2 = "Test Key 2".to_string();
    let test_value_2a = "Test Value 2A".to_string();
    let test_value_2b = "Test Value 2B".to_string();
    let test_key_3 = "Test Key 3".to_string();

    shell_memory.set_var(&test_key, &test_value);
    shell_memory.set_var(&test_key_2, &test_value_2a);

    //We expect this allocation to be contiguous from FRAME_STORE_SIZE
    for (i, mem) in shell_memory.memory.iter().enumerate(){
      match i {
        6 => { //We cant do FRAME_STORE_SIZE + 1 so...
          assert_eq!(mem.key, Some(test_key.clone()));
          assert_eq!(mem.value, Some(test_value.clone()));
        },
        7 => {
          assert_eq!(mem.key, Some(test_key_2.clone()));
          assert_eq!(mem.value, Some(test_value_2a.clone()));
        }
        _ => {
          assert_eq!(mem.key, None);
          assert_eq!(mem.value, None)
        }
      }
    }

    //We re-set test_key_2 to test_value_2b
    shell_memory.set_var(&test_key_2, &test_value_2b);

    for (i, mem) in shell_memory.memory.iter().enumerate(){
      match i {
        6 => { //We cant do FRAME_STORE_SIZE + 1 so...
          assert_eq!(mem.key, Some(test_key.clone()));
          assert_eq!(mem.value, Some(test_value.clone()));
        },
        7 => {
          assert_eq!(mem.key, Some(test_key_2.clone()));
          assert_eq!(mem.value, Some(test_value_2b.clone()));
        }
        _ => {
          assert_eq!(mem.key, None);
          assert_eq!(mem.value, None)
        }
      }
    }

    let return_value = shell_memory.get_var_by_key(&test_key);
    assert!(return_value.is_some());
    assert_eq!(return_value.unwrap(), test_value);

    let return_value_3 = shell_memory.get_var_by_key(&test_key_3);
    assert!(return_value_3.is_none());

    shell_memory.clear_variables();

    for mem in shell_memory.memory {
      assert_eq!(mem.value, None);
      assert_eq!(mem.key, None);
    }
  }
  #[test]
  fn test_alloc_frame_success() {
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    let pid = "testPid".to_string();
    let mut index = [999usize, 999usize, 999usize];
    let expected_indices = [0usize, 1usize, 2usize];
    let mut valid_bit = [true, true, true];

    let result = shell_memory.alloc_frame(&pid, &mut index, &mut valid_bit);

    assert!(result.is_ok());

    //We expect the allocation to be contiguous @0,1,2
    for (i, mem) in shell_memory.memory.iter().enumerate() {
      match i {
        0 | 1 | 2 => {
          assert_eq!(mem.key, Some(pid.clone()));
          assert_eq!(mem.value, None);
        }
        _ => {
          assert_eq!(mem.value, None);
          assert_eq!(mem.key, None);
        }
      }
    }

    //We expect index and valid_bit to all be mutated appropriately
    //Index = [0,1,2]
    //Valid bit = [false, false, false]
    for i in index {
      assert_eq!(i, expected_indices[i]);
    }

    for bit in valid_bit.iter() {
      assert_eq!(*bit, false);
    }
  }

  #[test]
  fn test_alloc_frame_fail() {
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);

    //We "allocate" the entire frame store area by placing in dummy values in the keys
    let dummy_key = "Dummy Key".to_string();
    for mem in shell_memory.memory[..FRAME_STORE_SIZE].iter_mut(){
      mem.key = Some(dummy_key.clone());
    }

    //We attempt to allocate a frame
    let pid = "testPid".to_string();
    let mut index = [999usize, 999usize, 999usize];
    let mut valid_bit = [true, true, true];

    let result = shell_memory.alloc_frame(&pid, &mut index, &mut valid_bit);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), InitialFrameAllocationFailed);

    //We assert that index and valid_bit have not been mutated, nor has shell_memory
    for mem in shell_memory.memory[..FRAME_STORE_SIZE].iter(){
      assert_eq!(mem.key, Some(dummy_key.clone()));
      assert_eq!(mem.value, None);
    }

    for i in index {
      assert_eq!(i, 999usize);
    }

    for bit in valid_bit.iter() {
      assert_eq!(*bit, true);
    }
  }

  #[test]
  fn test_clear_frame() {
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);

    let pid = "testPid".to_string();
    let mut index = [999usize, 999usize, 999usize];
    let mut valid_bit = [true, true, true];

    let _result = shell_memory.alloc_frame(&pid, &mut index, &mut valid_bit);

    shell_memory.clear_frame(0);

    shell_memory.memory.iter().for_each(|mem| {
      assert_eq!(mem.key, None);
      assert_eq!(mem.value, None);
    })
  }

  #[test]
  fn test_set_and_free_at() {
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    let test_key = "Test Key".to_string();
    let test_value = "Test Value".to_string();
    let mut flag = false;

    shell_memory.set_value_at(0, &test_key, &test_value, &mut flag);
    assert_eq!(flag, true);

    for (i, mem) in shell_memory.memory.iter().enumerate() {
      match i {
        0 => {
          assert_eq!(mem.key, Some(test_key.clone()));
          assert_eq!(mem.value, Some(test_value.clone()));
        }
        _ => {
          assert_eq!(mem.value, None);
          assert_eq!(mem.key, None);
        }
      }
    }

    shell_memory.free_at(0);

    shell_memory.memory.iter().for_each(|mem| {
      assert_eq!(mem.key, None);
      assert_eq!(mem.value, None);
    })
  }

  //No way to capture stdout, use cargo test -- --show-output
  #[test]
  fn test_print_empty() {
    let shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    shell_memory.print_memory();
  }

  #[test]
  fn test_print_full() {
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    let test_key = "Test Key".to_string();
    let test_value = "Test Value".to_string();

    shell_memory.memory.iter_mut().for_each(|mem| {
      mem.key = Some(test_key.clone());
      mem.value = Some(test_value.clone());
    });
    shell_memory.print_memory();
  }

  #[test]
  fn test_print_half() {
    let mut shell_memory = ShellMemory::new(FRAME_STORE_SIZE, VAR_STORE_SIZE);
    let test_key = "Test Key".to_string();
    let test_value = "Test Value".to_string();

    shell_memory.memory.iter_mut().step_by(2).for_each(|mem| {
      mem.key = Some(test_key.clone());
      mem.value = Some(test_value.clone());
    });
    shell_memory.print_memory();
  }
}
