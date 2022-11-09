use memmap::Mmap;

// Some of these could be combined
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
    pub data: VarVec,
}

#[derive(Debug, Clone)]
pub enum PropData {
    I32(i32),
    F32(f32),
    I64(i64),
    String(String),
    VecXY([f32; 2]),
    VecXYZ([f32; 3]),
    Vec(Vec<i32>),
}

#[derive(Debug, Clone)]
pub struct PropAtom {
    pub prop_name: String,
    pub data: PropData,
    pub tick: i32,
}

#[inline(always)]
pub fn create_default_from_pdata(pdata: PropData, playback_frames: usize) -> VarVec {
    match pdata {
        PropData::I32(_) => VarVec::I32(vec![None; playback_frames]),
        PropData::I64(_) => VarVec::I32(vec![None; playback_frames]),
        PropData::F32(_) => VarVec::F32(vec![None; playback_frames]),
        PropData::String(_) => VarVec::String(vec![None; playback_frames]),
        _ => panic!("no matching varvec for propdata: {:?}", pdata),
    }
}

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
                    //println!("{:?}", self);
                    panic!("Tried to push a {:?} into a string column", p);
                }
            },
            _ => panic!("bad type for prop"),
        }
    }
    pub fn push_string(&mut self, data: String) {
        if let VarVec::String(f) = self {
            f.push(Some(data))
        }
    }
    pub fn push_string_none(&mut self) {
        if let VarVec::String(f) = self {
            f.push(None)
        }
    }
    pub fn push_float_none(&mut self) {
        if let VarVec::F32(f) = self {
            f.push(None)
        }
    }
    pub fn push_i32_none(&mut self) {
        if let VarVec::I32(f) = self {
            f.push(None)
        }
    }
    pub fn push_none(&mut self) {
        match self {
            VarVec::I32(f) => f.push(None),
            VarVec::F32(f) => f.push(None),
            VarVec::String(f) => f.push(None),
            _ => panic!("unk col while pushing none"),
        }
    }
    pub fn push_u64(&mut self, data: u64) {
        match self {
            VarVec::U64(f) => f.push(Some(data)),
            _ => panic!("TRIED TO PUSH SMALLER TYPE TO U64"),
        }
    }
    pub fn push_i32(&mut self, data: i32) {
        match self {
            VarVec::I32(f) => f.push(Some(data)),
            _ => panic!("i32 push panic"),
        }
    }
    pub fn get_len(&self) -> usize {
        match self {
            VarVec::I32(v) => v.len(),
            VarVec::F32(v) => v.len(),
            VarVec::String(v) => v.len(),
            _ => panic!("bad len type"),
        }
    }
    pub fn insert_propdata(&mut self, inx: usize, pdata: PropData) {
        match self {
            VarVec::I32(v) => match pdata {
                PropData::I32(i) => v[inx] = Some(i),
                _ => panic!("varvec didnt match propdata"),
            },
            VarVec::F32(v) => match pdata {
                PropData::F32(f) => v[inx] = Some(f),
                _ => panic!("varvec didnt match propdata"),
            },
            VarVec::String(v) => match pdata {
                PropData::String(s) => v[inx] = Some(s),
                _ => panic!("varvec didnt match propdata"),
            },
            VarVec::U64(v) => match pdata {
                PropData::I64(s) => v[inx] = Some(s.try_into().unwrap()),
                _ => panic!("varvec didnt match propdata"),
            },
            VarVec::I64(v) => match pdata {
                PropData::I64(s) => v[inx] = Some(s.try_into().unwrap()),
                _ => panic!("varvec didnt match propdata"),
            },
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
            Self::Mmap(m) => &m[i],
            Self::Vec(v) => &v[i],
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
