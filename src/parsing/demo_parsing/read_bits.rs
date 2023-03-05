use crate::parsing::demo_parsing::Prop;
use crate::parsing::parser::Parser;
use crate::parsing::variants::PropData;
use bitter::BitReader;
use bitter::LittleEndianReader;
use core::panic;
use std::collections::HashMap;
use std::convert::TryInto;
use std::u32;

const SPROP_COORD_MP: i32 = 1 << 12;
const SPROP_COORD_MP_LOWPRECISION: i32 = 1 << 13;
const SPROP_COORD_MP_INTEGRAL: i32 = 1 << 14;

const coordFractionalBitsMp: u32 = 5;
const coordFractionalBitsMpLowPrecision: i32 = 3;
const coordDenominator: i32 = 1 << coordFractionalBitsMp;
const coordResolution: f32 = 1.0 / coordDenominator as f32;
const coordDenominatorLowPrecision: i32 = 1 << coordFractionalBitsMpLowPrecision;
const coordResolutionLowPrecision: f32 = 1.0 / coordDenominatorLowPrecision as f32;
const coordIntegerBitsMp: i32 = 11;
const coordIntegerBits: i32 = 14;

pub struct MyBitreader<'a> {
    pub reader: LittleEndianReader<'a>,
}

impl<'a> MyBitreader<'a> {
    pub fn new(bytes: &'a [u8]) -> MyBitreader<'a> {
        let b = MyBitreader {
            reader: LittleEndianReader::new(bytes),
        };
        b
    }
    pub fn read_nbits(&mut self, n: u32) -> Option<u32> {
        let bits = self.reader.read_bits(n)?;
        Some(bits as u32 & MASKS[n as usize])
    }
    pub fn read_boolie(&mut self) -> Option<bool> {
        self.reader.read_bit()
    }

    pub fn read_u_bit_var(&mut self) -> Option<u32> {
        let mut ret = self.read_nbits(6)?;
        if ret & 48 == 16 {
            ret = (ret & 15) | (self.read_nbits(4)? << 4);
        } else if ret & 48 == 32 {
            ret = (ret & 15) | (self.read_nbits(8)? << 4);
        } else if ret & 48 == 48 {
            ret = (ret & 15) | (self.read_nbits(28)? << 4);
        }
        Some(ret)
    }

    pub fn read_inx(&mut self, last: i32, new_way: bool) -> Option<i32> {
        if new_way && self.read_boolie()? {
            return Some(last + 1);
        }
        if new_way && self.read_boolie()? {
            let index = self.read_nbits(3)?;
            Some(last + 1 + index as i32)
        } else {
            let index = self.read_nbits(7)?;
            let val = index & (32 | 64);
            let index = match val {
                32 => (index & !96) | (self.read_nbits(2)? << 5),
                64 => (index & !96) | (self.read_nbits(4)? << 5),
                96 => (index & !96) | (self.read_nbits(7)? << 5),
                _ => index,
            };
            if index == 0xfff {
                return Some(-1);
            }
            Some(last + 1 + index as i32)
        }
    }

    pub fn read_varint(&mut self) -> Option<u32> {
        let mut result: u32 = 0;
        let mut count: i32 = 0;
        let mut b: u32;

        loop {
            if count >= 5 {
                return result.try_into().unwrap_or(None);
            }
            b = self.read_nbits(8)?;
            result |= (b & 127) << (7 * count);
            count += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        Some(result)
    }

    pub fn decode_string(&mut self) -> Option<String> {
        let mut length = self.read_nbits(9)?;
        if length == 0 {
            return Some("".to_string());
        }
        if length >= (1 << 9) {
            length = (1 << 9) - 1
        }
        Some(self.read_string(length as i32)?)
    }

    pub fn decode(&mut self, prop: &Prop) -> Option<PropData> {
        match prop.p_type {
            0 => Some(PropData::I32(self.decode_int(prop)? as i32)),
            1 => Some(PropData::F32(self.decode_float(prop)?)),
            2 => Some(PropData::VecXYZ(self.decode_vec(prop)?)),
            3 => Some(PropData::VecXY(self.decode_vec_xy(prop)?)),
            4 => Some(PropData::String(self.decode_string()?)),
            5 => Some(PropData::Vec(self.decode_array(prop)?)),
            _ => panic!("Prop had impossible type: {}", prop.p_type),
        }
    }

    pub fn decode_array(&mut self, prop: &Prop) -> Option<Vec<i32>> {
        // SUS
        let b = (prop.num_elements as f32).log2().floor() + 1.0;
        let num_elements = self.read_nbits(b as u32)?;
        let p = prop.arr.as_ref().unwrap();
        let mut elems = vec![];
        for _ in 0..num_elements {
            let pro = Prop {
                table: "oopsie".to_string(),
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
        Some(vec![0, 0, 0])
    }

    pub fn read_string(&mut self, length: i32) -> Option<String> {
        let mut s: Vec<u8> = Vec::new();
        for _ in 0..length {
            let c = self.read_sint_bits(8)? as u8;
            if c == 0 {
                break;
            }
            s.push(c);
        }
        let out = String::from_utf8_lossy(&s);
        Some(out.to_string())
    }
    pub fn read_string_lossy(&mut self, length: i32) -> Option<String> {
        let mut s: Vec<u8> = Vec::new();
        let mut inx = 1;
        loop {
            let c = self.read_sint_bits(8)? as u8;
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
        Some(out.to_string())
    }

    pub fn decode_vec(&mut self, prop: &Prop) -> Option<[f32; 3]> {
        let x = self.decode_float(prop)?;
        let y = self.decode_float(prop)?;

        if prop.flags & (1 << 5) == 0 {
            let z = self.decode_float(prop)?;
            Some([x, y, z])
        } else {
            let sign = self.reader.read_bit()?;
            let temp = (x * x) + (y * y);
            let mut z = 0.0;
            if temp < 1.0 {
                z = (1.0 - temp).sqrt();
            }
            if sign {
                return Some([x, y, -z]);
            }
            Some([x, y, z])
        }
    }

    pub fn read_bits_st(&mut self, n: u32) -> Option<Vec<u8>> {
        let eight = 8.try_into().unwrap();
        let mut bytarr: Vec<u8> = vec![];
        for i in 0..n {
            bytarr.push(self.read_nbits(eight)?.try_into().unwrap());
        }
        Some(bytarr)
    }

    pub fn decode_vec_xy(&mut self, prop: &Prop) -> Option<[f32; 2]> {
        let x = self.decode_float(prop)?;
        let y = self.decode_float(prop)?;
        let v = [x, y];
        Some(v)
    }

    pub fn read_sint_bits(&mut self, n: i32) -> Option<u32> {
        Some(self.read_nbits(n.try_into().unwrap())? << (32 - n) >> (32 - n))
    }

    pub fn read_bit_cell_coord(&mut self, n: usize, integral: bool, lowpres: bool) -> Option<f32> {
        let num_bits = n as i32;
        if integral {
            return Some(self.read_sint_bits(num_bits).unwrap() as f32);
        } else {
            let i = self.read_sint_bits(num_bits).unwrap();
            if lowpres {
                let fract = self.read_sint_bits(3).unwrap();
                return Some(i as f32 + fract as f32 * coordResolutionLowPrecision);
            } else {
                let fract = self
                    .read_sint_bits(coordFractionalBitsMp.try_into().unwrap())
                    .unwrap();
                return Some(i as f32 + fract as f32 * coordResolutionLowPrecision);
            }
        }
    }

    pub fn read_bit_normal(&mut self) -> Option<f64> {
        let sign = self.read_boolie()?;
        let frac = self.read_nbits(11)?;
        let result = frac as f64 * (1.0 / ((1 << 11) - 1) as f64);
        //println!("HERE2 {}", result as f32);
        if sign {
            Some(-result)
        } else {
            Some(result)
        }
    }

    pub fn read_bit_coord_mp(&mut self, integral: bool, lowpres: bool) -> Option<f32> {
        let in_bound = self.read_boolie().unwrap();
        let result = 0;
        if integral {
            if self.read_boolie().unwrap() {
                let is_negative = self.read_boolie().unwrap();
                if in_bound {
                    return Some((self.read_sint_bits(coordIntegerBitsMp).unwrap() + 1) as f32);
                } else {
                    return Some((self.read_sint_bits(coordIntegerBits).unwrap() + 1) as f32);
                }
            } else {
                panic!("WTF");
            }
        } else {
            let read_int_val = self.read_boolie().unwrap();
            let neg = self.read_boolie().unwrap();

            let mut intval = 0;

            if read_int_val {
                if in_bound {
                    intval = self.read_sint_bits(coordIntegerBitsMp).unwrap() + 1;
                } else {
                    intval = self.read_sint_bits(coordIntegerBits).unwrap() + 1;
                }
            }
            if lowpres {
                return Some(
                    intval as f32
                        + self
                            .read_sint_bits(coordFractionalBitsMpLowPrecision)
                            .unwrap() as f32
                            * coordResolutionLowPrecision,
                );
            } else {
                return Some(
                    intval as f32
                        + self
                            .read_sint_bits(coordFractionalBitsMp.try_into().unwrap())
                            .unwrap() as f32
                            * coordResolution,
                );
            }
        }
    }

    pub fn read_bit_coord(&mut self) -> Option<i32> {
        let mut int_val = 0;
        let mut frac_val = 0;

        let i2 = self.reader.read_bit()?;
        let f2 = self.reader.read_bit()?;

        if !i2 && !f2 {
            return Some(0);
        }
        let sign = self.reader.read_bit()?;
        if i2 {
            int_val = self.read_nbits(14)? + 1;
        }
        if f2 {
            frac_val = self.read_nbits(5)?;
        }
        let resol: f64 = 1.0 / (1 << 5) as f64;
        let result: i32 = (int_val as f64 + (frac_val as f64 * resol) as f64) as i32;
        if sign {
            Some(-result)
        } else {
            Some(result)
        }
    }

    #[inline(always)]
    pub fn decode_special_float(&mut self, prop: &Prop) -> Option<f32> {
        if prop.flags & (1 << 1) != 0 {
            return Some(self.read_bit_coord()? as f32);
        } else if prop.flags & SPROP_COORD_MP != 0 {
            return Some(self.read_bit_coord_mp(false, false).unwrap());
        } else if prop.flags & SPROP_COORD_MP_LOWPRECISION != 0 {
            return Some(self.read_bit_coord_mp(false, true).unwrap());
        } else if prop.flags & SPROP_COORD_MP_INTEGRAL != 0 {
            return Some(self.read_bit_coord_mp(true, false).unwrap());
        } else if prop.flags & (1 << 2) != 0 {
            return Some(self.reader.read_f32().unwrap());
        } else if prop.flags & (1 << 5) != 0 {
            return Some(self.read_bit_normal()? as f32);
        } else if prop.flags & (1 << 15) != 0 {
            return Some(self.read_bit_cell_coord(prop.num_bits as usize, false, false)? as f32);
        } else if prop.flags & (1 << 16) != 0 {
            return Some(self.read_bit_cell_coord(prop.num_bits as usize, true, false)? as f32);
        } else if prop.flags & (1 << 17) != 0 {
            return Some(self.read_bit_cell_coord(prop.num_bits as usize, false, true)? as f32);
        }
        return None;
    }

    #[inline(always)]
    pub fn decode_float(&mut self, prop: &Prop) -> Option<f32> {
        let val = self.decode_special_float(prop);
        match val {
            Some(v) => return Some(v),
            None => {
                let max_value = 1 << prop.num_bits;
                let interp = self.read_nbits(prop.num_bits as u32).unwrap();
                let fval = (interp / max_value) as f32;
                return Some(prop.low_value + (prop.high_value - prop.low_value) * fval);
            }
        }
    }

    pub fn decode_int(&mut self, prop: &Prop) -> Option<u32> {
        let flags = prop.flags;
        if flags & (1 << 19) != 0 {
            if flags & (1 << 0) != 0 {
                let result: i32 = self.read_varint()?.try_into().unwrap();
                Some(result as u32)
            } else {
                let mut result = self.read_varint()?;
                result = (result >> 1) ^ (!(result & 1));
                Some(result)
            }
        } else {
            if flags & (1 << 0) != 0 {
                if prop.num_bits == 1 {
                    let result = self.read_nbits(1)?;
                    Some(result as u32)
                } else {
                    let result: u32 = self.read_nbits(prop.num_bits as u32)?;
                    Some(result as u32)
                }
            } else {
                let result = self.read_sbit_long(prop.num_bits.try_into().unwrap())?;
                Some(result as u32)
            }
        }
    }

    pub fn read_sbit_long(&mut self, numbits: u32) -> Option<i32> {
        let nret = self.read_nbits(numbits)? as i32;
        Some((nret << (32 - numbits)) >> (32 - numbits))
    }
}

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
