use crate::Demo;
use memmap::Mmap;

// Some of these could be combined

impl VarVec {
    pub fn push_propdata(&mut self, item: PropData) {
        match item {
            PropData::F32(p) => match self {
                VarVec::F32(f) => f.push(Some(p)),
                _ => {
                    panic!("Tried to push a {:?} into a {:?} column", item, self);
                }
            },
            PropData::I32(p) => match self {
                VarVec::I32(f) => f.push(Some(p)),
                _ => {
                    panic!("Tried to push a {:?} into a {:?} column", item, self);
                }
            },
            PropData::I64(p) => match self {
                VarVec::I64(f) => f.push(Some(p)),
                _ => {
                    panic!("Tried to push a {:?} into a {:?} column", item, self);
                }
            },
            PropData::String(p) => match self {
                VarVec::String(f) => f.push(Some(p)),
                _ => {
                    panic!("Tried to push a {:?} into a string column", p);
                }
            },
            _ => panic!("bad type for prop"),
        }
    }
    pub fn push_string(&mut self, data: String) {
        match self {
            VarVec::String(f) => f.push(Some(data)),
            _ => {}
        }
    }
    pub fn push_string_none(&mut self) {
        match self {
            VarVec::String(f) => f.push(None),
            _ => {}
        }
    }
    pub fn push_float_none(&mut self) {
        match self {
            VarVec::F32(f) => f.push(None),
            _ => {}
        }
    }
    pub fn push_i32_none(&mut self) {
        match self {
            VarVec::I32(f) => f.push(None),
            _ => {}
        }
    }
}

pub enum BytesVariant {
    Mmap(Mmap),
    Vec(Vec<u8>),
}

impl<Idx> std::ops::Index<Idx> for BytesVariant
where
    Idx: std::slice::SliceIndex<[u8]>,
{
    type Output = Idx::Output;
    #[inline(always)]
    fn index(&self, i: Idx) -> &Self::Output {
        match self {
            Self::Mmap(m) => {
                return &m[i];
            }
            Self::Vec(v) => {
                return &v[i];
            }
        }
    }
}
impl BytesVariant {
    pub fn get_len(&self) -> usize {
        match self {
            Self::Mmap(m) => m.len(),
            Self::Vec(v) => v.len(),
        }
    }
}
#[derive(Debug, Clone)]
pub enum VarVec {
    U64(Vec<Option<u64>>),
    F32(Vec<Option<f32>>),
    I64(Vec<Option<i64>>),
    I32(Vec<Option<i32>>),
    String(Vec<Option<String>>),
}
#[derive(Debug, Clone)]
pub struct PropColumn {
    pub dtype: String,
    pub data: VarVec,
}

#[derive(Debug, Clone)]
pub enum PropData {
    I32(i32),
    F32(f32),
    I64(i64),
    String(String),
    VecXY(Vec<f32>),
    VecXYZ(Vec<f32>),
    Vec(Vec<i32>),
}

#[derive(Debug)]
pub struct PropAtom {
    pub prop_name: String,
    pub data: PropData,
    pub tick: i32,
}
