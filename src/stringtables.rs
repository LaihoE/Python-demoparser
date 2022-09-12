use std::convert::TryInto;

use csgoproto::cstrike15_gcmessages::score_leaderboard_data::Entry;
use csgoproto::netmessages::CSVCMsg_SendTable;

use crate::protobuf::Message;
use crate::read_bits::BitReader;
use crate::Demo;
use csgoproto::netmessages::CSVCMsg_CreateStringTable;

pub struct StringTable {
    userinfo: bool,
    name: String,
    max_entries: i32,
    uds: i32,
    udfs: bool,
    udsb: i32,
    data: Vec<StField>,
}
pub struct StField {
    entry: String,
    udata: String,
}
#[derive(Debug)]
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

impl Demo {
    pub fn parse_string_table(&mut self) {
        let length = self.read_i32();
        let data = self.read_n_bytes(length.try_into().unwrap());
        println!("{:?}", data);
    }
    pub fn parse_userinfo(&mut self, userdata: [u8; 340]) -> UserInfo {
        println!("{:?}", userdata);

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
        println!("{:#?}", ui);
        ui
    }

    pub fn update_string_table(
        &mut self,
        data: CSVCMsg_CreateStringTable,
        mut st: StringTable,
    ) -> StringTable {
        let mut buf = BitReader::new(data.string_data());
        let mut entry_bits = (data.max_entries() as f32).log2() as i32;
        let mut index = 0;
        let mut last_inx: i32 = -1;
        let mut idx = 0;
        let mut btc = 0;
        let mut history: Vec<String> = Vec::new();
        let mut entry = String::new();
        let mut user_data: [u8; 340] = [0; 340];
        buf.read_bool();

        for i in 1..20000 {
            history.push("".to_string())
        }

        for i in 0..data.num_entries() {
            index = last_inx + 1;
            if buf.read_bool() == false {
                index = buf
                    .read_nbits(entry_bits.try_into().unwrap())
                    .try_into()
                    .unwrap();
            }
            last_inx = index;
            //println!("INX = {}", index);
            if buf.read_bool() {
                if buf.read_bool() {
                    idx = buf.read_nbits(5);
                    btc = buf.read_nbits(5);
                    //let substring = &history[idx as usize][..btc as usize];
                    let substring = "";
                    let suffix = buf.read_string_lossy(0);
                    entry = (substring.to_string() + &suffix.to_owned());
                    //println!("ENTRY 1");
                } else {
                    entry = buf.read_string_lossy(0);
                    //println!("ENTRY 2");
                }
                st.data[index as usize].entry = entry.to_string()
            }
            if buf.read_bool() {
                if data.user_data_fixed_size() {
                    user_data = buf.read_bits_st(data.user_data_size());
                    //println!("USERDATA 1");
                    if st.userinfo {
                        let ui = self.parse_userinfo(user_data);
                        self.players.push(ui);
                    }
                } else {
                    let size = buf.read_nbits(14);
                    user_data = buf.read_bits_st(size.try_into().unwrap());

                    if st.userinfo {
                        let ui = self.parse_userinfo(user_data);
                        self.players.push(ui);
                    }
                }
                if st.userinfo {
                    /*
                    let ui = self.parse_userinfo(user_data);
                    self.players.push(ui);
                    */
                }
                if history.len() == 32 {
                    history.remove(0);
                }
            }
            history.push(entry.to_string())
        }
        println!("DONe@@@@@@@@@@@@@");
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
        for i in 1..20000 {
            st.data.push(StField {
                entry: "".to_string(),
                udata: "".to_string(),
            })
        }
        st = self.update_string_table(data, st);
        self.stringtables.push(st);
    }
}
