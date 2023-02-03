use crate::parsing::cache::cache_reader::ReadCache;
use crate::parsing::demo_parsing::*;
use crate::parsing::players::Players;
use ahash::HashMap;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use itertools::Itertools;

impl ReadCache {
    pub fn find_game_event_ticks(
        &self,
        name: String,
        event_map: &Option<HashMap<i32, Descriptor_t>>,
    ) {
        let event_bytes = self.event_bytes_by_name(name, event_map);
    }

    pub fn find_wanted_bytes(
        &mut self,
        ticks: &Vec<i32>,
        props: &Vec<String>,
        uids: &Vec<u32>,
        svc_map: &HashMap<u16, ServerClass>,
        players: &Players,
    ) -> Vec<u64> {
        let mut wanted_bytes = vec![];
        for prop in props {
            self.read_deltas_by_name(prop, &svc_map);
            for uid in uids {
                wanted_bytes.extend(self.find_delta_ticks(*uid, prop.to_owned(), &ticks, &players));
            }
        }
        // Unique bytes
        wanted_bytes.iter().map(|x| x.clone()).unique().collect()
    }

    pub fn find_delta_ticks(
        &mut self,
        userid: u32,
        prop_name: String,
        wanted_ticks: &Vec<i32>,
        players: &Players,
    ) -> Vec<u64> {
        let delta_vec = self.deltas.get(&prop_name).unwrap();
        let wanted_sid = players.uid_to_steamid(userid).unwrap();

        if players.is_easy_uid[userid as usize] {
            let eid = players.uid_to_entid(userid, 55555).unwrap();
            let all_deltas: Vec<(u64, i32, u32)> = delta_vec
                .iter()
                .filter(|x| x.entid == eid)
                .map(|x| (x.byte, x.tick, x.entid))
                .collect();
            self.filter_delta_ticks_wanted(&all_deltas, wanted_ticks)
        } else {
            let all_deltas: Vec<(u64, i32, u32)> = delta_vec
                .iter()
                .filter(|x| players.eid_to_sid(x.entid, x.tick) == Some(wanted_sid))
                .map(|x| (x.byte, x.tick, x.entid))
                .collect();
            self.filter_delta_ticks_wanted(&all_deltas, wanted_ticks)
        }
    }

    pub fn filter_delta_ticks_wanted(
        &self,
        temp_ticks: &Vec<(u64, i32, u32)>,
        wanted_ticks: &Vec<i32>,
    ) -> Vec<u64> {
        if temp_ticks.len() == 0 {
            return vec![];
        }

        let mut wanted_bytes = Vec::with_capacity(wanted_ticks.len());
        let mut sorted_ticks = temp_ticks.clone();
        sorted_ticks.sort_by_key(|x| x.1);
        let mut last_idx = 0;

        for wanted_tick in wanted_ticks {
            for j in sorted_ticks[last_idx..].windows(2) {
                last_idx += 1;
                if j[0].1 <= *wanted_tick && j[1].1 > *wanted_tick {
                    wanted_bytes.push(j[0].0);
                    break;
                }
            }
        }

        let mut bin = vec![];
        for wanted_tick in wanted_ticks {
            let idx = sorted_ticks.partition_point(|x| x.1 < *wanted_tick);
            if idx > 0 {
                bin.push(sorted_ticks[idx - 1].0);
            } else {
                bin.push(sorted_ticks[0].0);
            }
        }
        // println!("1 {:?}", wanted_bytes);
        // println!("2 {:?}", bin);
        // println!("{:?}", wanted_ticks);
        bin
    }

    pub fn get_stringtables(&self) -> Vec<u64> {
        self.stringtables.iter().map(|s| s.byte).collect()
    }
}
