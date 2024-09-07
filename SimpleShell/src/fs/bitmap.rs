use std::{
  cell::RefCell,
  mem
};

use bytemuck::{cast_vec, pod_collect_to_vec};

use crate::fs::{
  block::Block,
  cache::Cache,
  file::File,
  file_sys::FileSystem,
  fserrors::bitmap_errors::BitmapError
};

type ElementType = u32;
const ELEMENT_BITS: u32 = (mem::size_of::<ElementType>() * u8::BITS as usize) as u32;

///Returns the IDX of the element that contains the bit @ BIT_IDX
#[inline(always)]
pub fn element_idx(bit_idx: u32) -> u32 {
  bit_idx / ELEMENT_BITS
}

///Returns an ELEMENT_TYPE with only the bit @ BIT_IDX set to 1
#[inline(always)]
pub fn bit_mask(bit_idx: u32) -> ElementType {
  1 << (bit_idx % ELEMENT_BITS)
}

///Return the number of ELEMENT_TYPEs required for BIT_CNT bits
#[inline(always)]
pub fn element_cnt(bit_cnt: u32) -> u32 {
  bit_cnt.div_ceil(ELEMENT_BITS)
}

///Return the number of BYTEs required for BIT_CNT bits
#[inline(always)]
pub fn byte_cnt(bit_cnt: u32) -> u32 {
  mem::size_of::<ElementType>() as u32 * element_cnt(bit_cnt)
}

///Returns a bit mask where the actually used BITS in the last ELEMENT_TYPE of
///BITMAP are set to 1 and rest are o
#[inline(always)]
pub fn last_mask(bitmap: &Bitmap) -> ElementType {
  let last_bits = bitmap.bit_cnt % ELEMENT_BITS;
  return match last_bits {
    0 => ElementType::MAX,
    _ => (1 << last_bits) - 1
  }
}

///A data structure representing a Bitmap
///
///Each bit may be set indivdually by modifying the ELEMENT_TYPE that contains it
///
///Bits 0-31 are at idx 0, 32-63 @ idx 1, etc...
pub(crate) struct Bitmap {
  inner: RefCell<Vec<ElementType>>,
  bit_cnt: u32
}

impl Bitmap {
  pub fn new(bit_cnt: u32) -> Self {
    let element_cnt = element_cnt(bit_cnt);

    Self {
      inner: RefCell::new(vec![0; element_cnt as usize]),
      bit_cnt
    }
  }

  ///Creates a BITMAP with BIT_CNT bits in BLOCK_SIZE bytes of preallocate storage on BLOCK
  ///
  ///Unused
  fn create_in_buf() -> Self { todo!() }

  ///Creates a BITMAP with BIT_CNT bits from a pre-provided BUFFER
  fn create_from_buf(bit_cnt: u32, buffer: &[u8]) -> Self {
    let bitmap = Bitmap::new(bit_cnt);
    let inner = pod_collect_to_vec::<u8, ElementType>(&buffer);

    assert_eq!(inner.len(), bitmap.inner.borrow().len());
    bitmap.inner.replace(inner);

    bitmap
  }

  ///Return the number of BITS contained in the BITMAP
  pub fn get_size(&self) -> u32 {
    self.bit_cnt
  }

  ///Returns a vector of the BITS in the BITMAP
  pub fn get_bits(&self) -> Vec<u8> {
    let len = self.get_file_size();
    let bits = cast_vec::<ElementType, u8>(self.inner.borrow().clone());

    assert_eq!(len as usize, bits.len());
    bits
  }

  fn get_file_size(&self) -> u32 {
    byte_cnt(self.bit_cnt)
  }

  ///Sets the bit @ IDX to VALUE
  ///
  ///Panics if IDX exceeds the BIT_CNT of the bitmap
  pub fn set(&self, idx: u32, value: bool) {
    assert!(idx < self.bit_cnt);

    match value {
      true => {
        self.mark(idx);
      },
      false => {
        self.reset(idx);
      }
    }
  }

  pub fn mark(&self, bit_idx: u32) {
    let idx = element_idx(bit_idx);
    let mask = bit_mask(bit_idx);

    self.inner.borrow_mut()[idx as usize] |= mask;
  }

  pub fn reset(&self, bit_idx: u32) {
    let idx = element_idx(bit_idx);
    let mask = bit_mask(bit_idx);

    self.inner.borrow_mut()[idx as usize] &= !mask;
  }

  fn flip(&self, bit_idx: u32) {
    let idx = element_idx(bit_idx);
    let mask = bit_mask(bit_idx);

    self.inner.borrow_mut()[idx as usize] ^= mask;
  }

  pub fn test(&self, idx: u32) -> bool {
    assert!(idx < self.bit_cnt);
    (self.inner.borrow()[element_idx(idx) as usize] & bit_mask(idx)) != 0
  }

  fn set_all(&self, val: bool) {
    self.set_multiple(0, self.get_size(), val)
  }

  pub fn set_multiple(&self, start: u32, cnt: u32, val: bool) {
    assert!(start < self.bit_cnt);
    assert!(start + cnt < self.bit_cnt);

    (start..start + cnt).for_each(|i| self.set(start + i, val));
  }

  pub fn count(&self, start: u32, cnt: u32, val: bool) -> u32 {
    assert!(start < self.bit_cnt);
    assert!(start + cnt < self.bit_cnt);

    (start..start + cnt).fold(0, |acc, i| if self.test(i) == val { acc + 1 } else { acc })
  }

  fn contains(&self, start: u32, cnt: u32, val: bool) -> bool {
    assert!(start < self.bit_cnt);
    assert!(start + cnt < self.bit_cnt);

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

  fn scan(&self, start: u32, cnt: u32, val: bool) -> Result<u32, BitmapError> {
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
        Err(BitmapError::NoContiguousAllocationFound(cnt))
      }
    }
  }

  pub fn scan_and_flip(&self, start: u32, cnt: u32, val: bool) -> Result<u32, BitmapError> {
    let idx = self.scan(start, cnt, val)?;

    self.set_multiple(start, cnt, !val);
    Ok(idx)
  }

  pub fn read_from_file(&self, block: &Block, cache: &Cache, file: &File) -> Result<(), BitmapError> {
    let len = self.get_file_size();
    let mut buffer = Vec::<u8>::with_capacity(len as usize);

    let bytes_read = file.read_at(block, cache, &mut buffer, len, 0)?;

    assert_eq!(bytes_read, len);

    let mut read_bits = cast_vec::<u8, ElementType>(buffer);
    read_bits[(element_cnt(self.bit_cnt) - 1) as usize] &= last_mask(self);
    self.inner.replace(read_bits);

    Ok(())
  }
}
