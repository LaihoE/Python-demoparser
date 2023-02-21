/*
use std::sync::Arc;

use crate::parsing::cache::cache_reader::ReadCache;
use crate::parsing::demo_parsing::entities_cache_only::ClsEidMapper;
use crate::parsing::demo_parsing::EidClsHistoryEntry;
use crate::parsing::parser::*;
use crate::parsing::players::Players;
pub use crate::parsing::variants::*;
use polars::prelude::{NamedFrom, NamedFromOwned};
use polars::series::Series;

// Its not worth to filter if we want too many ticks of data
// --> Just parse everything
const TICKS_FILTER_LIMIT: usize = 100000000;

#[derive(Debug, Clone)]
pub struct ExtraEventRequest {
    pub tick: i32,
    pub userid: u32,
    pub prop: String,
}

impl Parser {
    pub fn compute_jobs_no_cache(&mut self) -> (Vec<JobResult>, Arc<ClsEidMapper>) {
        let results: (Vec<JobResult>, Arc<ClsEidMapper>) = self.parse_blueprints(true, None);
        results
    }
    pub fn other_outputs(
        &mut self,
        cache: &mut ReadCache,
        ticks: &Vec<i32>,
        players: &Players,
        other_props: &Vec<String>,
    ) -> Vec<Series> {
        let mut wanted_bytes = vec![];
        if other_props.len() == 0 {
            return vec![];
        }

        for prop in other_props {
            for i in 0..32 {
                let p = if i < 10 {
                    prop.to_owned() + &".00" + &i.to_string()
                } else {
                    prop.to_owned() + &".0" + &i.to_string()
                };
                cache.read_other_deltas_by_name(&p, &self.maps.serverclass_map, 41);
                wanted_bytes.extend(cache.find_delta_ticks_others(55, p, ticks, players))
            }
        }
        wanted_bytes.sort();
        if wanted_bytes.len() > 0 {
            self.parse_bytes(wanted_bytes);
            let (results, _) = self.parse_blueprints(false, Some(cache.eid_cls_map.clone()));
            let ticks = self.get_wanted_ticks();
            return self.create_series_others(&results, &other_props, &ticks, players);
        }
        //self.parse_bytes(vec![]);
        vec![]
    }

    pub fn compute_jobs_with_cache(&mut self, cache: &mut ReadCache) -> ParsingOutPut {
        // Need to parse players to understand cache. This is fast
        let eid_cls_map = cache.eid_cls_map.clone();

        let (player_results, _) = self.parse_blueprints(false, Some(cache.eid_cls_map.clone()));

        let players = Players::new(&player_results);
        let ticks = self.get_wanted_ticks();

        let mut player_props = vec![];
        let mut other_props = vec![];

        for prop in &self.settings.wanted_props {
            let p: Vec<&str> = prop.split("@").collect();
            if p[0] == "player" {
                player_props.push(prop.clone());
            } else {
                other_props.push(prop.clone());
            }
        }

        if !self.settings.only_events {
            if ticks.len() < TICKS_FILTER_LIMIT {
                let wanted_bytes = cache.find_wanted_bytes(
                    &ticks,
                    &player_props,
                    &players.get_uids(),
                    &self.maps.serverclass_map,
                    &players,
                );
                if wanted_bytes.len() != 0 {
                    self.parse_bytes(wanted_bytes);
                }
            }
        }
        self.parse_bytes(vec![]);
        let (results, _) = self.parse_blueprints(false, Some(cache.eid_cls_map.clone()));

        //let weaps = self.find_weapon_values(&results, &ticks, &players, 497);
        //let ammo = Series::from_vec("Ammo", weaps);

        let defs = self.find_weapon_values(&results, &ticks, &players, 402);
        let weapid = Series::from_vec("weapid", defs);
        //panic!("done");

        //println!("{:?}", results);

        let other_s = self.other_outputs(cache, &ticks, &players, &other_props);
        let mut df = self.create_series(&results, &player_props, &ticks, &players);
        println!("{:?}", df);
        df.extend(other_s);
        //df.push(ammo);
        df.push(weapid);

        let events = if self.settings.only_events {
            cache.read_game_events();
            let event_ticks = cache
                .find_game_event_ticks(self.settings.event_name.to_string(), &self.maps.event_map);
            self.parse_bytes(event_ticks);

            let (results, _) = self.parse_blueprints(false, Some(cache.eid_cls_map.clone()));

            self.get_game_events(&results, &players, cache)
        } else {
            vec![]
        };

        ParsingOutPut {
            df: df,
            events: events,
        }
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
            let (out, ids, names, ticks) =
                self.find_multiple_values(&results, prop.to_owned(), &ticks, &players);

            let s = Series::from_vec(prop, out);
            if idx == 0 {
                let steamids = Series::from_vec("steamid", ids);
                let names = Series::new("name", names);
                let ticks = Series::from_vec("tick", ticks);
                all_series.push(steamids);
                all_series.push(ticks);
                all_series.push(names);
            }
            all_series.push(s);
        }
        all_series
    }
    fn create_series_others(
        &self,
        results: &Vec<JobResult>,
        props: &Vec<String>,
        ticks: &Vec<i32>,
        players: &Players,
    ) -> Vec<Series> {
        let mut all_series = vec![];
        for (idx, prop) in props.iter().enumerate() {
            let (out, labels, names, ticks) =
                self.find_other_values(&results, prop.to_owned(), &ticks, &players);

            let s = Series::from_vec(prop, out);
            if idx == 0 {
                let ls = Series::from_vec("steamid", labels);
                let names = Series::new("name", names);
                let ts = Series::from_vec("tick", ticks);
                all_series.push(ls);
                all_series.push(ts);
                all_series.push(names);
            }
            all_series.push(s);
        }
        all_series
    }
}
*/
