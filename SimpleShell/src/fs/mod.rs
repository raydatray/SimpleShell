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

use clap::{Parser, Subcommand};

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
