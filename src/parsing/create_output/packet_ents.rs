/*
use crate::parsing::cache::MANAGER_CLSID;
use crate::parsing::cache::PLAYER_CLSID;
use crate::parsing::cache::RULES_CLSID;
use crate::parsing::cache::TEAM_CLSID;
use crate::parsing::demo_parsing::entities::PacketEntsOutput;
use crate::parsing::parser::*;
use crate::parsing::players::Players;
use crate::parsing::utils::TYPEHM;
pub use crate::parsing::variants::*;
use ahash::HashMap;
use ahash::HashSet;
use itertools::izip;
use itertools::Itertools;
use polars::prelude::{DataFrame, Int64Type, NamedFrom, NamedFromOwned};

use std::time::Instant;

impl Parser {
    pub fn filter_jobs_by_pidx_other(
        &self,
        results: &Vec<JobResult>,
        lower_boundary: i32,
        prop_name: &String,
        players: &Players,
        wanted_sid: u64,
    ) -> Vec<(f32, i32, i32, i32)> {
        let mut v = vec![];
        for x in results {
            if let JobResult::PacketEntities(pe) = x {
                v.push(pe);
            }
        }

        //let prop_type = TYPEHM.get(&prop_name[..&prop_name.len() - 4]).unwrap();
        let prop_type = TYPEHM.get(&prop_name).unwrap();

        let prefix: Vec<&str> = prop_name.split("@").collect();

        let wanted_entid_type = match prefix[0] {
            "player" => 0,
            "team" => 1,
            "manager" => 2,
            "rules" => 3,
            _ => panic!("unknown prefix: {}", prefix[0]),
        };
        let mut vector = vec![];

        for pe in v {
            match players.sid_to_entid(wanted_sid, pe.tick) {
                Some(eid) => {
                    let wanted_prop_idx = lower_boundary + eid as i32;
                    match prop_type {
                        0 => self.match_int_other(
                            pe,
                            wanted_prop_idx,
                            &mut vector,
                            wanted_entid_type,
                        ),
                        // 1 => self.match_float(pe, prop_idx, &mut vector, wanted_entid_type),
                        // 2 => self.match_str(pe, prop_idx, &mut vector),
                        _ => panic!("Unsupported prop type: {}", prop_type),
                    }
                }
                None => {}
            }
        }
        return vector;
    }

    pub fn filter_jobs_by_pidx(
        &self,
        results: &Vec<JobResult>,
        prop_idx: i32,
        prop_name: &String,
    ) -> Vec<(f32, i32, i32)> {
        let mut v = vec![];
        for x in results {
            if let JobResult::PacketEntities(pe) = x {
                v.push(pe);
            }
        }
        let prop_type = TYPEHM.get(&prop_name).unwrap();
        let mut vector = vec![];

        for pe in v {
            match prop_type {
                0 => self.match_int(pe, prop_idx, &mut vector),
                1 => self.match_float(pe, prop_idx, &mut vector),
                // 2 => self.match_str(pe, prop_idx, &mut vector),
                _ => panic!("Unsupported prop type: {}", prop_type),
            }
        }
        return vector;
    }

    #[inline(always)]
    pub fn match_int_other(
        &self,
        pe: &PacketEntsOutput,
        wanted_pidx: i32,
        v: &mut Vec<(f32, i32, i32, i32)>,
        wanted_entid_type: i32,
    ) {
        for x in &pe.data {
            match wanted_entid_type {
                2 => {
                    if x.prop_inx == wanted_pidx && x.ent_id == 70 {
                        if let PropData::I32(f) = x.data {
                            v.push((f as f32, x.prop_inx, pe.tick, x.ent_id));
                        }
                    }
                }
                3 => {
                    if x.prop_inx == wanted_pidx && x.ent_id == 71 {
                        if let PropData::I32(f) = x.data {
                            v.push((f as f32, x.prop_inx, pe.tick, x.ent_id));
                        }
                    }
                }
                _ => panic!(":/"),
            }
        }
    }
    #[inline(always)]
    pub fn match_float(&self, pe: &PacketEntsOutput, pidx: i32, v: &mut Vec<(f32, i32, i32)>) {
        for x in &pe.data {
            if x.prop_inx == pidx && x.ent_id < 64 {
                match &x.data {
                    PropData::F32(s) => {
                        v.push((*s as f32, pe.tick, x.ent_id));
                    }
                    _ => {}
                }
            }
        }
    }
    #[inline(always)]
    pub fn match_int(&self, pe: &PacketEntsOutput, pidx: i32, v: &mut Vec<(f32, i32, i32)>) {
        for x in &pe.data {
            if x.prop_inx == pidx && x.ent_id < 64 {
                match &x.data {
                    PropData::I32(s) => {
                        v.push((*s as f32, pe.tick, x.ent_id));
                    }
                    _ => {}
                }
            }
        }
    }
    #[inline(always)]
    pub fn match_str(&self, pe: &PacketEntsOutput, pidx: i32, v: &mut Vec<(String, i32, i32)>) {
        for x in &pe.data {
            if x.prop_inx == pidx && x.ent_id < 64 {
                match &x.data {
                    PropData::String(s) => {
                        v.push((s.to_owned(), pe.tick, x.ent_id));
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn find_wanted_value(&self, data: &mut Vec<&(f32, i32, i32)>, tick: i32) -> Option<f32> {
        data.sort_by_key(|x| x.1);
        data.reverse();

        for j in &mut *data {
            if j.1 <= tick {
                return Some(j.0);
            }
        }
        None
    }

    pub fn find_wanted_values(
        &self,
        data: &mut Vec<&(f32, i32, i32)>,
        ticks: &Vec<i32>,
        out_data: &mut Vec<f32>,
    ) {
        for i in &*data {
            if i.0 < 1000.0 {
                println!("{:?}", i);
            }
        }
        // Fast due to mostly sorted already
        data.sort_by_key(|x| x.1);

        for tick in ticks {
            let idx = data.partition_point(|x| x.1 <= *tick);
            if idx > 0 {
                out_data.push(data[idx - 1].0);
            } else {
                out_data.push(data[0].0);
            }
        }
    }
    pub fn find_wanted_values2(
        &self,
        data: &mut Vec<(f32, i32, i32, i32)>,
        ticks: &Vec<i32>,
    ) -> Vec<f32> {
        if data.len() == 0 {
            return vec![];
        }
        let mut output = Vec::with_capacity(ticks.len());
        // Fast due to mostly sorted already
        data.sort_by_key(|x| x.2);

        for tick in ticks {
            let idx = data.partition_point(|x| x.2 <= *tick);
            if idx > 0 {
                output.push(data[idx - 1].0);
            } else {
                output.push(data[0].0);
            }
        }
        output
    }

    pub fn str_name_to_first_idx(&self, str_name: String) -> Option<i32> {
        // Finds pidx that maps to array prop with ending .000
        // The array ending means which entid is in question.
        // The pidxs for same array prop are contiguous ie.
        // If manager@m_iKills.000 has pidx of 50 then
        // manager@m_iKills.001 has pidx 51 and so on
        let splitted: Vec<&str> = str_name.split("@").collect();
        let prefix = splitted[0];

        let sv_map = match prefix {
            "player" => self.maps.serverclass_map.get(&PLAYER_CLSID).unwrap(),
            "manager" => self.maps.serverclass_map.get(&MANAGER_CLSID).unwrap(),
            "rules" => self.maps.serverclass_map.get(&RULES_CLSID).unwrap(),
            "team" => self.maps.serverclass_map.get(&TEAM_CLSID).unwrap(),
            _ => panic!("unkown prefix: {}", prefix),
        };
        let wanted_name = str_name.to_owned() + &".000";
        for (idx, prop) in sv_map.props.iter().enumerate() {
            if "manager@".to_string() + &prop.table + "." + &prop.name == wanted_name {
                return Some(idx as i32);
            }
        }
        return None;
    }

    pub fn str_name_to_idx(&self, str_name: String) -> Option<i32> {
        let prefix: Vec<&str> = str_name.split("@").collect();
        match prefix[0] {
            "player" => match str_name.as_str() {
                "player@m_vecOrigin_X" => return Some(10000),
                "player@m_vecOrigin_Y" => return Some(10001),
                _ => {
                    let sv_map = self.maps.serverclass_map.get(&40).unwrap();
                    for (idx, prop) in sv_map.props.iter().enumerate() {
                        if prop.table.to_owned() + "." + &prop.name.to_owned() == prefix[1] {
                            return Some(idx as i32);
                        }
                    }
                    return None;
                }
            },
            _ => panic!("NON PLAYER PREFIX IN PLAYER FUNC: {}", prefix[0]),
        }
    }
    pub fn find_one_value(
        &self,
        results: &Vec<JobResult>,
        prop_name: String,
        tick: i32,
        players: &Players,
        userid: u32,
    ) -> Option<f32> {
        let idx = self.str_name_to_idx(prop_name.clone()).unwrap();
        let mut filtered = self.filter_jobs_by_pidx(results, idx, &prop_name);

        let mut filtered_uid: Vec<&(f32, i32, i32)> = filtered
            .iter()
            .filter(|x| Some(x.2 as u32) == players.uid_to_entid_tick(userid, tick))
            .collect();

        filtered_uid.sort_by_key(|x| x.1);
        self.find_wanted_value(&mut filtered_uid, tick)
    }

    pub fn find_other_values(
        &self,
        results: &Vec<JobResult>,
        prop_name: String,
        ticks: &Vec<i32>,
        players: &Players,
    ) -> (Vec<f32>, Vec<u64>, Vec<String>, Vec<i32>) {
        let mut v = vec![];
        let mut steamids = players.get_steamids();
        for sid in steamids {
            let lower_b = self.str_name_to_first_idx(prop_name.clone());
            let mut filtered =
                self.filter_jobs_by_pidx_other(results, lower_b.unwrap(), &prop_name, players, sid);
            filtered.sort_by_key(|x| x.2);

            let out = self.find_wanted_values2(&mut filtered, &ticks);
            v.push((sid, out))
        }

        let mut out = vec![];
        let mut ids = vec![];
        let mut out_ticks = vec![];
        let mut names = vec![];

        v.sort_by_key(|x| x.0);
        for t in v {
            if t.1.len() > 0 && t.0 != 0 {
                out.extend(t.1);
                ids.extend(vec![t.0; ticks.len()]);
                out_ticks.extend(ticks.clone());
                names.extend(vec![players.steamid_to_name(t.0); ticks.len()]);
            }
        }
        (out, ids, names, out_ticks)
    }

    #[inline(always)]
    pub fn find_multiple_values(
        &self,
        results: &Vec<JobResult>,
        prop_name: String,
        ticks: &Vec<i32>,
        players: &Players,
    ) -> (Vec<f32>, Vec<u64>, Vec<String>, Vec<i32>) {
        // Takes Vec<JobResult> and creates our vectors that
        // then get converted to Series.
        let idx = match self.str_name_to_idx(prop_name.clone()) {
            Some(i) => i,
            None => return (vec![], vec![], vec![], vec![]),
        };
        let mut tasks: Vec<(u64, Vec<&(f32, i32, i32)>)> = vec![];
        let mut ids = Vec::with_capacity(ticks.len() * 10);
        let mut out_ticks = Vec::with_capacity(ticks.len() * 10);
        let mut names = Vec::with_capacity(ticks.len() * 10);
        let mut out_data: Vec<f32> = Vec::with_capacity(ticks.len() * 10);

        let mut filtered = self.filter_jobs_by_pidx(results, idx, &prop_name);
        let grouped_by_sid = filtered
            .iter()
            .into_group_map_by(|x| players.eid_to_sid(x.2 as u32, x.1));
        for (sid, data) in grouped_by_sid {
            if sid != None && sid != Some(0) {
                tasks.push((sid.unwrap(), data));
            }
        }
        // Check that we have data for every player
        let found_sids: Vec<u64> = tasks.iter().map(|x| x.0).collect();
        let all_sids = players.get_steamids();
        for sid in all_sids {
            if !found_sids.contains(&sid) && sid != 0 {
                tasks.push((sid, vec![&(0.0, 0, 0)]));
            }
        }
        tasks.sort_by_key(|x| x.0);
        // Create metadata columns
        for i in &tasks {
            ids.extend(vec![i.0; ticks.len()]);
            out_ticks.extend(ticks.clone());
            names.extend(vec![players.steamid_to_name(i.0); ticks.len()]);
        }
        // Pass mut ref to function for perf reasons
        for (_, data) in &mut tasks {
            self.find_wanted_values(data, ticks, &mut out_data);
        }
        (out_data, ids, names, out_ticks)
    }

    pub fn find_weapon_values(
        &self,
        results: &Vec<JobResult>,
        og_ticks: &Vec<i32>,
        players: &Players,
        pidx: i32,
    ) -> Vec<f32> {
        let tickss: Vec<i32> = (0..700000).collect();
        let (weapon_handles, xuids, name, ticks) = self.find_multiple_values(
            results,
            "player@DT_BaseCombatCharacter.m_hActiveWeapon".to_string(),
            &tickss,
            players,
        );

        let mut weap_map = HashMap::default();

        for (weap, xuid, tick) in izip!(&weapon_handles, &xuids, &ticks) {
            weap_map
                .entry(tick)
                .or_insert(vec![])
                .push((xuid, *weap as i32 & 0x7FF));
        }

        let eids: Vec<i32> = weapon_handles.iter().map(|x| *x as i32 & 0x7FF).collect();
        let uniq_eids: HashSet<i32> = HashSet::from_iter(eids.iter().map(|x| *x).dedup());

        println!("UE {:?}", uniq_eids);

        let mut v = vec![];
        for x in results {
            if let JobResult::PacketEntities(pe) = x {
                v.push(pe);
            }
        }
        let mut filtered = vec![];
        for i in v {
            for j in &i.data {
                if uniq_eids.contains(&j.ent_id) && j.prop_inx == pidx {
                    match j.data {
                        PropData::I32(int) => {
                            filtered.push((int as f32, i.tick, j.ent_id));
                        }
                        _ => {}
                    }
                }
            }
        }

        let g = filtered
            .iter()
            .into_group_map_by(|x| Parser::weapid_to_person(&weap_map, x.1, x.2));

        let mut tasks = vec![];
        for (sid, data) in g {
            if sid != None && sid != Some(0) {
                tasks.push((sid.unwrap(), data));
            }
        }

        let found_sids: Vec<u64> = tasks.iter().map(|x| x.0).collect();
        let all_sids = players.get_steamids();

        for sid in all_sids {
            if !found_sids.contains(&sid) && sid != 0 {
                tasks.push((sid, vec![&(0.0, 0, 0)]));
            }
        }
        tasks.sort_by_key(|x| x.0);

        for i in &filtered {
            println!("***");
            println!("{:?}", i);
            Parser::weapid_to_person(&weap_map, i.1, i.2);
            println!("{:?}", &weap_map[&i.1])
        }
        let mut out: Vec<f32> = vec![];

        for (k, mut v) in tasks {
            println!("{:?} {:?}", k, v.len());
            self.find_wanted_values(&mut v, &og_ticks, &mut out);
        }
        return out;
    }
    fn weapid_to_person(
        weap_map: &HashMap<&i32, Vec<(&u64, i32)>>,
        tick: i32,
        entid: i32,
    ) -> Option<u64> {
        match weap_map.get(&tick) {
            Some(m) => {
                for i in m {
                    if i.1 == entid {
                        return Some(*i.0);
                    }
                }
            }
            None => println!("no tick found"),
        }
        return None;
    }
}
*/
