use std::{cell::RefCell, rc::Rc};

use crate::fs::{
  block::Block,
  cache::Cache,
  directory::MemoryDirectory,
  file::FileTable,
  freemap::Freemap,
  fserrors::FSErrors,
  inode::InodeList, util::hex_dump
};

use super::{directory::split_path, file::File, inode::DiskInode};

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
        inode = MemoryDirectory::get_inode(dir.borrow());
      },
      _ => {
        inode = MemoryDirectory::search(dir.borrow(), self, suffix)?;
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

    let names = MemoryDirectory::read(dir.borrow(), self)?;

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

    let curr_ofst = file.borrow().tell();
    file.borrow_mut().seek(0);

    let mut buffer = [0u8; 1024];

    loop {
      let bytes_read = file.borrow_mut().read(&self.block, &self.cache, &mut buffer, 1024)?;

      if bytes_read == 0 {
        break;
      }

      hex_dump((file.borrow().tell() -  bytes_read) as usize, &buffer[..bytes_read as usize], bytes_read as usize, true);
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

  pub fn util_write(&mut self, name: &str, len: u32) -> Result<(), FSErrors> {
    todo!()
  }

}
