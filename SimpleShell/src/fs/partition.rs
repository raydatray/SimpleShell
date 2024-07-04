use std::mem;

use super::{block::{BLOCK_SECTOR_SIZE, Block, BlockSectorT}, fs_errors::FsErrors};

struct Partition {
  block: Block,
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
  loader: [u8; 446], //??
  partitions: [PartitionTableEntry; 4],
  signature: u16 //?? Should be 0xAA55
}


impl PartitionTable {
  fn read_partition_table(block: Block, sector: BlockSectorT, primary_extended_sector: BlockSectorT, part_nr: usize, fname: String) -> Result<(), FsErrors> {
    if sector >= block.get_block_size() {
      println!("Partition table @ sector {} is past end of device", sector);
      return Err(FsErrors::SectorOutOfBounds(sector));
    }

    let mut buf = vec![0u8; BLOCK_SECTOR_SIZE as usize]; //We create an empty buffer of 512 bytes

    assert_eq!(mem::size_of::<PartitionTable>(), BLOCK_SECTOR_SIZE as usize); //We assert that Partition Table is 512 bytes
    assert_eq!(buf.len(), BLOCK_SECTOR_SIZE as usize); //We assert that the size of the buffer is 512 bytes

    //We fill the buffer with the first block (as bytes)

    //We then coerce those 512 bytes into a Partition Table SCARY !!

    unsafe{

    }


    Ok(())
  }
}
