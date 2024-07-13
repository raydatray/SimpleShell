use std::{f64::consts, mem::{self, MaybeUninit}};

use super::{block::{BLOCK_SECTOR_SIZE, Block, BlockSectorT}, fs_errors::FsErrors};

struct Partition<'a>{
  block: Block<'a>, //The block device in which the parition lives.
  start: BlockSectorT
}

struct PartitionTableEntry {
  bootable: bool,
  start_chs: [u8; 3], //Encoding starting cylinder, head, sector
  end_chs: [u8; 3], //Encoding ending cylinder, head, sector
  partition_type: u8, //??
  offset: u32,
  num_of_sectors: u32
}

#[repr(C, packed)] //Ensure struct ordering is C-like
struct PartitionTable {
  loader: [u8; 446], //Emulation of Master Boot Record (does nothing here....)
  partitions: [PartitionTableEntry; 4],
  signature: u16 //0xAA55 signature
}

impl<'a> Partition<'a> {
  pub fn new(block: Block<'a>, start: BlockSectorT) -> Partition<'a> {
    Partition {
      block,
      start
    }
  }

  pub fn register_partition(block: Block, partition_type: u8, start: BlockSectorT, size: BlockSectorT, part_nr: usize, fname: String) -> Result<(), FsErrors> {
    if start >= block.get_block_size() {
      todo!("Error Partition past end of device");
    } else if start + size < start || start + size > block.get_block_size() {
      todo!("Error partition end past end of device");
    } else {
      let partition_name = format!("{}-{}", block.get_block_name(), part_nr);
      let paritiion = Partition::new(block, start);


      Ok(())
    }
  }


  pub fn read_partition_to_buffer(&self, sector: BlockSectorT, buffer: &mut Vec<u8>) -> Result<(), FsErrors> {
    self.block.read_block_to_buffer(sector + 1, buffer)
  }

  pub fn write_buffer_to_partition(&self, sector: BlockSectorT, buffer: &Vec<u8>) -> Result<(), FsErrors> {
    self.block.write_buffer_to_block(sector + 1, buffer)
  }
}

impl PartitionTable {
  fn read_partition_table(block: &Block, sector: BlockSectorT, primary_extended_sector: BlockSectorT, part_nr: usize, fname: String) -> Result<(), FsErrors> {
    if sector >= block.get_block_size() {
      return Err(FsErrors::SectorOutOfBounds(sector));
    }

    let mut buffer = vec![0u8; BLOCK_SECTOR_SIZE as usize]; //We create an empty buffer of 512 bytes

    assert_eq!(mem::size_of::<PartitionTable>(), BLOCK_SECTOR_SIZE as usize); //We assert that Partition Table is 512 bytes
    assert_eq!(buffer.len(), BLOCK_SECTOR_SIZE as usize); //We assert that the size of the buffer is 512 bytes

    block.read_block_to_buffer(sector, &mut buffer)?;

    //We coerce the buffer into a Partition Table SCARY !!
    let partition_table: PartitionTable = unsafe { std::ptr::read(buffer.as_ptr() as *const _) };

    if partition_table.signature != 0xAA55 {
      if primary_extended_sector == 0 {
        todo!("Implement invalid parition signature error");
      } else {
        todo!("Implement invalid ext partition table");
      }
    }

    for entry in partition_table.partitions.iter() {
      match (entry.num_of_sectors, entry.partition_type) {
        (0, _) | (_, 0) => {
          todo!("Immediately return (ignore)")
        },
        (_, 0x05) | (_, 0x0f) | (0, 0x85) | (0, 0xc5) => {
          format!("{}: Extended partition in sector: {}", block.get_block_name(), sector);
          if sector == 0 {
            return PartitionTable::read_partition_table(block, entry.offset, entry.offset, part_nr, fname)
          } else {
            return PartitionTable::read_partition_table(block, entry.offset + primary_extended_sector, primary_extended_sector, part_nr, fname)
          }
        },
        _ => {
          part_nr += 1;

        }
      }
    }



    Ok(())
  }
}
