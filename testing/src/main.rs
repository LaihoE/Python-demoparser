use bitreader::BitReader;
use std::convert::TryFrom;
use std::convert::TryInto;
static MASKS: [u32; 32 + 1] = [
    0,
    u32::MAX >> 31,
    u32::MAX >> 30,
    u32::MAX >> 29,
    u32::MAX >> 28,
    u32::MAX >> 27,
    u32::MAX >> 26,
    u32::MAX >> 25,
    u32::MAX >> 24,
    u32::MAX >> 23,
    u32::MAX >> 22,
    u32::MAX >> 21,
    u32::MAX >> 20,
    u32::MAX >> 19,
    u32::MAX >> 18,
    u32::MAX >> 17,
    u32::MAX >> 16,
    u32::MAX >> 15,
    u32::MAX >> 14,
    u32::MAX >> 13,
    u32::MAX >> 12,
    u32::MAX >> 11,
    u32::MAX >> 10,
    u32::MAX >> 9,
    u32::MAX >> 8,
    u32::MAX >> 7,
    u32::MAX >> 6,
    u32::MAX >> 5,
    u32::MAX >> 4,
    u32::MAX >> 3,
    u32::MAX >> 2,
    u32::MAX >> 1,
    u32::MAX,
];

pub struct Bitr<'a> {
    data: &'a [u8],
    bits_avail: u32,
    bytepos: usize,
    bytes_len: usize,
    buffer: BitReader<'a>,
}

impl<'a> Bitr<'a> {
    pub fn new(data: &'a [u8]) -> Bitr<'a> {
        let mut bits_avail = 0;
        let mut bytepos = 0;
        let mut buffer = BitReader::new(data);
        let left_over = data.len() % 4;
        let size_type = 4;

        if data.len() > size_type && left_over != 0 {
            bytepos = left_over - size_type;
            bits_avail = left_over * 8;
            //buffer = (buffer << (size_type * 8 - bits_avail) >> (size_type * 8 - bits_avail));
        } else if data.len() <= 8 {
            bits_avail = data.len() * 8;
        } else {
            bits_avail = size_type * 8;
        }
        println!("{}", bits_avail);
        Bitr {
            data: data,
            bits_avail: bits_avail.try_into().unwrap(),
            bytepos: bytepos,
            bytes_len: data.len(),
            buffer: buffer,
        }
    }

    pub fn read_bits(&mut self, n: u32) -> u32 {
        // If we have enough bits to fulfill request
        if self.bits_avail >= n {
            let temp = self.buffer.read_u32(n.try_into().unwrap()).unwrap();
            let result = temp & MASKS[12];
            //self.buffer.read_u8(n.try_into().unwrap());
            self.bits_avail -= n;
            result
        } else {
            // First read current buffer empty and then refill with new bits and read rest
            let bits_still_needed = n - self.bits_avail;

            let mut pre = self
                .buffer
                .read_u32(self.bits_avail.try_into().unwrap())
                .unwrap();

            self.buffer.read_u32(bits_still_needed.try_into().unwrap());
            println!("{} {}", bits_still_needed, self.bits_avail);

            self.bits_avail = 32 - bits_still_needed;
            let rest = self.buffer.read_u32(32).unwrap();
            pre |= rest & MASKS[n as usize];
            return pre;
        }
    }
}

use std::any::Any;
use std::io;
use std::mem;
use std::u32;

const NBITS: usize = 32;

pub(crate) struct poop<R: io::Read> {
    inner: R,
    bits: u32,
    available: usize,
}

impl<R: io::Read> poop<R> {
    pub fn new(reader: R) -> poop<R> {
        poop {
            inner: reader,
            bits: 0,
            available: 0,
        }
    }

    pub fn ensure_bits(&mut self) -> io::Result<()> {
        let mut buf = [0; NBITS / 8];
        self.inner.read_exact(&mut buf)?;
        self.bits = unsafe { mem::transmute(buf) };
        self.available = NBITS;
        Ok(())
    }

    pub fn consume(&mut self, n: usize) {
        self.bits = match n {
            NBITS => 0,
            n => self.bits >> n,
        };
        self.available -= n;
    }

    pub fn read_bool(&mut self) -> bool {
        if self.available == 0 {
            self.ensure_bits();
        }
        let ret = self.bits & 1 == 1;
        self.consume(1);
        ret
    }

    pub fn read_nbits(&mut self, n: usize) -> u32 {
        debug_assert!(n <= NBITS);

        if self.available >= n {
            let ret = self.bits & MASKS[n];
            self.consume(n);
            ret
        } else {
            let in_buf = self.bits;
            let consumed = self.available;
            let remaining = n - consumed;
            self.ensure_bits();
            let ret = in_buf | ((self.bits & MASKS[remaining]) << consumed);
            self.consume(remaining);
            ret.to_le()
        }
    }

    pub fn read_u_bit_var(&mut self) -> u32 {
        let mut ret = self.read_nbits(6);
        if ret & 48 == 16 {
            ret = (ret & 15) | (self.read_nbits(4) << 4);
            assert!(ret >= 16);
        } else if ret & 48 == 32 {
            ret = (ret & 15) | (self.read_nbits(8) << 4);
            assert!(ret >= 256);
        } else if ret & 48 == 48 {
            ret = (ret & 15) | (self.read_nbits(28) << 4);
            assert!(ret >= 4096);
        }
        ret
    }

    pub fn read_inx(&mut self, last: i32, new_way: bool) -> i32 {
        let mut ret = 0;
        let mut val = 0;

        if new_way && self.read_bool() {
            return last + 1;
        }
        if new_way && self.read_bool() {
            ret = self.read_nbits(3);
        } else {
            ret = self.read_nbits(7);
            val = ret & (32 | 64);
            match val {
                32 => ret = (ret & !96) | (self.read_nbits(2) << 5),
                64 => ret = (ret & !96) | (self.read_nbits(4) << 5),
                96 => ret = (ret & !96) | (self.read_nbits(7) << 5),
                _ => {}
            }
        }
        if ret == 0xfff {
            return -1;
        }
        let y: i32 = ret.try_into().unwrap();
        return last + 1 + y;
    }

    pub fn read_varint(&mut self) -> u32 {
        let mut result: u32 = 0;
        let mut count: i32 = 0;
        let mut b: u32;

        loop {
            if count >= 5 {
                return result.try_into().unwrap();
            }
            b = self.read_nbits(8);
            result |= (b & 127) << (7 * count);
            count += 1;
            if !(b & 0x80 != 0) {
                break;
            }
        }
        return result.try_into().unwrap();
    }

    pub fn skip(&mut self, nbits: i32) -> f32 {
        self.read_nbits(nbits.try_into().unwrap());
        0.0
    }

    pub fn read_bit_coord_mp(&mut self, coord_type: u32) -> f64 {
        let mut result = 0.0;
        let ret = 0;
        let sign = false;
        let integral = (coord_type == 2);
        let low_pres = (coord_type == 1);
        let mut in_bounds = false;
        if self.read_bool() {
            in_bounds = true;
        } else {
            in_bounds = true;
        }

        if integral {
            let int_val = self.read_bool();
            if int_val {
                let sign = self.read_bool();
                if in_bounds {
                    let result = self.read_nbits(11) + 1;
                } else {
                    let result = self.read_nbits(14) + 1;
                }
            }
        } else {
            let int_val = self.read_bool();
            let sign = self.read_bool();
            if int_val {
                if in_bounds {
                    let int_val = self.read_nbits(11) + 1;
                } else {
                    let int_val = self.read_nbits(14) + 1;
                }
            }
            if low_pres {
                let lp = (1.0 / (1 << 3) as f64);
                let frac_val = self.read_nbits(3);
                let result = int_val as i32 as f64 + frac_val as f64 * lp;
            } else {
                let cr: f64 = (1.0 / (1 << 5) as f64);
                let frac_val = self.read_nbits(5);
                let result = int_val as i32 as f64 + frac_val as f64 * cr;
            }
        }
        if sign {
            -result
        } else {
            result
        }
    }

    pub fn read_bit_cell_coord(&mut self, n: usize, coord_type: u32) -> u32 {
        let mut frac_bits = 0;
        let mut resolution = 0;
        let low_prec = (coord_type == 1);
        let mut result = 0;
        if coord_type == 2 {
            let result = self.read_nbits(n);
        } else {
            if coord_type == 3 {
                let frac_bits = low_prec;
            } else {
                let frac_bits = 5;
            }
            if low_prec {
                let resolution = (1.0 / (1 << 3) as f64);
            } else {
                let cr: f64 = (1.0 / (1 << 5) as f64);
            }

            let int_val = self.read_nbits(n);
            let frac_val = self.read_nbits(frac_bits);
            let result = int_val + (frac_val * resolution);
        }
        return result;
    }

    pub fn read_bit_normal(&mut self) -> f64 {
        let sign = self.read_bool();
        let frac = self.read_nbits(11);
        let result = frac as f64 * (1.0 / ((1 << 11) - 1) as f64);
        if sign {
            -result
        } else {
            result
        }
    }
}

fn main() {
    let slice_of_u8: &[u8] = &[17, 34, 4, 8, 9];
    // println!("{:?}", slice_of_u8);

    // You probably should use try! or some other error handling mechanism in real code if the
    // length of the input is not known in advance.
    //let x = reader.read_u32(12).unwrap();
    let mut poop = poop::new(slice_of_u8);
    //println!("{}", a_single_bit);

    //let slice_of_u8: &[u8] = &[17, 34];

    //let mut b = Bitr::new(slice_of_u8);
    //let mut bb = BitReader::new(slice_of_u8);
    //bb.ensure_bits();
    // let x = b.read_bits(13);
    // let x = bb.read_nbits(4);
    // let x = bb.read_nbits(32);
    // let x = bb.read_nbits(8);
    // let x = b.read_bits(8);
    // let x = poop.read_nbits(32);
    let x = poop.read_nbits(8);
    println!("{:?}", x.to_le());
}
