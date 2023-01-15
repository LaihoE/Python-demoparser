use crate::parsing::parser::Parser;
use memmap2::Mmap;
use std::sync::Arc;
use varint_simd::{decode, decode_two_unsafe, encode, encode_zigzag};

pub struct ByteReader {
    pub bytes: Arc<Mmap>,
    pub byte_idx: usize,
    pub single: bool,
}

impl ByteReader {
    pub fn new(bytes: Arc<Mmap>, single: bool, start_idx: usize) -> Self {
        ByteReader {
            bytes: bytes,
            byte_idx: start_idx,
            single: single,
        }
    }

    #[inline]
    pub fn read_varint(&mut self) -> u32 {
        let mut result: u32 = 0;
        let mut count: u8 = 0;
        let mut b: u32;

        loop {
            if count >= 5 {
                return result as u32;
            }
            b = self.bytes[self.byte_idx].try_into().unwrap();
            self.byte_idx += 1;
            result |= (b & 127) << (7 * count);
            count += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        result as u32
    }
    #[inline]
    pub fn read_short(&mut self) -> u16 {
        let s = u16::from_le_bytes(
            self.bytes[self.byte_idx..self.byte_idx + 2]
                .try_into()
                .unwrap(),
        );
        self.byte_idx += 2;
        s
    }
    #[inline]
    pub fn read_string(&mut self) -> String {
        let mut v = vec![];
        loop {
            let c = self.read_byte();
            if c != 0 {
                v.push(c)
            } else {
                break;
            }
        }
        let s = String::from_utf8_lossy(&v);
        s.to_string()
    }
    #[inline]
    pub fn read_i32(&mut self) -> i32 {
        let i = i32::from_le_bytes(
            self.bytes[self.byte_idx..self.byte_idx + 4]
                .try_into()
                .unwrap(),
        );
        self.byte_idx += 4;
        i
    }
    #[inline]
    pub fn read_byte(&mut self) -> u8 {
        let b = self.bytes[self.byte_idx];
        self.byte_idx += 1;
        b
    }
    #[inline]
    pub fn skip_n_bytes(&mut self, n: u32) {
        self.byte_idx += n as usize;
    }
    #[inline]
    pub fn read_n_bytes(&mut self, n: u32) -> &[u8] {
        let s = &self.bytes[self.byte_idx..self.byte_idx + n as usize];
        self.byte_idx += n as usize;
        s
    }
    #[inline(always)]
    pub fn read_frame(&mut self) -> (u8, i32) {
        let cmd = self.read_byte();
        let tick = self.read_i32();
        self.skip_n_bytes(1);
        (cmd, tick)
    }
    #[inline(always)]
    pub fn read_two_varints(&mut self) -> (u32, u32) {
        let max_two = self.bytes[self.byte_idx..self.byte_idx + 10].as_ptr();
        unsafe {
            {
                let (a, b, one, two) = decode_two_unsafe(max_two);
                self.byte_idx += one as usize;
                self.byte_idx += two as usize;
                return (a, b);
            }
        }
    }
}
