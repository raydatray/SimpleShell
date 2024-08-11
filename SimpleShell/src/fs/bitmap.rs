use std::{
  cell::RefCell,
  mem
};

use bytemuck::pod_collect_to_vec;

use crate::fs::file::File;
use crate::fs::free_map::Freemap;
use crate::fs::fs_errors::FsErrors;

type ElementType = u32;
const ELEMENT_BITS: u32 = (mem::size_of::<ElementType>() * u8::BITS as usize) as u32; //How many bits in ElemType

#[inline]
///Returns the index of the element that contains the bit numbered bit_idx
pub fn element_idx(bit_idx: u32) -> u32 {
  bit_idx / ELEMENT_BITS
}

#[inline]
//Returns an ElementType where only bit_idx is 1
pub fn bit_mask(bit_idx: u32) -> ElementType {
  1 << (bit_idx % ELEMENT_BITS)
}

#[inline]
//Returns the number of elements required for bit_cnt bits
pub fn element_cnt(bit_cnt: u32) -> u32 {
  (bit_cnt + ELEMENT_BITS - 1) / ELEMENT_BITS
}

#[inline]
//Returns the number of bytes required for bit_cnt bits
pub fn byte_cnt(bit_cnt: u32) -> u32 {
  mem::size_of::<ElementType>() as u32 * element_cnt(bit_cnt)
}

#[inline]
//Returns a bit mask where the actually used in the lsat element of B's bits are set to 1 and rest are 0
pub fn last_mask(bitmap: &Bitmap) -> ElementType {
  let last_bits = bitmap.bit_cnt % ELEMENT_BITS;
  return match last_bits {
    0 => ElementType::MAX,
    _ => (1 << last_bits) - 1
  }
}

#[inline]
fn bitmap_byte_size(bit_cnt: u32) -> u32 {
  mem::size_of::<Bitmap>() as u32 + byte_cnt(bit_cnt)
}

///A data structure representing a bitmap, internally comprised of a vector of ElementType's (u32)
///
///Each bit may be set individually, by modifying the ElementType that contains it.
///
///Ex. [Element 1, Element 2] Element 1 contains bits 0-31, Element 2 contains bits 32-63, etc..
pub struct Bitmap {
  bit_cnt: u32,
  bits: RefCell<Vec<ElementType>>
}

impl Bitmap {
  ///Init a new BITMAP of size BIT_CNT with all bits set to 0
  pub fn new(bit_cnt: u32) -> Self {
    let element_count = byte_cnt(bit_cnt);
    Self {
      bit_cnt,
      bits: RefCell::new(vec![0; element_count as usize])
    }
  }

  //Write a bitmap into a preallocated location in a block device?
  fn new_in_block() -> () {
    todo!();
  }

  ///Init a new BITMAP of size BIT_CNT with all bits set to the values found in the buffer
  ///
  ///Safety: the len of BUFFER must equal the byte_cnt of BIT_CNT
  fn new_from_buffer(bit_cnt: u32, buffer: &[u8]) -> Self {
    assert!(!buffer.is_empty());
    assert_eq!(byte_cnt(bit_cnt) as usize, buffer.len());

    let bits = pod_collect_to_vec::<u8, ElementType>(buffer);

    Self {
      bit_cnt,
      bits: RefCell::new(bits)
    }
  }

  ///Return the number of BITS contained in the BITMAP
  pub fn get_size(&self) -> u32 {
    self.bit_cnt
  }

  ///Returns a clone of the BITS in the BITMAP
  fn get_bits(&self) -> Vec<ElementType> {
    self.bits.borrow().clone()
  }

  ///Sets the bit @ INDEX to VALUE
  ///
  ///Fails if index is out of bounds
  pub fn set(&self, index: u32, value: bool) {
    assert!(index < self.bit_cnt);

    match value {
      true => {
        self.mark(index);
      },
      false => {
        self.reset(index);
      }
    }
  }

  pub fn mark(&self, bit_idx: u32) {
    let idx = element_idx(bit_idx);
    let mask = bit_mask(bit_idx);

    self.bits.borrow_mut()[idx as usize] |= mask;
  }

  pub fn reset(&self, bit_idx: u32) {
    let idx = element_idx(bit_idx);
    let mask = bit_mask(bit_idx);

    self.bits.borrow_mut()[idx as usize] &= !mask;
  }

  fn flip(&self, bit_idx: u32) {
    let idx = element_idx(bit_idx);
    let mask = bit_mask(bit_idx);

    self.bits.borrow_mut()[idx as usize] ^= mask;
  }

  fn test(&self, idx: u32) -> bool {
    assert!(idx < self.bit_cnt);
    (self.bits.borrow()[element_idx(idx) as usize] & bit_mask(idx)) != 0
  }

  fn set_all(&self, val: bool) {
    self.set_multiple(0, self.get_size(), val)
  }

  pub fn set_multiple(&self, start: u32, cnt: u32, val: bool) {
    assert!(start <= self.bit_cnt);
    assert!(start + cnt <= self.bit_cnt);

    (start..start + cnt).for_each(|i| self.set(start + i, val));
  }

  pub fn count(&self, start: u32, cnt: u32, val: bool) -> u32 {
    assert!(start <= self.bit_cnt);
    assert!(start + cnt <= self.bit_cnt);

    (start..start + cnt).fold(0, |acc, i| if self.test(i) == val { acc + 1 } else { acc })
  }

  fn contains(&self, start: u32, cnt: u32, val: bool) -> bool {
    assert!(start <= self.bit_cnt);
    assert!(start + cnt <= self.bit_cnt);

    (start..start + cnt).any(|i| self.test(i) == val)
  }

  fn any(&self, start: u32, cnt: u32) -> bool {
    self.contains(start, cnt, true)
  }

  fn none(&self, start: u32, cnt: u32) -> bool {
    !self.contains(start, cnt, true)
  }

  pub fn all(&self, start: u32, cnt: u32) -> bool {
    !self.contains(start, cnt, false)
  }

  fn scan(&self, start: u32, cnt: u32, val: bool) -> Result<u32, FsErrors> {
    assert!(start <= self.bit_cnt);

    if cnt > self.bit_cnt {
      todo!("Error: not enough bits");
    }

    let last = self.bit_cnt - cnt;
    return match (start..=last).find(|i| !self.contains(*i, cnt, !val)) {
      Some(index) => {
        Ok(index)
      },
      None => {
        Err(todo!())
      }
    }
  }

  pub fn scan_and_flip(&self, start: u32, cnt: u32, val: bool) -> Result<u32, FsErrors> {
    let idx = self.scan(start, cnt, val)?;

    self.set_multiple(start, cnt, !val);
    Ok(idx)
  }

  fn file_size(&self) -> u32 {
    byte_cnt(self.bit_cnt)
  }

  pub fn read_from_file(&self, file: &mut File) -> Result<u32, FsErrors> {
    let size = byte_cnt(self.bit_cnt);
    let mut buffer= vec![0u8; size as usize];
    let bytes_read = file.read_at(&mut buffer, size, 0)?;

    assert_eq!(bytes_read, size);

    let mut read_bits: Vec<ElementType> = bytemuck::allocation::cast_vec(buffer);

    read_bits[(element_cnt(self.bit_cnt) - 1) as usize] &= last_mask(self);
    self.bits.replace(read_bits);
    Ok(bytes_read)
  }

  pub fn write_to_file(&self, freemap: &mut Freemap, file: &mut File) -> Result<u32, FsErrors>{
    let size = byte_cnt(self.bit_cnt);
    let bits: Vec<u8> = bytemuck::cast_vec(self.bits.clone().take());

    file.write_at(freemap, &bits, size, 0)
  }
}
