use std::cmp;

pub struct BitBuffer {
    pub data: Vec<u8>,
    pub databytes: i32,
    pub databytes_len: usize,
    pub data_part: u8,
    pub bits_free: u32,
    pub pos_byte: usize,
    pub overflow: bool,
}

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
#[allow(arithmetic_overflow)]
impl BitBuffer {
    #[allow(arithmetic_overflow)]
    pub fn startup(&mut self) {
        let left_over: usize = self.databytes_len % 4;
        let head = left_over as u32;

        if self.databytes < 4 || head > 0 {
            if head > 2 {
                self.data_part = (self.data[0] + (self.data[1] << 8) + (self.data[2] << 16));
                self.pos_byte = 3;
            } else if head > 1 {
                self.data_part = self.data[0] + (self.data[1] << 8);
                self.pos_byte = 2;
            } else {
                self.data_part = self.data[0];
                self.pos_byte = 1;
            }
            self.bits_free = head << 3;
        } else {
            self.pos_byte = head as usize;
            self.data_part = (self.data[self.pos_byte]
                + (self.data[self.pos_byte + 1] << 8)
                + (self.data[self.pos_byte + 2] << 16)
                << (self.data[self.pos_byte + 3] << 24));
            if self.data.len() > 0 {
                self.fetch_next();
            } else {
                self.data_part = 0;
                self.bits_free = 1;
            }
            self.bits_free = cmp::min(self.bits_free, 32);
        }
    }
    #[allow(arithmetic_overflow)]
    pub fn read_bit(&mut self) -> u8 {
        let abit = self.data_part & 1;
        self.bits_free -= 1;
        if self.bits_free == 0 {
            self.fetch_next();
        } else {
            self.data_part >>= 1;
        }
        abit
    }
    #[allow(arithmetic_overflow)]
    pub fn grab_next_4_bytes(&mut self) {
        if self.pos_byte >= self.data.len() {
            self.bits_free = 1;
            self.data_part = 0;
            self.overflow = true;
        } else {
            self.data_part = (self.data[self.pos_byte]
                + (self.data[self.pos_byte + 1] << 8)
                + (self.data[self.pos_byte + 2] << 16)
                << (self.data[self.pos_byte + 3] << 24));

            self.pos_byte += 4;
        }
    }
    #[allow(arithmetic_overflow)]
    pub fn fetch_next(&mut self) {
        self.bits_free = 32;
        self.grab_next_4_bytes();
    }
    #[allow(arithmetic_overflow)]
    pub fn read_u_bit_var(&mut self) -> u8 {
        let mut ret = self.read_uint_bits(6);
        if ret & 48 == 16 {
            ret = (ret & 15) | (self.read_uint_bits(4) << 4);
            assert!(ret >= 16);
        } else if ret & 48 == 32 {
            ret = (ret & 15) | (self.read_uint_bits(8) << 4);
        } else if ret & 48 == 48 {
            ret = (ret & 15) | (self.read_uint_bits(28) << 4);
        }
        ret
    }
    #[allow(arithmetic_overflow)]
    pub fn read_uint_bits(&mut self, mut a_bits: u32) -> u8 {
        if self.bits_free >= a_bits {
            let res = self.data_part & MASKS[a_bits as usize] as u8;
            self.bits_free -= a_bits;

            if self.bits_free == 0 {
                self.fetch_next();
            } else {
                self.data_part >>= a_bits;
            }
            res
        } else {
            let mut res = self.data_part;
            a_bits -= self.bits_free;
            let t_bits_free = self.bits_free;
            self.fetch_next();
            if self.overflow {
                return 0;
            }
            res |=
                (self.data_part & (self.data_part & MASKS[a_bits as usize] as u8) << t_bits_free);
            self.bits_free -= a_bits;
            self.data_part >>= a_bits;
            res
        }
    }
}
