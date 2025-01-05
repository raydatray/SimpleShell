use std::cmp;

/// Dumps the `size` bytes in `buf` to the console as hex bytes
/// arranged 16 per line. Numeric offsets are also included,
/// starting at `ofs` for the first byte in `buf`. If `ascii` is true
/// then the corresponding ASCII characters are also rendered
/// alongside.
pub fn hex_dump(ofs: usize, buf: &[u8], size: usize, ascii: bool) {
  const PER_LINE: usize = 16; // Maximum bytes per line.

  let mut remaining = size;
  let mut buf_offset = 0;
  let mut ofs = ofs;

  while remaining > 0 {
    // Number of bytes on this line.
    let start = ofs % PER_LINE;
    let end = cmp::min(PER_LINE, start + remaining);
    let n = end - start;

    // Print line.
    print!("{:08x}  ", ofs - start);

    for i in 0..PER_LINE {
      if i < start {
        print!("   ");
      } else if i < end {
        print!("{:02x}{}", buf[buf_offset + i - start], if i == PER_LINE / 2 - 1 { '-' } else { ' ' });
      } else {
        break;
      }
    }

    if ascii {
      for _ in end..PER_LINE {
        print!("   ");
      }
      print!("|");
      for i in 0..PER_LINE {
        if i < start {
          print!(" ");
        } else if i < end {
          let c = buf[buf_offset + i - start];
          print!("{}", if c.is_ascii_graphic() { c as char } else { '.' });
        } else {
          print!(" ");
        }
      }
      print!("|");
    }
    println!();

    ofs += n;
    buf_offset += n;
    remaining -= n;
  }
}
