use core::num;
use csgoproto::cstrike15_gcmessages::score_leaderboard_data::Entry;
use csgoproto::netmessages::{CSVCMsg_SendTable, CSVCMsg_UpdateStringTable};
use pyo3::{Py, Python};
use std::convert::TryInto;
use std::hash::Hash;
//use hashbrown::HashMap;
use crate::parsing::read_bits::BitReader;
use crate::Demo;
use csgoproto::netmessages::CSVCMsg_CreateStringTable;
use pyo3::PyAny;
use pyo3::ToPyObject;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone)]
pub struct StringTable {
    userinfo: bool,
    name: String,
    max_entries: i32,
    uds: i32,
    udfs: bool,
    udsb: i32,
    data: Vec<StField>,
}
#[derive(Clone)]
pub struct StField {
    entry: String,
    udata: String,
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
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut hm: HashMap<String, String> = HashMap::new();
        //hm.insert("version".to_string(), self.version.to_string());
        hm.insert("steamid".to_string(), self.xuid.to_string());
        //hm.insert("name".to_string(), self.name.to_string());
        //hm.insert("user_id".to_string(), self.user_id.to_string());
        //hm.insert("guid".to_string(), self.guid.to_string());
        //hm.insert("friends_id".to_string(), self.friends_id.to_string());
        hm.insert("name".to_string(), self.name.to_string());
        //hm.insert("fake_player".to_string(), self.fake_player.to_string());
        //hm.insert("hltv".to_string(), self.hltv.to_string());
        //hm.insert("custom_files".to_string(), self.custom_files.to_string());
        //hm.insert("files_downloaded".to_string(), self.files_downloaded.to_string());
        hm.insert("entity_id".to_string(), self.entity_id.to_string());
        //hm.insert("tbd".to_string(), self.tbd.to_string());
        hm
    }

    pub fn to_py_hashmap(&self, py: Python<'_>) -> Py<PyAny> {
        let hm = self.to_hashmap();
        let dict = pyo3::Python::with_gil(|py| hm.to_object(py));
        dict
    }
}

impl Demo {
    pub fn parse_string_table(&mut self) {
        let length = self.read_i32();
        let data = self.read_n_bytes(length.try_into().unwrap());
    }
    pub fn parse_userinfo(userdata: [u8; 340]) -> UserInfo {
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
        userinfo: bool,
        num_entries: i32,
        max_entries: i32,
        user_data_size: i32,
        user_data_fixsize: bool,
    ) -> StringTable {
        //println!("DATALEN {}", data.len());
        let left_over = (data.len() % 4) as i32;
        let mut buf = BitReader::new(data, left_over);
        buf.read_uneven_end_bits();

        let mut entry_bits = (max_entries as f32).log2() as i32;
        let mut index = 0;
        let mut last_inx: i32 = -1;
        let mut idx = 0;
        let mut btc = 0;
        let mut history: Vec<String> = Vec::with_capacity(50000);
        let mut entry = String::new();
        let mut user_data: [u8; 340] = [0; 340];
        buf.read_bool();

        for i in 1..50000 {
            history.push("".to_string())
        }

        for i in 0..num_entries {
            index = last_inx + 1;
            if buf.read_bool() == false {
                index = buf
                    .read_nbits(entry_bits.try_into().unwrap())
                    .try_into()
                    .unwrap();
            }
            last_inx = index;
            if buf.read_bool() {
                if buf.read_bool() {
                    idx = buf.read_nbits(5);
                    btc = buf.read_nbits(5);
                    let substring = "";
                    let suffix = buf.read_string_lossy(0);
                    entry = (substring.to_string() + &suffix.to_owned());
                } else {
                    entry = buf.read_string_lossy(0);
                }
                st.data[index as usize].entry = entry.to_string()
            }
            if buf.read_bool() {
                if user_data_fixsize {
                    user_data = buf.read_bits_st(user_data_size);
                    //println!("USERDATA 1");
                    if st.userinfo {
                        let mut ui = Demo::parse_userinfo(user_data);
                        if ui.xuid > 76500000000000000 && ui.xuid < 76600000000000000 {
                            self.players_connected += 1;
                        }
                        ui.friends_name = ui.friends_name.trim_end_matches("\x00").to_string();
                        ui.name = ui.name.trim_end_matches("\x00").to_string();
                        self.wanted_ent_ids.push(ui.entity_id.clone());
                        self.players.insert(ui.entity_id.clone(), ui);
                    }
                } else {
                    let size = buf.read_nbits(14);
                    user_data = buf.read_bits_st(size.try_into().unwrap());

                    if st.userinfo {
                        let mut ui = Demo::parse_userinfo(user_data);
                        ui.entity_id = (st.data[index as usize].entry).parse::<u32>().unwrap() + 2;
                        if ui.xuid > 76500000000000000 && ui.xuid < 76600000000000000 {
                            self.players_connected += 1;
                        }
                        ui.friends_name = ui.friends_name.trim_end_matches("\x00").to_string();
                        ui.name = ui.name.trim_end_matches("\x00").to_string();
                        self.wanted_ent_ids.push(ui.entity_id.clone());
                        self.players.insert(ui.entity_id.clone(), ui);
                    }
                }
                if history.len() == 32 {
                    history.remove(0);
                }
            }
            history.push(entry.to_string());
        }
        st
    }

    pub fn create_string_table(&mut self, data: CSVCMsg_CreateStringTable) {
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
            udsb: data.user_data_size_bits(),
            data: Vec::new(),
        };
        for _ in 1..50000 {
            st.data.push(StField {
                entry: "".to_string(),
                udata: "".to_string(),
            })
        }
        let ui = st.userinfo;
        let st = &self.update_string_table(
            data.string_data(),
            st,
            ui,
            data.num_entries(),
            data.max_entries(),
            data.user_data_size_bits(),
            data.user_data_fixed_size(),
        );
        self.stringtables.push(st.clone());
    }

    pub fn update_string_table_msg(&mut self, data: CSVCMsg_UpdateStringTable) {
        let st = self.stringtables.get_mut(data.table_id() as usize).unwrap();

        if st.name != "userinfo" {
            return;
        }

        let left_over = (data.string_data().len() % 4) as i32;
        let mut buf = BitReader::new(data.string_data(), left_over);
        buf.read_uneven_end_bits();

        let mut entry_bits = (st.max_entries as f32).log2() as i32;
        let mut index = 0;
        let mut last_inx: i32 = -1;
        let mut idx = 0;
        let mut btc = 0;
        let mut history: Vec<String> = Vec::new();
        let mut entry = String::new();
        let mut user_data: [u8; 340] = [0; 340];
        buf.read_bool();

        for i in 1..50000 {
            history.push("".to_string())
        }

        for i in 0..st.max_entries {
            index = last_inx + 1;
            if buf.read_bool() == false {
                index = buf
                    .read_nbits(entry_bits.try_into().unwrap())
                    .try_into()
                    .unwrap();
            }
            last_inx = index;
            if buf.read_bool() {
                if buf.read_bool() {
                    idx = buf.read_nbits(5);
                    btc = buf.read_nbits(5);
                    let substring = "";
                    let suffix = buf.read_string_lossy(0);
                    entry = (substring.to_string() + &suffix.to_owned());
                } else {
                    entry = buf.read_string_lossy(0);
                }
                st.data[index as usize].entry = entry.to_string()
            }
            if buf.read_bool() {
                if st.udfs {
                    user_data = buf.read_bits_st(st.uds);
                    if st.userinfo {
                        let mut ui = Demo::parse_userinfo(user_data);
                        ui.entity_id = (st.data[index as usize].entry).parse::<u32>().unwrap() + 2;
                        if ui.xuid > 76500000000000000 && ui.xuid < 76600000000000000 {
                            self.players_connected += 1;
                        }

                        ui.friends_name = ui.friends_name.trim_end_matches("\x00").to_string();
                        ui.name = ui.name.trim_end_matches("\x00").to_string();
                        //println!("Created player: {} {}", ui.name, ui.entity_id);
                        self.wanted_ent_ids.push(ui.entity_id.clone());
                        self.players.insert(ui.entity_id.clone(), ui);
                    }
                } else {
                    let size = buf.read_nbits(14);
                    user_data = buf.read_bits_st(size.try_into().unwrap());

                    if st.userinfo {
                        let mut ui = Demo::parse_userinfo(user_data);
                        if st.data[index as usize].entry != "" {
                            let temp_id = (st.data[index as usize].entry).parse::<u32>();
                            match temp_id {
                                Err(e) => ui.entity_id = 99999,
                                Ok(ok) => {
                                    ui.entity_id =
                                        (st.data[index as usize].entry).parse::<u32>().unwrap() + 2;
                                    if ui.xuid > 76500000000000000 && ui.xuid < 76600000000000000 {
                                        self.players_connected += 1;
                                    }
                                    ui.friends_name =
                                        ui.friends_name.trim_end_matches("\x00").to_string();
                                    ui.name = ui.name.trim_end_matches("\x00").to_string();
                                    //println!("Created player: {} {}", ui.name, ui.entity_id);
                                    self.wanted_ent_ids.push(ui.entity_id.clone());
                                    self.players.insert(ui.entity_id.clone(), ui);
                                }
                            }
                        }
                    }
                }
                if history.len() == 32 {
                    history.remove(0);
                }
            }
            history.push(entry.to_string());
        }
    }
}
