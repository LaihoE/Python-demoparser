use std::convert::TryFrom;
use std::convert::TryInto;

use crate::Demo;

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
    buffer: u32,
}

impl<'a> Bitr<'a> {
    pub fn new(&mut self, data: &'a [u8]) -> Bitr<'a> {
        let mut bits_avail = 0;
        let mut bytepos = 0;
        let mut buffer = 0;
        let left_over = data.len() % 4;
        let size_type = 4;

        if data.len() > size_type && left_over != 0 {
            bytepos = left_over - size_type;
            bits_avail = left_over * 8;
            buffer = (self.buffer << (size_type * 8 - bits_avail) >> (size_type * 8 - bits_avail));
        } else if data.len() <= 8 {
            bits_avail = data.len() * 8;
        } else {
            bits_avail = size_type * 8;
        }

        Bitr {
            data: data,
            bits_avail: bits_avail.try_into().unwrap(),
            bytepos: bytepos,
            bytes_len: data.len(),
            buffer: buffer,
        }
    }

    pub fn consume_n(&mut self, n: u32) {
        self.bits_avail -= n;
        self.buffer >>= n;
    }

    pub fn fill_buffer_32(&mut self) {
        // Read 4 bytes into buffer (32 bits)
        self.bits_avail = 32;
        self.buffer = self.data[self.bytepos] as u32
            + (self.data[self.bytepos + 1] << 8) as u32
            + (self.data[self.bytepos + 2] << 16) as u32
            + (self.data[self.bytepos + 3] << 24) as u32;
    }

    pub fn read_bits(&mut self, n: u32) -> u32 {
        // If we have enough bits to fulfill request
        if self.bits_avail <= n {
            let result = self.buffer & MASKS[n as usize];
            self.consume_n(n);
            result
        } else {
            // First read current buffer empty and then refill with new bits and read rest
            let mut result = self.buffer;
            self.consume_n(self.bits_avail);

            let bits_still_needed = n - self.bits_avail;
            self.fill_buffer_32();
            result |= self.buffer & MASKS[n as usize];
            self.bits_avail -= bits_still_needed;
            return result.to_le();
        }
    }
}
