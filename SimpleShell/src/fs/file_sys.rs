use std::{borrow::Borrow, cell::{Ref, RefCell}, fs::OpenOptions, io::{BufReader, BufWriter, Read, Write}, os::unix::fs::MetadataExt, path::Path, rc::Rc};

use bytemuck::from_bytes;

use crate::fs::{
  block::Block,
  cache::Cache,
  directory::MemoryDirectory,
  file::FileTable,
  freemap::Freemap,
  fserrors::FSErrors,
  inode::InodeList, util::hex_dump
};

use super::{block::BLOCK_SECTOR_SIZE, directory::split_path, file::File, inode::{DiskInode, MemoryInode, INODE_SIGNATURE}};

pub const FREE_MAP_SECTOR: u32 = 0u32;
pub const ROOT_DIR_SECTOR: u32 = 1u32;
pub const MAX_FILES_PER_DIRECTORY: u32 = 1000u32;

pub struct FileSystem<'file_sys> {
  pub block: Block<'file_sys>,
  pub cache: Cache,
  pub file_table: FileTable,
  pub freemap: Freemap,
  pub inode_list: InodeList,
  pub cwd: Option<Rc<RefCell<MemoryDirectory>>>
}

impl<'file_sys> FileSystem<'file_sys> {
  pub fn new(block: Block<'file_sys>, format: bool) -> Result<Self, FSErrors> {
    let block_size = block.get_size();

    let mut file_sys = Self {
      block,
      cache: Cache::new(),
      freemap: Freemap::new(block_size),
      file_table: FileTable::new(),
      inode_list: InodeList::new(),
      cwd: None
    };

    if format {
      Self::format(&mut file_sys)?;
    }

    Freemap::open_from_file(&mut file_sys)?;
    println!("Number of free sectors: {}", file_sys.freemap.num_free_sectors());

    Ok(file_sys)
  }

  fn format(&mut self) -> Result<(), FSErrors> {
    println!("Formatting file system...");
    Freemap::create_on_disk(self)?;
    MemoryDirectory::new_on_disk(self, ROOT_DIR_SECTOR, MAX_FILES_PER_DIRECTORY)?;
    Freemap::close(self)?;
    Ok(())
  }


  pub fn close(&mut self) -> Result<(), FSErrors> {
    Freemap::close(self)?;
    Cache::close(&self.cache, &self.block)?;
    FileTable::close(self)?;
    Ok(())
  }

  pub fn create(&mut self, path: &str, init_size: u32, is_dir: bool) -> Result<(), FSErrors> {
    let (_, suffix) = split_path(path);
    let dir = MemoryDirectory::open_root(self)?;

    let sector = Freemap::allocate(self, 1)?;
    if let Err(e) = DiskInode::new(self, sector, init_size, is_dir) {
      Freemap::release(self, sector, 1)?;
      return Err(FSErrors::InodeError(e));
    }

    if let Err(e) = MemoryDirectory::add(dir.borrow_mut(), self, suffix, sector, is_dir) {
      Freemap::release(self, sector, 1)?;
      return Err(FSErrors::DirError(e));
    }

    Ok(())
  }

  pub fn open(&mut self, path: &str) -> Result<Rc<RefCell<File>>, FSErrors> {
    let (_, suffix) = split_path(path);
    let dir = MemoryDirectory::open_root(self)?;
    let inode;

    match suffix.len() {
      0 => {
        inode = dir.as_ref().borrow().get_inode();
      },
      _ => {
        inode = dir.as_ref().borrow().search(self, path)?;
      }
    }
    Ok(Rc::new(RefCell::new(File::open(inode))))
  }

  pub fn remove(&mut self, path: &str) -> Result<(), FSErrors> {
    let (prefix, suffix) = split_path(path);
    let dir = MemoryDirectory::open_path(self, prefix)?;

    MemoryDirectory::remove(dir.borrow_mut(), self, suffix)?;
    Ok(())
  }

  pub fn chdir(&mut self, path: &str) -> Result<(), FSErrors> {
    let dir = MemoryDirectory::open_path(self, path)?;

    self.cwd = Some(dir);
    Ok(())
  }

  ///Utilities
  ///Lists all files in the ROOT DIRECTORY
  pub fn util_ls(&mut self) -> Result<(), FSErrors> {
    let dir = MemoryDirectory::open_root(self)?;
    println!("Files in the root directory");

    let names = dir.as_ref().borrow().read_names(self)?;

    for name in names {
      println!("{}", name);
    }

    println!("End of listing");
    Ok(())
  }

  pub fn util_cat(&mut self, name: &str) -> Result<(), FSErrors> {
    println!("Printing <{}> as ASCII and HEX...", name);

    let opened = FileTable::get_by_name(&self.file_table, name);

    let file = match opened {
      Some(file) => {
        file
      },
      None => {
        let file = self.open(name)?;
        FileTable::add_by_name(&mut self.file_table, file.clone(), name);
        file
      }
    };

    let curr_ofst = file.as_ref().borrow().tell();
    file.borrow_mut().seek(0);

    let mut buffer = [0u8; 1024];

    loop {
      let bytes_read = file.borrow_mut().read(&self.block, &self.cache, &mut buffer, 1024)?;

      if bytes_read == 0 {
        break;
      }

      hex_dump((file.as_ref().borrow().tell() -  bytes_read) as usize, &buffer[..bytes_read as usize], bytes_read as usize, true);
    }

    file.borrow_mut().seek(curr_ofst);
    println!("Done printing");

    Ok(())
  }

  pub fn util_rm(&mut self, name: &str) -> Result<(), FSErrors> {
    FileTable::remove_by_name(self, name)?;
    self.remove(name)
  }

  pub fn util_create(&mut self, name: &str, len: u32, is_dir: bool) -> Result<(), FSErrors> {
    if name.len() >= 255 {
      return Err(FSErrors::InvalidName(name.to_string(), name.len()))
    }
    self.create(name, len, false)
  }

  pub fn util_write(&mut self, name: &str, buffer: &[u8], len: u32) -> Result<(), FSErrors> {
    let opened = FileTable::get_by_name(&self.file_table, name);

    let file = match opened {
      Some(file) => {
        file
      },
      None => {
        let file = self.open(name)?;
        FileTable::add_by_name(&mut self.file_table, file.clone(), name);
        file
      }
    };

    file.borrow_mut().write(self, buffer, len)?;
    Ok(())
  }

  pub fn util_read(&mut self, name: &str, buffer: &mut [u8], len: u32) -> Result<(), FSErrors> {
    let opened = FileTable::get_by_name(&self.file_table, name);

    let file = match opened {
      Some(file) => {
        file
      },
      None => {
        let file = self.open(name)?;
        FileTable::add_by_name(&mut self.file_table, file.clone(), name);
        file
      }
    };

    file.borrow_mut().read(&self.block, &self.cache, buffer, len)?;
    Ok(())
  }

  pub fn util_size(&mut self, name: &str) -> Result<(), FSErrors> {
    let opened = FileTable::get_by_name(&self.file_table, name);

    let file = match opened {
      Some(file) => {
        file
      },
      None => {
        let file = self.open(name)?;
        FileTable::add_by_name(&mut self.file_table, file.clone(), name);
        file
      }
    };

    let curr_ofst = file.as_ref().borrow().tell();
    let len = file.as_ref().borrow().len();
    file.as_ref().borrow().seek(curr_ofst);
    println!("Size of file: {} is {} bytes", name, len);
    Ok(())
  }

  pub fn util_seek(&mut self, name: &str, ofst: u32) -> Result<(), FSErrors> {
    let opened = FileTable::get_by_name(&self.file_table, name);

    let file = match opened {
      Some(file) => {
        file
      },
      None => {
        let file = self.open(name)?;
        FileTable::add_by_name(&mut self.file_table, file.clone(), name);
        file
      }
    };

    file.as_ref().borrow().seek(ofst);
    Ok(())
  }

  pub fn util_close(&mut self, name: &str) -> Result<(), FSErrors> {
    FileTable::remove_by_name(self, name)?;
    Ok(())
  }

  pub fn util_freespace(&self) {
    let num_free_sectors = self.freemap.num_free_sectors();
    println!("Number of free sectors: {}", num_free_sectors);
  }

  pub fn util_copy_in(&mut self, name: &str) -> Result<(), FSErrors> {
    let source_file = OpenOptions::new().read(true).open(name)?;
    let source_file_size = source_file.metadata().unwrap().size(); //TODO: u32

    let target_file_name = Path::new(name)
      .file_name()
      .and_then(|name|{
        name.to_str()
      })
      .ok_or_else(|| {
        FSErrors::InvalidName(name.to_string(), name.len())
      })?;

    println!("Source File Name: {}", name);
    println!("Target File Name: {}", target_file_name);
    println!("Size of source file: {}", source_file_size);

    self.create(target_file_name, 10, false)?;
    let target_file = self.open(target_file_name)?;

    let mut reader = BufReader::new(source_file);
    let mut buffer = [0u8; 1024];
    let mut bytes_written = 0;

    loop {
      let bytes_read = reader.read(&mut buffer)? as u32;

      if bytes_read == 0 {
        break;
      }

      let actual_bytes_written = target_file.borrow_mut().write(self, &buffer, bytes_read)?;
      bytes_written += actual_bytes_written;

      if actual_bytes_written < bytes_read {
        println!("Warning: Could only write {} out of {} bytes (reached end of file)", bytes_written, source_file_size as u32);
        return Ok(())
      }
    }

    println!("Bytes written: {}", bytes_written);
    Ok(())
  }

  pub fn util_copy_out(&mut self, name: &str) -> Result<(), FSErrors> {
    let opened = FileTable::get_by_name(&self.file_table, name);

    let (source_file, close) = match opened {
      Some(file) => {
        (file, false)
      },
      None => {
        let file = self.open(name)?;
        FileTable::add_by_name(&mut self.file_table, file.clone(), name);
        (file, true)
      }
    };

    let target_file = OpenOptions::new().write(true).create(true).open(name)?;

    let mut writer = BufWriter::new(target_file);
    let mut buffer = [0u8; 1024];
    let mut bytes_written = 0;
    let mut ofst = 0;

    loop {
      let bytes_read = source_file.borrow_mut().read_at(&self.block, &self.cache, &mut buffer, 1024, ofst)?;

      if bytes_read == 0 {
        break;
      }

      let actual_bytes_written = writer.write(&buffer)? as u32;
      bytes_written += actual_bytes_written;
      ofst += actual_bytes_written;
    }

    if close {
      FileTable::remove_by_name(self, name)?;
    }

    Ok(())
  }

  pub fn util_find_file(&mut self, pat: &str) -> Result<(),FSErrors> {
    let root = MemoryDirectory::open_root(self)?;
    let entry_names = root.as_ref().borrow().read_names(self)?;

    let mut buffer = [0u8; 1024];

    for entry_name in entry_names {
      let opened = FileTable::get_by_name(&self.file_table, &entry_name);

      let (file, close) = match opened {
        Some(file) => {
          (file, false)
        },
        None => {
          let file = self.open(&entry_name)?;
          FileTable::add_by_name(&mut self.file_table, file.clone(), &entry_name);
          (file, true)
        }
      };

      let mut ofst = 0;

      loop {
        let bytes_read = file.borrow_mut().read_at(&self.block, &self.cache, &mut buffer, 1024, ofst)?;

        if bytes_read == 0 {
          break;
        }

        let buffer_as_str = String::from_utf8_lossy(&buffer[..bytes_read as usize]);

        if buffer_as_str.contains(pat) {
          println!("{}", entry_name);
          break;
        }

        ofst += bytes_read;
      }

      if close {
        FileTable::remove_by_name(self, &entry_name)?;
      }
    }

    Ok(())
  }

  pub fn util_frag_degree(&mut self) -> Result<(), FSErrors> {
    let root = MemoryDirectory::open_root(self)?;
    let entry_names = root.as_ref().borrow().read_names(self)?;

    let (fragmented_files, total_files) = entry_names.iter().try_fold((0, 0), |acc, name| -> Result<(i32, i32), FSErrors> {
      let opened = FileTable::get_by_name(&self.file_table, name);

      let (file, close) = match opened {
        Some(file) => (file.clone(), false),
        None => {
          let file = self.open(name)?;
          FileTable::add_by_name(&mut self.file_table, file.clone(), name);
          (file, true)
        }
      };

      let memory_inode = file.as_ref().borrow().inode(self)?;
      let data_sectors = memory_inode.as_ref().borrow().data_sectors(&self.block, &self.cache)?;

      let fragmented = data_sectors.windows(2).any(|window| window[1] - window[0] > 3);

      if close {
        FileTable::remove_by_name(self, name)?;
      }

      Ok((
        acc.0 + if fragmented { 1 } else { 0 },
        acc.1 + 1
        ))
    })?;

    println!("Fragmented Files: {}", fragmented_files);
    println!("Total Files: {}", total_files);
    if total_files > 0 {
      println!("Fragmentation %: {:.2}%", (fragmented_files as f64 / total_files as f64) * 100.0);
    } else {
      println!("Fragmentation %: 0%");
    }
    Ok(())
  }

  pub fn util_defrag(&mut self) -> Result<(), FSErrors> {
    struct TempFile {
      file_name: String,
      content: Vec<u8>
    }

    let root = MemoryDirectory::open_root(self)?;
    let entry_names = root.as_ref().borrow().read_names(self)?;

    let mut temp_files = Vec::new();

    for name in entry_names {
      let file = self.open(&name)?;
      let file_len = file.as_ref().borrow().len();

      let mut content = vec![0u8; file_len as usize];
      file.borrow_mut().read_at(&self.block, &self.cache, &mut content, file_len, 0)?;

      temp_files.push(TempFile {
        file_name: name.to_string(),
        content,
      });

      self.remove(&name)?;
    }

    self.util_freespace();

    for file in temp_files {
      self.create(&file.file_name, file.content.len() as u32, false)?;

      let new_file = self.open(&file.file_name)?;
      new_file.borrow_mut().write(self, &file.content, file.content.len() as u32)?;
    }

    Ok(())
  }

  pub fn util_recover(&mut self) -> Result<(), FSErrors> {
    for idx in 0..self.freemap.inner.get_size() {
      if !self.freemap.inner.test(idx) {
        let mut buffer = [0u8; BLOCK_SECTOR_SIZE as usize];

        self.cache.read_to_buffer(&self.block, idx, &mut buffer)?;
        let recovered_inode = from_bytes::<DiskInode>(&buffer).to_owned();

        if recovered_inode.sign == INODE_SIGNATURE {
          let recovered_name = format!("recovered_file-{}", idx);

          let is_dir = match recovered_inode.is_dir {
            0u8 => { false },
            1u8 => { true },
            _ => panic!()
          };

          self.create(&recovered_name, recovered_inode.len, is_dir)?;
          let recovered_file = self.open(&recovered_name)?;
          let sectors = recovered_file.borrow_mut().inode(self)?.borrow_mut().data_sectors(&self.block, &self.cache)?;

          for sector in sectors.iter() {
            self.cache.read_to_buffer(&self.block, *sector, &mut buffer)?;
            recovered_file.borrow_mut().write(self, &buffer, BLOCK_SECTOR_SIZE)?;
          }
          recovered_file.borrow_mut().close(self)?;
        }
      }
    }
    Ok(())
  }
}
