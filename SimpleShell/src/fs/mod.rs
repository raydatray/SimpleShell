mod ata;
mod block;
mod bitmap;
mod cache;
mod directory;
mod file;
mod file_sys;
mod freemap;
mod fserrors;
mod inode;
mod util;

use std::f64::consts::PI;

use clap::{Parser, Subcommand};

use block::Block;
use file_sys::FileSystem;
use fserrors::FSErrors;

#[derive(Parser)]
#[command(name = "fs")]
#[command(about = "File system operations")]
pub struct FSCommands {
  #[command(subcommand)]
  pub command: FSSubcommands
}

#[derive(Subcommand)]
pub enum FSSubcommands {
  #[command(about = "Createa a file or directory")]
  Create {
    #[arg(help = "Name of file or directory to create")]
    name: String,
    #[arg(short, long, help = "Initial size of file in bytes (Default 0")]
    size: Option<u32>,
    #[arg(short, long, help = "Create as directory")]
    is_dir: bool
  },
  #[command(about = "Display the contents of a file as ASCII and HEX")]
  Cat {
    #[arg(help = "Name of file to cat")]
    name: String
  },
  #[command(about = "Remove a file or directory")]
  Remove {
    #[arg(help = "Name of file or directory to remove")]
    name: String
  },
  #[command(about = "List files and directories of CWD")]
  List {},
  #[command(about = "Write content to a file")]
  Write {
    #[arg(help = "Name of file to write to")]
    name: String,
    #[arg(help = "Contents to write")]
    content: String
  },
  #[command(about = "Find files or directories whose content contains a pattern")]
  Find {
    #[arg(help = "Pattern to search for")]
    pat: String
  },
  #[command(about = "Read content from file")]
  Read {
    #[arg(help = "Name of file to read from")]
    name: String,
    #[arg(help = "Number of bytes to read ")]
    size: u32
  },
  #[command(about = "Copy in a file from host device to device")]
  CopyIn {
    #[arg(help = "Name of file to read from")]
    name: String
  },
  #[command(about = "Copy out a file on device to host device")]
  CopyOut {
    #[arg(help = "Name of file to read from")]
    name: String
  },
  #[command(about = "Size of a file")]
  Size {
    #[arg(help = "Name of file to read from")]
    name: String
  },
  #[command(about = "Set offset of a file")]
  Seek {
    #[arg(help = "Name of file to read from")]
    name: String,
    #[arg(help = "Offset in bytes from start to set")]
    ofst: u32
  },
  #[command(about = "Number of free sectors on device")]
  FreeSpace {},
  #[command(about = "Degree of fragmentation of all files on device")]
  FragmentationDegree {},
  #[command(about = "Defragment all files")]
  Defragment {},
  #[command(about = "Recover deleted files")]
  Recover {}
}

pub struct FSModule<'a> {
  inner: FileSystem<'a>
}

impl<'a> FSModule<'a> {
  pub fn new(block: Block<'a>, fmt: bool) -> Result<Self, FSErrors> {
    let file_sys = FileSystem::new(block, fmt)?;
    Ok(
      Self {
        inner: file_sys
      }
    )
  }

  pub fn exec_cmd(&mut self, cmd: FSSubcommands) -> Result<(), FSErrors> {
    match cmd {
      FSSubcommands::Create { name, size, is_dir } => {
        self.inner.util_create(&name, size.unwrap_or(0), is_dir)
      },
      FSSubcommands::Cat { name } => {
        self.inner.util_cat(&name)
      },
      FSSubcommands::Remove { name } => {
        self.inner.util_rm(&name)
      },
      FSSubcommands::List {} => {
        self.inner.util_ls()
      },
      FSSubcommands::Write { name, content } => {
        self.inner.util_write(&name, content.as_bytes(), content.len() as u32)
      },
      FSSubcommands::Find { pat } => {
        self.inner.util_find_file(&pat)
      },
      FSSubcommands::Read { name, size } => {
        let mut buffer = vec![0u8; size as usize];
        self.inner.util_read(&name, &mut buffer, size)?;
        println!("{}", String::from_utf8_lossy(&buffer));
        Ok(())
      },
      FSSubcommands::CopyIn { name } => {
        self.inner.util_copy_in(&name)
      },
      FSSubcommands::CopyOut { name } => {
        self.inner.util_copy_out(&name)
      },
      FSSubcommands::Size { name } => {
        self.inner.util_size(&name)
      },
      FSSubcommands::Seek { name, ofst } => {
        self.inner.util_seek(&name, ofst)
      },
      FSSubcommands::FreeSpace {} => {
        self.inner.util_freespace();
        Ok(())
      },
      FSSubcommands::FragmentationDegree {} => {
        self.inner.util_frag_degree()
      },
      FSSubcommands::Defragment {} => {
        self.inner.util_defrag()
      },
      FSSubcommands::Recover {} => {
        self.inner.util_recover()
      },
    }
  }
}
