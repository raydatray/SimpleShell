use std::mem;
use super::{file::File, free_map::Freemap, fs_errors::FsErrors};

type ElementType = u32;
const ELEMENT_BITS: u32 = (mem::size_of::<ElementType>() * u8::BITS as usize) as u32; //How many bits in ElemType

#[inline]
//Returns the idx of the element that contains the bit numbered bit_idx
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
  mem::size_of::<ElementType>()  as u32 * element_cnt(bit_cnt)
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

pub struct Bitmap {
  bit_cnt: u32, //size_t is type alias for long, 32 bits
  bits: Vec<ElementType>
}

impl Bitmap {
  //Init a new bitmap with all bits set as 0
  pub fn new(bit_cnt: u32) -> Self {
    let element_count = byte_cnt(bit_cnt);
    Self {
      bit_cnt,
      bits: vec![0; element_count as usize]
    }
  }

  //Write a bitmap into a preallocated location in a block device?
  fn new_in_block() -> () {
    todo!();
  }

  fn new_from_buffer(bit_cnt: u32, buffer: &[u8]) -> Self {
    assert!(!buffer.is_empty());

    let mut bitmap = Bitmap::new(bit_cnt);
    let element_count = byte_cnt(bit_cnt);

    assert!(element_count <= bitmap.bits.len() as u32);

    unsafe {
      std::ptr::copy_nonoverlapping(
        buffer.as_ptr() as *const ElementType,
        bitmap.bits.as_mut_ptr(),
        element_count as usize);
    }
    bitmap
  }

  pub fn get_size(&self) -> u32 {
    self.bit_cnt
  }

  fn get_bits(&self) -> &Vec<ElementType> {
    &self.bits
  }

  pub fn set(&mut self, index: u32, value: bool) {
    assert!(index < self.bit_cnt);

    return match value {
      true => {
        self.mark(index);
      },
      false => {
        self.reset(index);
      }
    }
  }

  pub fn mark(&mut self, bit_idx: u32) {
    let idx = element_idx(bit_idx);
    let mask = bit_mask(bit_idx);

    self.bits[idx as usize] |= mask;
  }

  pub fn reset(&mut self, bit_idx: u32) {
    let idx = element_idx(bit_idx);
    let mask = bit_mask(bit_idx);

    self.bits[idx as usize] &= !mask;
  }

  fn flip(&mut self, bit_idx: u32) {
    let idx = element_idx(bit_idx);
    let mask = bit_mask(bit_idx);

    self.bits[idx as usize] ^= mask;
  }

  fn test(&self, idx: u32) -> bool {
    assert!(idx < self.bit_cnt);
    (self.bits[element_idx(idx) as usize] & bit_mask(idx)) != 0
  }

  fn set_all(&mut self, val: bool) {
    self.set_multiple(0, self.get_size(), val)
  }

  pub fn set_multiple(&mut self, start: u32, cnt: u32, val: bool) {
    assert!(start <= self.bit_cnt);
    assert!(start + cnt <= self.bit_cnt);

    (start..start + cnt).map(|i| self.set(start + i, val));
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
    (start..=last).find(|i| !self.contains(*i, cnt, !val)).ok_or(todo!("Error: no contiguous allocaiton found"))
  }

  pub fn scan_and_flip(&mut self, start: u32, cnt: u32, val: bool) -> Result<u32, FsErrors> {
    let idx = self.scan(start, cnt, val)?;

    self.set_multiple(start, cnt, !val);
    Ok(idx)
  }

  fn file_size(&self) -> u32 {
    byte_cnt(self.bit_cnt)
  }

  pub fn read_from_file(&mut self, file: &mut File) -> Result<u32, FsErrors> {
    let size = byte_cnt(self.bit_cnt);
    let bytes_written = file.read_at(&mut self.bits, size, 0)?;

    assert_eq!(bytes_written, size);

    self.bits[(element_cnt(self.bit_cnt) - 1) as usize] &= last_mask(self);
    Ok(())
  }

  pub fn write_to_file(&self, freemap: &mut Freemap, file: &mut File) -> Result<u32, FsErrors>{
    let size = byte_cnt(self.bit_cnt);
    file.write_at(freemap, &self.bits, size, 0)
  }
}
