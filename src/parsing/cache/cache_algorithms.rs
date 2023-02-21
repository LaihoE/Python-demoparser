/*
use crate::parsing::cache::cache_reader::ReadCache;
use crate::parsing::create_output::create_output::ExtraEventRequest;
use crate::parsing::demo_parsing::*;
use crate::parsing::players::Players;
use ahash::{HashMap, HashSet};
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use itertools::Itertools;

impl ReadCache {
    pub fn find_game_event_ticks(
        &self,
        name: String,
        event_map: &Option<HashMap<i32, Descriptor_t>>,
    ) -> Vec<u64> {
        self.event_bytes_by_name(name, event_map)
    }

    pub fn event_name_to_id(
        &self,
        name: &String,
        event_map: &Option<HashMap<i32, Descriptor_t>>,
    ) -> Option<i32> {
        let map = event_map.as_ref().unwrap();
        for (k, v) in map {
            if v.name() == name {
                return Some(*k);
            }
        }
        None
    }

    pub fn find_request_bytes(
        &mut self,
        requests: &Vec<ExtraEventRequest>,
        svc_map: &HashMap<u16, ServerClass>,
        players: &Players,
    ) -> Vec<u64> {
        let uids: Vec<u32> = requests.into_iter().map(|x| x.userid).unique().collect();
        let props: Vec<String> = requests.iter().map(|x| x.prop.clone()).unique().collect();
        let ticks: Vec<i32> = requests.iter().map(|x| x.tick).unique().collect();

        self.find_wanted_bytes(&ticks, &props, &uids, svc_map, players)
    }

    pub fn find_wanted_bytes(
        &mut self,
        ticks: &Vec<i32>,
        props: &Vec<String>,
        uids: &Vec<u32>,
        svc_map: &HashMap<u16, ServerClass>,
        players: &Players,
    ) -> Vec<u64> {
        let mut wanted_bytes = HashSet::default();
        for prop in props {
            let prefix: Vec<&str> = prop.split("@").collect();

            match prefix[0] {
                "player" => {
                    self.read_deltas_by_name(prop, &svc_map);
                    for uid in uids {
                        let bytes = self.find_delta_ticks(*uid, prop.to_owned(), &ticks, &players);
                        for byte in bytes {
                            wanted_bytes.insert(byte);
                        }
                    }
                }
                "rules" => self.read_other_deltas_by_name(prop, &svc_map, 39),
                "manager" => self.read_other_deltas_by_name(prop, &svc_map, 41),
                "team" => self.read_other_deltas_by_name(prop, &svc_map, 43),
                //"other" => self.find_weapon_deltas(prop_name, wanted_ticks, players),
                _ => panic!("unknown prop prefix {:?}", prefix[0]),
            }
        }
        println!("Wanted bytes {:?}", wanted_bytes.len());
        // Unique bytes
        wanted_bytes.iter().map(|x| *x).collect()
    }

    pub fn find_delta_ticks_others(
        &mut self,
        userid: u32,
        prop_name: String,
        wanted_ticks: &Vec<i32>,
        players: &Players,
    ) -> Vec<u64> {
        let x = match self.deltas.get(&prop_name) {
            Some(x) => {
                let mut last_idx = 0;
                let mut out_bytes = vec![];

                // retur n vec![];
                if x.len() == 0 {
                    return vec![];
                }

                for wanted_tick in wanted_ticks {
                    for j in x[last_idx..].windows(2) {
                        last_idx += 1;
                        if j[0].tick <= *wanted_tick && j[1].tick > *wanted_tick {
                            out_bytes.push(j[0].byte);
                            break;
                        }
                    }
                    out_bytes.push(x[x.len() - 1].byte)
                }

                return out_bytes;
            }
            None => return vec![],
        };
    }

    pub fn find_delta_ticks(
        &mut self,
        userid: u32,
        prop_name_temp: String,
        wanted_ticks: &Vec<i32>,
        players: &Players,
    ) -> Vec<u64> {
        let prop_name = if prop_name_temp.contains("m_vecOrigin") {
            "player@DT_CSNonLocalPlayerExclusive.m_vecOrigin"
        } else {
            &prop_name_temp
        };

        let delta_vec = match self.deltas.get_mut(prop_name) {
            Some(delta_v) => delta_v,
            None => return vec![],
        };
        let sid = match players.uid_to_steamid(userid) {
            Some(sid) => sid,
            None => return vec![],
        };
        println!("{}", delta_vec.len());
        delta_vec.sort_by_key(|x| x.tick);

        if players.is_easy_uid[userid as usize] {
            let eid = players.uid_to_entid_tick(userid, 55555).unwrap();
            let all_deltas: Vec<(u64, i32, u32)> = delta_vec
                .iter()
                .filter(|x| x.entid & (1 << eid) != 0)
                .map(|x| (x.byte, x.tick, eid))
                .collect();
            self.filter_delta_ticks_wanted(&all_deltas, wanted_ticks)
        } else {
            let mut all_deltas = vec![];
            for i in 0..16 {
                let temp_deltas: Vec<(u64, i32, u32)> = delta_vec
                    .iter()
                    .filter(|x| {
                        (x.entid & (1 << i) != 0) && players.eid_to_sid(i, x.tick) == Some(sid)
                    })
                    .map(|x| (x.byte, x.tick, i))
                    .collect();
                all_deltas.extend(temp_deltas)
            }
            self.filter_delta_ticks_wanted(&all_deltas, wanted_ticks)
        }
    }

    pub fn filter_delta_ticks_wanted(
        &self,
        sorted_ticks: &Vec<(u64, i32, u32)>,
        wanted_ticks: &Vec<i32>,
    ) -> Vec<u64> {
        if sorted_ticks.len() == 0 {
            return vec![];
        }

        let mut wanted_bytes = Vec::with_capacity(wanted_ticks.len());
        for wanted_tick in wanted_ticks {
            let idx = sorted_ticks.partition_point(|x| x.1 < *wanted_tick);
            if idx > 0 {
                wanted_bytes.push(sorted_ticks[idx - 1].0);
            } else {
                wanted_bytes.push(sorted_ticks[0].0);
            }
        }
        wanted_bytes
    }

    pub fn get_stringtables(&self) -> Vec<u64> {
        self.stringtables.iter().map(|s| s.byte).collect()
    }
}
 */
