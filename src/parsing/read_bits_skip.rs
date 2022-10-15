use std::collections::HashMap;

use crate::parsing::entities::Prop;
use crate::parsing::variants::PropData;
use core::panic;
use std::convert::TryInto;
use std::io;
use std::mem;
use std::u32;

const NBITS: usize = 32;

static MASKS: [u32; NBITS + 1] = [
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

pub struct BitReader<R: io::Read> {
    inner: R,
    bits: u32,
    available: usize,
    left_over: i32,
}

impl<R: io::Read> BitReader<R> {
    pub fn new(reader: R, left_over: i32) -> BitReader<R> {
        let b = BitReader {
            inner: reader,
            bits: 0,
            available: 0,
            left_over: left_over,
        };
        b
    }

    pub fn read_uneven_end_bits(&mut self) {
        match self.left_over {
            0 => {}
            1 => {
                let mut buf = [0; 1];
                self.inner.read_exact(&mut buf).unwrap();
                self.available = 8;
                self.bits = u32::from(buf[0]);
            }
            2 => {
                let mut buf = [0; 16 / 8];
                self.inner.read_exact(&mut buf).unwrap();
                self.available = 16;
                self.bits = u32::from(buf[0]) + (u32::from(buf[1]) << 8);
            }
            3 => {
                let mut buf = [0; 24 / 8];
                self.inner.read_exact(&mut buf).unwrap();
                self.available = 24;
                self.bits =
                    u32::from(buf[0]) + (u32::from(buf[1]) << 8) + (u32::from(buf[2]) << 16);
            }
            _ => panic!("UNK LEFT OVER"),
        }
    }

    #[inline(always)]
    pub fn ensure_bits(&mut self) {
        let mut buf = [0; NBITS / 8];
        let had_enough = self.inner.read_exact(&mut buf);
        self.bits = unsafe { mem::transmute(buf) };
        self.available = NBITS;
    }
    #[inline(always)]
    pub fn consume(&mut self, n: usize) {
        self.bits = match n {
            NBITS => 0,
            n => self.bits >> n,
        };
        self.available -= n;
    }
    #[inline(always)]
    pub fn read_bool(&mut self) -> bool {
        if self.available == 0 {
            self.ensure_bits();
        }
        let ret = self.bits & 1 == 1;
        self.consume(1);
        ret
    }
    #[inline(always)]
    pub fn read_nbits(&mut self, n: usize) -> u32 {
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
    #[inline(always)]
    pub fn skip_nbits(&mut self, n: usize) {
        if self.available >= n {
            //let ret = self.bits & MASKS[n];
            self.consume(n);
            //ret
        } else {
            let in_buf = self.bits;
            let consumed = self.available;
            let remaining = n - self.available;
            self.ensure_bits();
            //let ret = in_buf | ((self.bits & MASKS[remaining]) << consumed);
            self.consume(remaining);
            //ret.to_le()
        }
    }

    #[inline(always)]
    pub fn read_inx(&mut self, last: i32, new_way: bool) -> i32 {
        if new_way && self.read_bool() {
            return last + 1;
        }
        if new_way && self.read_bool() {
            let index = self.read_nbits(3);
            if index == 0xfff {
                return -1;
            }
            return last + 1 + index as i32;
        } else {
            let mut index = self.read_nbits(7);
            let val = index & (32 | 64);
            match val {
                32 => index = (index & !96) | (self.read_nbits(2) << 5),
                64 => index = (index & !96) | (self.read_nbits(4) << 5),
                96 => index = (index & !96) | (self.read_nbits(7) << 5),
                _ => {}
            }
            if index == 0xfff {
                return -1;
            }
            return last + 1 + index as i32;
        }
    }
    #[inline(always)]
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
            if b & 0x80 == 0 {
                break;
            }
        }
        result
    }

    #[inline(always)]
    pub fn skip_string(&mut self) {
        let mut length = self.read_nbits(9);
        if length == 0 {
            return;
        }
        if length >= (1 << 9) {
            length = (1 << 9) - 1
        }
        self.read_string(length.try_into().unwrap());
    }
    #[inline(always)]
    pub fn skip_decode(&mut self, prop: &Prop) {
        match prop.p_type {
            //1 => PropData::F32(self.decode_float(prop)),
            1 => self.skip_float(prop),
            0 => self.skip_int(prop),
            3 => self.skip_vec_xy(prop),
            2 => self.skip_vec(prop),
            4 => self.skip_string(),
            5 => self.skip_array(prop),
            _ => panic!("EEK"),
        }
    }

    pub fn skip_array(&mut self, prop: &Prop) {
        let b = (prop.num_elements as f32).log2().floor() + 1.0;
        let num_elements = self.read_nbits(b as usize);

        let p = prop.arr.as_ref().unwrap();
        let mut elems = vec![];

        for _ in 0..num_elements {
            let pro = Prop {
                name: p.to_string(),
                arr: None,
                col: 0,
                data: None,
                flags: p.flags(),
                num_elements: p.num_elements(),
                num_bits: p.num_bits(),
                low_value: p.high_value(),
                high_value: p.high_value(),
                priority: p.priority(),
                p_type: p.type_(),
            };
            let val = self.skip_decode(&pro);
            elems.push(val);
        }
    }
    #[inline(always)]
    pub fn read_string(&mut self, length: i32) -> String {
        let mut s: Vec<u8> = Vec::new();
        let mut inx = 1;
        loop {
            let c = self.read_sint_bits(8) as u8;
            if c == 0 {
                break;
            }
            s.push(c);
            if inx == length {
                break;
            }
            inx += 1;
        }
        let out = String::from_utf8_lossy(&s);
        out.try_into().unwrap()
    }
    #[inline(always)]
    pub fn read_string_lossy(&mut self, length: i32) -> String {
        let mut s: Vec<u8> = Vec::new();
        let mut inx = 1;
        loop {
            let c = self.read_sint_bits(8) as u8;
            if c == 0 {
                break;
            }
            s.push(c);
            if inx == length {
                break;
            }
            inx += 1;
        }
        let out = String::from_utf8_lossy(&s);
        out.to_string()
    }
    #[inline(always)]
    pub fn skip_vec(&mut self, prop: &Prop) {
        let x = self.skip_float(prop);
        let y = self.skip_float(prop);
        if prop.flags & (1 << 5) == 0 {
            let z = self.skip_float(prop);
        } else {
            let sign = self.read_bool();
        }
    }
    #[inline(always)]
    pub fn skip_vec_xy(&mut self, prop: &Prop) {
        self.skip_float(prop);
        self.skip_float(prop);
    }
    #[inline(always)]
    pub fn read_sint_bits(&mut self, n: i32) -> u32 {
        self.read_nbits(n.try_into().unwrap()) << (32 - n) >> (32 - n)
    }
    #[inline(always)]
    pub fn skip_bit_cell_coord(&mut self, n: usize, coord_type: u32) {
        let frac_bits = 0;
        let resolution = 0;
        let low_prec = coord_type == 1;
        let result = 0;
        if coord_type == 2 {
            let result = self.skip_nbits(n);
        } else {
            if coord_type == 3 {
                let frac_bits = low_prec;
            } else {
                let frac_bits = 5;
            }
            self.skip_nbits(n);
            self.skip_nbits(frac_bits);
        }
    }
    #[inline(always)]
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
    #[inline(always)]
    pub fn skip_bit_coord(&mut self) {
        let i2 = self.read_bool();
        let f2 = self.read_bool();

        if i2 == false && f2 == false {
            return;
        }
        let sign = self.read_bool();
        if i2 {
            self.skip_nbits(14);
        }
        if f2 {
            self.skip_nbits(5);
        }
    }
    #[inline(always)]
    pub fn read_bits(&mut self, n: i32) -> f32 {
        let mut bitsleft = n;
        let eight = 8 as usize;
        let mut bytarr: [u8; 4] = [0, 0, 0, 0];
        while bitsleft >= 32 {
            bytarr[0] = self.read_nbits(eight).try_into().unwrap();
            bytarr[1] = self.read_nbits(eight).try_into().unwrap();
            bytarr[2] = self.read_nbits(eight).try_into().unwrap();
            bytarr[3] = self.read_nbits(eight).try_into().unwrap();
            bitsleft -= 32;
        }
        f32::from_le_bytes(bytarr)
    }
    #[inline(always)]
    pub fn read_bits_st(&mut self, n: i32) -> [u8; 340] {
        let mut res = 0;
        let mut bitsleft = n;
        let eight = 8.try_into().unwrap();
        let mut bytarr: [u8; 340] = [0; 340];
        for i in 0..340 {
            bytarr[i] = self.read_nbits(eight).try_into().unwrap();
        }
        bytarr
    }
    #[inline(always)]
    pub fn decode_special_float(&mut self, prop: &Prop) -> f32 {
        if prop.flags & (1 << 2) != 0 {
            self.read_nbits(32) as f32;
        } else if prop.flags & (1 << 1) != 0 {
            self.read_bit_coord() as f32;
        } else if prop.flags & (1 << 5) != 0 {
            self.read_bit_normal() as f32;
        } else if prop.flags & (1 << 15) != 0 {
            self.read_bit_cell_coord(prop.num_bits as usize, 0) as f32;
        } else if prop.flags & (1 << 16) != 0 {
            self.read_bit_cell_coord(prop.num_bits as usize, 1) as f32;
        } else if prop.flags & (1 << 17) != 0 {
            self.read_bit_cell_coord(prop.num_bits as usize, 2) as f32;
        }
        0.0
    }
    /*
    #[inline(always)]
    pub fn decode_float(&mut self, prop: &Prop) -> f32 {
        let val = self.decode_special_float(prop);
        if val != 0.0 {
            return val as f32;
        } else {
            let mut interp = 1;
            if prop.num_bits != -1 {
                interp = self.read_nbits(prop.num_bits as usize);
            }
            let mut val = (interp / (1 << prop.num_bits - 1)) as f32;
            val = prop.low_value + (prop.high_value - prop.low_value) * (val as f32);
            val
        }
    }
    */
    #[inline(always)]
    pub fn skip_float(&mut self, prop: &Prop) {
        let val = self.decode_special_float(prop);
        if val != 0.0 {
            return;
        } else {
            if prop.num_bits != -1 {
                self.skip_nbits(prop.num_bits as usize);
            }
        }
    }

    #[inline(always)]
    pub fn skip_int(&mut self, prop: &Prop) {
        let flags = prop.flags;
        if flags & (1 << 19) != 0 {
            if flags & (1 << 0) != 0 {
                let result: i32 = self.read_varint().try_into().unwrap();
            } else {
                let mut result = self.read_varint();
            }
        } else {
            if flags & (1 << 0) != 0 {
                if prop.num_bits == 1 {
                    self.skip_nbits(1);
                } else {
                    self.skip_nbits(prop.num_bits as usize);
                }
            } else {
                // WTF
                let result = self.read_sbit_long(prop.num_bits.try_into().unwrap());
            }
        }
    }
    #[inline(always)]
    pub fn read_sbit_long(&mut self, numbits: u32) -> i32 {
        let nret = self.read_nbits(numbits.try_into().unwrap()) as i32;
        return (nret << (32 - numbits)) >> (32 - numbits);
    }

    #[inline(always)]
    pub fn read_u_bit_var(&mut self) -> u32 {
        let mut ret = self.read_nbits(6);
        //println!("{} {}", self.available, ret);
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

    #[inline(always)]
    pub fn decode_string(&mut self) -> String {
        let mut length = self.read_nbits(9);
        if length == 0 {
            return "".to_string();
        }
        if length >= (1 << 9) {
            length = (1 << 9) - 1
        }
        self.read_string(length.try_into().unwrap())
    }
    #[inline(always)]
    pub fn decode(&mut self, prop: &Prop) -> PropData {
        match prop.p_type {
            1 => PropData::F32(self.decode_float(prop)),
            0 => PropData::I32(self.decode_int(prop) as i32),
            3 => PropData::VecXY(self.decode_vec_xy(prop)),
            2 => PropData::VecXYZ(self.decode_vec(prop)),
            4 => PropData::String(self.decode_string()),
            5 => PropData::Vec(self.decode_array(prop)),
            _ => panic!("EEK"),
        }
    }

    pub fn decode_array(&mut self, prop: &Prop) -> Vec<i32> {
        let b = (prop.num_elements as f32).log2().floor() + 1.0;
        let num_elements = self.read_nbits(b as usize);

        let p = prop.arr.as_ref().unwrap();
        let mut elems = vec![];

        for _ in 0..num_elements {
            let pro = Prop {
                name: p.to_string(),
                arr: None,
                col: 0,
                data: None,
                flags: p.flags(),
                num_elements: p.num_elements(),
                num_bits: p.num_bits(),
                low_value: p.high_value(),
                high_value: p.high_value(),
                priority: p.priority(),
                p_type: p.type_(),
            };
            let val = self.decode(&pro);
            elems.push(val);
        }
        vec![0, 0, 0]
    }

    #[inline(always)]
    pub fn decode_vec(&mut self, prop: &Prop) -> [f32; 3] {
        let x = self.decode_float(prop);
        let y = self.decode_float(prop);
        if prop.flags & (1 << 5) == 0 {
            let z = self.decode_float(prop);
            return [x, y, z];
        } else {
            let sign = self.read_bool();
            let temp = (x * x) + (y * y);
            let mut z = 0.0;
            if temp < 1.0 {
                z = (1.0 - temp).sqrt();
            }
            if sign {
                return [x, y, -z];
            }
            [x, y, z]
        }
    }
    #[inline(always)]
    pub fn decode_vec_xy(&mut self, prop: &Prop) -> [f32; 2] {
        let x = self.decode_float(prop);
        let y = self.decode_float(prop);
        let v = [x, y];
        v
    }

    #[inline(always)]
    pub fn read_bit_cell_coord(&mut self, n: usize, coord_type: u32) -> u32 {
        // SKIP FOR NOW, WATCH OUT
        match coord_type {
            2 => {
                let _ = self.read_nbits(n);
                return 0;
            }
            _ => {
                let frac_bits = if coord_type == 3 { 1 } else { 5 };
                self.read_nbits(frac_bits);
                return 0;
            }
        }
    }

    #[inline(always)]
    pub fn read_bit_coord(&mut self) -> i32 {
        let mut int_val = 0;
        let mut frac_val = 0;

        let i2 = self.read_bool();
        let f2 = self.read_bool();

        if i2 == false && f2 == false {
            return 0;
        }
        let sign = self.read_bool();
        if i2 {
            int_val = self.read_nbits(14) + 1;
        }
        if f2 {
            frac_val = self.read_nbits(5);
        }
        let resol: f64 = 1.0 / (1 << 5) as f64;
        let result: i32 = (int_val as f64 + (frac_val as f64 * resol) as f64) as i32;
        if sign {
            return -result;
        } else {
            result
        }
    }

    #[inline(always)]
    pub fn decode_float(&mut self, prop: &Prop) -> f32 {
        let val = self.decode_special_float(prop);
        if val != 0.0 {
            return 0.0 as f32;
        } else {
            let mut interp = 1;
            if prop.num_bits != -1 {
                interp = self.read_nbits(prop.num_bits as usize);
            }
            let mut val = (interp / (1 << prop.num_bits - 1)) as f32;
            val = prop.low_value + (prop.high_value - prop.low_value) * (val as f32);
            val
        }
    }

    #[inline(always)]
    pub fn decode_int(&mut self, prop: &Prop) -> u32 {
        let flags = prop.flags;
        if flags & (1 << 19) != 0 {
            if flags & (1 << 0) != 0 {
                let result: i32 = self.read_varint().try_into().unwrap();
                result as u32
            } else {
                let mut result = self.read_varint();
                result = (result >> 1) ^ (!(result & 1));
                result
            }
        } else {
            if flags & (1 << 0) != 0 {
                if prop.num_bits == 1 {
                    let result = self.read_nbits(1);
                    result as u32
                } else {
                    let result: u32 = self.read_nbits(prop.num_bits as usize);
                    result as u32
                }
            } else {
                // WTF
                let result = self.read_sbit_long(prop.num_bits.try_into().unwrap());
                result as u32 //.try_into().unwrap()
            }
        }
    }
}
