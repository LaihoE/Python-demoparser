use crate::Demo;

impl Demo {
    pub fn read_varint(&mut self) -> u32 {
        let mut result: u32 = 0;
        let mut count: i32 = 0;
        let mut b: u32;

        loop {
            if count >= 5 {
                return result.try_into().unwrap();
            }
            b = self.bytes[self.fp].try_into().unwrap();
            self.fp += 1;
            result |= (b & 127) << (7 * count);
            count += 1;
            if !(b & 0x80 != 0) {
                break;
            }
        }
        return result.try_into().unwrap();
    }
    pub fn read_short(&mut self) -> u16 {
        let s = u16::from_le_bytes(self.bytes[self.fp..self.fp + 2].try_into().unwrap());
        self.fp += 2;
        s
    }
    pub fn read_string(&mut self) -> String {
        // SLOW?
        let mut v = vec![];
        loop {
            let c = self.read_byte();
            if c != 0 {
                v.push(c)
            } else {
                break;
            }
        }
        let s = String::from_utf8(v).unwrap();
        s
    }
    pub fn read_i32(&mut self) -> i32 {
        let i = i32::from_le_bytes(self.bytes[self.fp..self.fp + 4].try_into().unwrap());
        self.fp += 4;
        i
    }
    pub fn read_byte(&mut self) -> u8 {
        let b = self.bytes[self.fp];
        self.fp += 1;
        b
    }

    pub fn read_n_bytes(&mut self, n: u32) -> &[u8] {
        let s = &self.bytes[self.fp..self.fp + n as usize];
        self.fp += n as usize;
        s
    }
}
