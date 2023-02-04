use std::collections::HashMap;
use std::time::Instant;

use super::demo_parsing::{GameEvent, NameDataPair};
use super::utils::TYPEHM;
use crate::parsing::cache::cache_reader::ReadCache;
use crate::parsing::demo_parsing::entities::PacketEntsOutput;
use crate::parsing::demo_parsing::KeyData;
use crate::parsing::parser::*;
use crate::parsing::players::Players;
pub use crate::parsing::variants::*;
use derive_more::TryInto;
use itertools::Itertools;
use polars::df;
use polars::export::regex::internal::Inst;
use polars::prelude::{DataFrame, Int64Type, NamedFrom, NamedFromOwned};
use polars::series::Series;

#[derive(Debug, Clone)]
pub struct ExtraEventRequest {
    pub tick: i32,
    pub userid: u32,
    pub prop: String,
}

impl Parser {
    pub fn compute_jobs_no_cache(&mut self) -> Vec<JobResult> {
        let results: Vec<JobResult> = self.parse_blueprints();
        results
    }

    pub fn compute_jobs_with_cache(&mut self, cache: &mut ReadCache) -> ParsingOutPut {
        // Need to parse players to understand cache. This is fast
        let player_results: Vec<JobResult> = self.parse_blueprints();
        let players = Players::new(&player_results);
        let ticks = self.get_wanted_ticks();

        let wanted_bytes = cache.find_wanted_bytes(
            &ticks,
            &self.settings.wanted_props,
            &players.get_uids(),
            &self.maps.serverclass_map,
            &players,
        );
        //println!("{}", wanted_bytes.len());

        self.parse_bytes(wanted_bytes);
        let results: Vec<JobResult> = self.parse_blueprints();
        let before = Instant::now();
        //println!("{}", results.len());
        let df = self.create_series(&results, &self.settings.wanted_props, &ticks, &players);
        println!("here {:2?}", before.elapsed());

        let events = if self.settings.only_events {
            let event_ticks =
                cache.find_game_event_ticks("player_death".to_string(), &self.maps.event_map);
            self.parse_bytes(event_ticks);
            let results: Vec<JobResult> = self.parse_blueprints();
            self.get_game_events(&results, &players, cache)
        } else {
            vec![]
        };

        ParsingOutPut {
            df: df,
            events: events,
        }
    }

    fn filter_to_vec<Wanted>(v: impl IntoIterator<Item = impl TryInto<Wanted>>) -> Vec<Wanted> {
        v.into_iter().filter_map(|x| x.try_into().ok()).collect()
    }

    fn temp(pairs: Vec<&NameDataPair>, name: &String) -> Series {
        let only_data: Vec<KeyData> = pairs.iter().map(|x| x.data.clone()).collect();
        match pairs[0].data_type {
            1 => Series::new(name, &Parser::filter_to_vec::<String>(only_data)),
            2 => Series::new(name, &Parser::filter_to_vec::<f32>(only_data)),
            3 => Series::new(name, &Parser::filter_to_vec::<i64>(only_data)),
            4 => Series::new(name, &Parser::filter_to_vec::<i64>(only_data)),
            5 => Series::new(name, &Parser::filter_to_vec::<i64>(only_data)),
            6 => Series::new(name, &Parser::filter_to_vec::<bool>(only_data)),
            7 => Series::new(name, &Parser::filter_to_vec::<u64>(only_data)),
            _ => panic!("Keydata got unknown type: {}", pairs[0].data_type),
        }
    }

    fn series_from_events(&self, events: Vec<GameEvent>) -> Vec<Series> {
        // Example [Hashmap<"distance": 21.0>, Hashmap<"distance": 24.0>, Hashmap<"name": "Steve">]
        // ->
        // Hashmap<"distance": [21.0, 24.0], "name": ["Steve"]>,
        // -> Series::new("distance", [21.0, 24.0]) <-- needs to be mapped as "f32" not as enum(KeyData)
        let pairs: Vec<NameDataPair> = events.iter().map(|x| x.fields.clone()).flatten().collect();
        let per_key_name = pairs.iter().into_group_map_by(|x| &x.name);
        let mut series = vec![];
        for (name, vals) in per_key_name {
            series.push(Parser::temp(vals, name));
        }
        series
    }

    fn convert_to_requests(
        &self,
        ticks: Vec<i64>,
        userids: Vec<i64>,
        attackers: Vec<i64>,
        wanted_props: &Vec<String>,
    ) -> Vec<ExtraEventRequest> {
        let mut requests = vec![];
        for prop in wanted_props {
            if ticks.len() > 0 && attackers.len() > 0 {
                for (tick, uid) in ticks.iter().zip(&attackers) {
                    requests.push(ExtraEventRequest {
                        tick: *tick as i32 - 1,
                        userid: *uid as u32,
                        prop: prop.to_string(),
                    });
                }
            }
            if ticks.len() > 0 && userids.len() > 0 {
                for (tick, uid) in ticks.iter().zip(&userids) {
                    requests.push(ExtraEventRequest {
                        tick: *tick as i32 - 1,
                        userid: *uid as u32,
                        prop: prop.to_string(),
                    });
                }
            }
        }
        requests
    }

    fn fill_wanted_extra_props(
        &self,
        series: &Vec<Series>,
        wanted_props: &Vec<String>,
    ) -> Vec<ExtraEventRequest> {
        let mut ticks: Vec<i64> = vec![];
        let mut userids: Vec<i64> = vec![];
        let mut attackers: Vec<i64> = vec![];

        for s in series {
            match s.name() {
                "tick" => ticks.extend(s.i64().unwrap().into_no_null_iter()),
                //"userid" => userids.extend(s.i64().unwrap().into_no_null_iter()),
                "attacker" => attackers.extend(s.i64().unwrap().into_no_null_iter()),
                _ => {}
            }
        }
        self.convert_to_requests(ticks, userids, attackers, &wanted_props)
    }

    fn get_game_events(
        &mut self,
        results: &Vec<JobResult>,
        players: &Players,
        cache: &mut ReadCache,
    ) -> Vec<Series> {
        let event_id = cache
            .event_name_to_id(&"player_death".to_string(), &self.maps.event_map)
            .unwrap();

        let mut v = vec![];
        for x in results {
            if let JobResult::GameEvents(ge) = x {
                if ge[0].id == event_id {
                    v.push(ge[0].clone());
                }
            }
        }

        let mut series = self.series_from_events(v);
        let requests = self.fill_wanted_extra_props(&series, &self.settings.wanted_props.clone());
        let extra_bytes = cache.find_request_bytes(&requests, &self.maps.serverclass_map, players);
        self.parse_bytes(extra_bytes);
        let results = self.parse_blueprints();
        let s = self.find_requested_vals(requests, &results, &players);
        series.extend(s);
        series
    }

    fn find_requested_vals(
        &mut self,
        requests: Vec<ExtraEventRequest>,
        results: &Vec<JobResult>,
        players: &Players,
    ) -> Vec<Series> {
        let mut series = vec![];
        let request_per_prop = requests.iter().into_group_map_by(|x| x.prop.clone());
        let before = Instant::now();
        for (name, requests) in request_per_prop {
            let mut v = vec![];
            for request in requests {
                v.push(self.find_one_value(
                    results,
                    request.prop.clone(),
                    request.tick,
                    players,
                    request.userid,
                ));
            }
            series.push(Series::new(&name, v))
        }
        println!("{:2?}", before.elapsed());
        series
    }

    fn get_wanted_ticks(&self) -> Vec<i32> {
        // If len wanted ticks == 0 then all ticks should be parsed
        match self.settings.wanted_ticks.len() {
            0 => (0..self.settings.playback_frames as i32).collect(),
            _ => self.settings.wanted_ticks.clone(),
        }
    }

    fn create_series(
        &self,
        results: &Vec<JobResult>,
        props: &Vec<String>,
        ticks: &Vec<i32>,
        players: &Players,
    ) -> Vec<Series> {
        let mut all_series = vec![];
        for (idx, prop) in props.iter().enumerate() {
            let (out, labels, ticks) =
                self.find_multiple_values(&results, prop.to_owned(), &ticks, &players);

            let s = Series::from_vec(prop, out);
            if idx == 0 {
                let ls = Series::from_vec("steamid", labels);
                let ts = Series::from_vec("ticks", ticks);
                all_series.push(ls);
                all_series.push(ts);
            }
            all_series.push(s);
        }
        all_series
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

        let mut vector = vec![];
        let prop_type = TYPEHM.get(&prop_name).unwrap();
        for pe in v {
            match prop_type {
                0 => self.match_int(pe, prop_idx, &mut vector),
                1 => self.match_float(pe, prop_idx, &mut vector),
                // 2 => self.match_str(pe, prop_idx, &mut vector),
                _ => panic!("Unsupported prop type: {}", prop_type),
            }
        }
        vector
    }
    #[inline(always)]
    pub fn match_float(&self, pe: &PacketEntsOutput, pidx: i32, v: &mut Vec<(f32, i32, i32)>) {
        for x in &pe.data {
            if x.prop_inx == pidx && x.ent_id < 64 {
                if let PropData::F32(f) = x.data {
                    v.push((f, pe.tick, x.ent_id));
                }
            }
        }
    }
    #[inline(always)]
    pub fn match_int(&self, pe: &PacketEntsOutput, pidx: i32, v: &mut Vec<(f32, i32, i32)>) {
        for x in &pe.data {
            if x.prop_inx == pidx && x.ent_id < 64 {
                if let PropData::I32(i) = x.data {
                    v.push((i as f32, pe.tick, x.ent_id));
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
    ) -> Vec<f32> {
        let mut output = Vec::with_capacity(ticks.len());
        // Fast due to mostly sorted already
        data.sort_by_key(|x| x.1);

        for tick in ticks {
            let idx = data.partition_point(|x| x.1 <= *tick);
            if idx > 0 {
                output.push(data[idx - 1].0);
            } else {
                output.push(data[0].0);
            }
        }
        output
    }

    pub fn str_name_to_idx(&self, str_name: String) -> Option<i32> {
        if str_name == "m_vecOrigin_X" {
            return Some(10000);
        }
        if str_name == "m_vecOrigin_Y" {
            return Some(10001);
        }
        let sv_map = self.maps.serverclass_map.get(&40).unwrap();
        for (idx, prop) in sv_map.props.iter().enumerate() {
            if prop.table.to_owned() + "." + &prop.name.to_owned() == str_name {
                return Some(idx as i32);
            }
        }
        None
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

    #[inline(always)]
    pub fn find_multiple_values(
        &self,
        results: &Vec<JobResult>,
        prop_name: String,
        ticks: &Vec<i32>,
        players: &Players,
    ) -> (Vec<f32>, Vec<u64>, Vec<i32>) {
        let idx = self.str_name_to_idx(prop_name.clone()).unwrap();
        let mut filtered = self.filter_jobs_by_pidx(results, idx, &prop_name);
        filtered.sort_by_key(|x| x.1);

        let grouped_by_sid = filtered
            .iter()
            .into_group_map_by(|x| players.eid_to_sid(x.2 as u32, x.1));

        let mut tasks: Vec<(u64, Vec<&(f32, i32, i32)>)> = vec![];
        let mut labels = vec![];
        let mut out_ticks = vec![];

        for (sid, data) in grouped_by_sid {
            if sid != None && sid != Some(0) {
                tasks.push((sid.unwrap(), data));
            }
        }

        tasks.sort_by_key(|x| x.0);

        for i in &tasks {
            labels.extend(vec![i.0; ticks.len()]);
            out_ticks.extend(ticks.clone());
        }

        let out: Vec<f32> = tasks
            .iter_mut()
            .flat_map(|(_, data)| self.find_wanted_values(data, ticks))
            .collect();

        (out, labels, out_ticks)
    }
}
