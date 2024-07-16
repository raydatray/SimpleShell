use std::{fs::File, mem, os::unix::fs::FileExt};

type ElementType = u32;
const ELEMENT_BITS: usize = mem::size_of::<ElementType>() * u8::BITS as usize; //How many bits in ElemType

#[inline]
//Returns the idx of the element that contains the bit numbered bit_idx
pub fn element_idx(bit_idx: usize) -> usize {
  bit_idx / ELEMENT_BITS
}

#[inline]
//Returns an ElementType where only bit_idx is 1
pub fn bit_mask(bit_idx: usize) -> ElementType {
  1 << (bit_idx % ELEMENT_BITS)
}

#[inline]
//Returns the number of elements required for bit_cnt bits
pub fn element_cnt(bit_cnt: usize) -> usize {
  (bit_cnt + ELEMENT_BITS - 1) / ELEMENT_BITS
}

#[inline]
pub fn byte_cnt(bit_cnt: usize) -> usize {
  mem::size_of::<ElementType>() * element_cnt(bit_cnt)
}

#[inline]
pub fn last_mask(bitmap: &Bitmap) -> ElementType {
  let last_bits = bitmap.bit_cnt % ELEMENT_BITS;
  return match last_bits {
    0 => ElementType::MAX,
    _ => (1 << last_bits) - 1
  }
}

struct Bitmap {
  bit_cnt: usize, //size_t in original implementation. usize is the most similar type
  bits: Vec<ElementType>
}

impl Bitmap {
  //Init a new bitmap with all bits set as 0
  fn new(bit_cnt: usize) -> Bitmap {
    let element_count = element_cnt(bit_cnt);
    Bitmap {
      bit_cnt,
      bits: vec![0; element_count]
    }
  }




  fn bitmap_file_size(&self) -> usize {
    byte_cnt(self.bit_cnt)
  }

  pub fn read_bitmap_from_file(&mut self,) {
    if self.bit_cnt > 0 {
      let size = byte_cnt(self.bit_cnt);


    }
  }
}
