//use crate::parsing::read_bits_old::BitReader;
use super::read_bits::MyBitreader;
use crate::parsing::entities::parse_baselines;
use crate::parsing::parser::Parser;
use bitter::BitReader;
use core::num;
use csgoproto::netmessages::CSVCMsg_CreateStringTable;
use csgoproto::netmessages::CSVCMsg_UpdateStringTable;
use pyo3::ffi::PyObject;
use pyo3::PyAny;
use pyo3::ToPyObject;
use pyo3::{Py, Python};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;

// THIS ENTIRE FILE IS A MESS
// THIS ENTIRE FILE IS A MESS
// THIS ENTIRE FILE IS A MESS
// THIS ENTIRE FILE IS A MESS
// THIS ENTIRE FILE IS A MESS

#[derive(Clone, Debug)]
pub struct StringTable {
    userinfo: bool,
    name: String,
    max_entries: i32,
    uds: i32,
    udfs: bool,
    data: Vec<StField>,
}
#[derive(Clone, Debug)]
pub struct StField {
    entry: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    pub version: u64,
    pub xuid: u64,
    pub name: String,
    pub user_id: u32,
    pub guid: String,
    pub friends_id: u32,
    pub friends_name: String,
    pub fake_player: bool,
    pub hltv: bool,
    pub custom_files: u64,
    pub files_downloaded: bool,
    pub entity_id: u32,
    pub tbd: u32,
}

impl UserInfo {
    pub fn to_hashmap(&self, py: Python<'_>) -> HashMap<String, pyo3::Py<PyAny>> {
        let mut hm: HashMap<String, pyo3::Py<PyAny>> = HashMap::new();
        //hm.insert("version".to_string(), self.version.to_string());
        hm.insert("steamid".to_string(), self.xuid.to_object(py));
        //hm.insert("name".to_string(), self.name.to_string());
        hm.insert("user_id".to_string(), self.user_id.to_object(py));
        //hm.insert("guid".to_string(), self.guid.to_string());
        //hm.insert("friends_id".to_string(), self.friends_id.to_string());
        hm.insert("name".to_string(), self.name.to_string().to_object(py));
        //hm.insert("fake_player".to_string(), self.fake_player.to_string());
        //hm.insert("hltv".to_string(), self.hltv.to_string());
        //hm.insert("custom_files".to_string(), self.custom_files.to_string());
        //hm.insert("files_downloaded".to_string(), self.files_downloaded.to_string());
        hm.insert("entity_id".to_string(), self.entity_id.to_object(py));
        //hm.insert("tbd".to_string(), self.tbd.to_string());
        hm
    }
}

impl Parser {
    pub fn parse_userinfo(userdata: Vec<u8>) -> UserInfo {
        let ui = UserInfo {
            version: u64::from_be_bytes(userdata[0..8].try_into().unwrap()),
            xuid: u64::from_be_bytes(userdata[8..16].try_into().unwrap()),
            name: String::from_utf8_lossy(&userdata[16..144]).to_string(),
            user_id: u32::from_be_bytes(userdata[144..148].try_into().unwrap()),
            guid: String::from_utf8(userdata[148..181].to_vec()).unwrap(),
            friends_id: u32::from_be_bytes(userdata[181..185].try_into().unwrap()),
            friends_name: String::from_utf8_lossy(&userdata[185..313]).to_string(),
            fake_player: userdata[313] != 0,
            hltv: userdata[314] != 0,
            custom_files: 5,
            files_downloaded: userdata[330] != 0,
            entity_id: u32::from_be_bytes(userdata[331..335].try_into().unwrap()),
            tbd: u32::from_be_bytes(userdata[331..335].try_into().unwrap()),
        };
        ui
    }

    pub fn update_string_table(
        &mut self,
        data: &[u8],
        mut st: StringTable,
        _userinfo: bool,
        num_entries: i32,
        max_entries: i32,
        _user_data_size: i32,
        user_data_fixsize: bool,
    ) -> Option<StringTable> {
        let mut buf = MyBitreader::new(data);
        let entry_bits = (max_entries as f32).log2() as i32;
        let mut entry_index = 0;
        let mut last_inx: i32 = -1;
        let mut history: Vec<String> = Vec::new();
        let mut entry = String::new();

        buf.read_boolie()?;

        for _i in 0..num_entries {
            let mut user_data = vec![];
            entry_index = last_inx + 1;
            if !buf.read_boolie()? {
                entry_index = buf
                    .read_nbits(entry_bits.try_into().unwrap())?
                    .try_into()
                    .unwrap();
            }
            last_inx = entry_index;
            if buf.read_boolie()? {
                if buf.read_boolie()? {
                    let idx = buf.read_nbits(5)? as i32;
                    let bytes_to_copy = buf.read_nbits(5)?;
                    let s = &history[idx as usize];
                    let s_slice = &s[..bytes_to_copy as usize];
                    entry = s_slice.to_owned() + &buf.read_string(4096)?;
                } else {
                    entry = buf.read_string(4096)?;
                }
                st.data[entry_index as usize].entry = entry.to_string()
            }
            if history.len() >= 32 {
                history.remove(0);
            }
            history.push(entry.clone());
            if buf.read_boolie()? {
                user_data = if user_data_fixsize {
                    vec![buf
                        .read_nbits(st.uds.try_into().unwrap())?
                        .try_into()
                        .unwrap()]
                } else {
                    let size = buf.read_nbits(14)?;
                    buf.read_bits_st(size)?
                };
                /*
                if st.name == "instancebaseline" {
                    let k = entry.parse::<u32>().unwrap_or(999999);
                    match self.maps.serverclass_map.read().get(&(k as u16)) {
                        Some(sv_cls) => {
                            parse_baselines(&user_data, sv_cls, &mut self.maps.baselines);
                        }
                        None => {
                            // Serverclass_map is not initiated yet, we need to parse this
                            // later. Just why??? :() just seems unnecessarily complicated
                            self.maps.baseline_no_cls.insert(k, user_data.clone());
                        }
                    }
                    history.push(entry.to_string());
                }
                */
                if st.userinfo {
                    let mut ui = Parser::parse_userinfo(user_data);
                    ui.entity_id = entry_index as u32 + 1;
                    //println!("CREATE: {} {} {}", ui.xuid, ui.entity_id, self.tick);

                    ui.friends_name = ui.friends_name.trim_end_matches("\x00").to_string();
                    ui.name = ui.name.trim_end_matches("\x00").to_string();

                    self.maps
                        .userid_sid_map
                        .entry(ui.user_id)
                        .or_insert(Vec::new())
                        .push((ui.xuid.clone(), self.state.tick));

                    self.maps
                        .uid_eid_map
                        .entry(ui.user_id)
                        .or_insert(Vec::new())
                        .push((ui.entity_id, self.state.tick));
                    self.maps
                        .sid_entid_map
                        .entry(ui.xuid.clone())
                        .or_insert(Vec::new())
                        .push((ui.entity_id, self.state.tick));

                    self.maps.players.insert(ui.xuid.clone(), ui);
                }
            }
        }
        Some(st)
    }

    pub fn create_string_table(&mut self, data: CSVCMsg_CreateStringTable) -> Option<bool> {
        let mut uinfo = false;

        if data.name() == "userinfo" {
            uinfo = true;
        }

        let mut st = StringTable {
            name: data.name().to_string(),
            userinfo: uinfo,
            max_entries: data.max_entries(),
            udfs: data.user_data_fixed_size(),
            uds: data.user_data_size(),
            data: Vec::new(),
        };
        if st.name == "userinfo" || st.name == "instancebaseline" {
            for _ in 1..50000 {
                st.data.push(StField {
                    entry: "".to_string(),
                })
            }
            let ui = st.userinfo;
            st = self.update_string_table(
                data.string_data(),
                st,
                ui,
                data.num_entries(),
                data.max_entries(),
                data.user_data_size_bits(),
                data.user_data_fixed_size(),
            )?;
        }
        self.state.stringtables.push(st.clone());
        Some(true)
    }

    pub fn update_string_table_msg(&mut self, data: CSVCMsg_UpdateStringTable) -> Option<bool> {
        let st = self
            .state
            .stringtables
            .get_mut(data.table_id() as usize)
            .unwrap();
        let mut buf = MyBitreader::new(data.string_data());
        let entry_bits = (st.max_entries as f32).log2() as i32;
        let mut index = 0;
        let mut last_inx: i32 = -1;
        let mut idx = 0;
        let mut btc = 0;
        let mut history: Vec<String> = Vec::new();
        let mut entry = String::new();
        buf.read_boolie()?;

        if !(st.name == "userinfo" || st.name == "instancebaseline") {
            return Some(true);
        }

        for _i in 0..data.num_changed_entries() {
            index = last_inx + 1;
            if !(buf.read_boolie()?) {
                index = buf
                    .read_nbits(entry_bits.try_into().unwrap())?
                    .try_into()
                    .unwrap();
            }
            last_inx = index;
            if buf.read_boolie()? {
                if buf.read_boolie()? {
                    let idx = buf.read_nbits(5)? as i32;
                    let bytes_to_copy = buf.read_nbits(5)?;
                    let s = &history[idx as usize];
                    let s_slice = &s[..bytes_to_copy as usize + 1];
                    entry = s_slice.to_owned() + &buf.read_string(4096)?;
                } else {
                    entry = buf.read_string(4096)?;
                }
                st.data[index as usize].entry = entry.to_string()
            }
            if history.len() >= 32 {
                history.remove(0);
            }
            history.push(entry.clone());
            if buf.read_boolie()? {
                let user_data = if st.udfs {
                    vec![buf
                        .read_nbits(st.uds.try_into().unwrap())?
                        .try_into()
                        .unwrap()]
                } else {
                    let size = buf.read_nbits(14)?;
                    buf.read_bits_st(size)?
                };
                /*
                if st.name == "instancebaseline" {
                    let k = entry.parse::<u32>().unwrap_or(999999);
                    match self.maps.serverclass_map.get(&(k as u16)) {
                        Some(sv_cls) => {
                            parse_baselines(&user_data, sv_cls, &mut self.maps.baselines);
                        }
                        None => {
                            // Serverclass_map is not initiated yet, we need to parse this
                            // later. Just why??? :() just seems unnecessarily complicated
                            self.maps.baseline_no_cls.insert(k, user_data.clone());
                        }
                    }
                }
                */
                if st.userinfo {
                    let mut ui = Parser::parse_userinfo(user_data.clone());
                    ui.entity_id = index as u32 + 1;
                    ui.friends_name = ui.friends_name.trim_end_matches("\x00").to_string();
                    ui.name = ui.name.trim_end_matches("\x00").to_string();

                    self.maps
                        .userid_sid_map
                        .entry(ui.user_id)
                        .or_insert(Vec::new())
                        .push((ui.xuid.clone(), self.state.tick));

                    self.maps
                        .uid_eid_map
                        .entry(ui.user_id)
                        .or_insert(Vec::new())
                        .push((ui.entity_id, self.state.tick));

                    self.maps
                        .sid_entid_map
                        .entry(ui.xuid.clone())
                        .or_insert(Vec::new())
                        .push((ui.entity_id, self.state.tick));

                    self.maps.players.insert(ui.xuid.clone(), ui);
                }
            }
            history.push(entry.to_string());
        }
        Some(true)
    }
}
