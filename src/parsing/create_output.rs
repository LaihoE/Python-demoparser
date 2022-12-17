use super::data_table::ServerClasses;
use super::entities::PacketEntsOutput;
use super::game_events::GameEvent;
use super::stringtables::StringTable;
use super::stringtables::UserInfo;
use crate::parsing::columnmapper::EntColMapper;
use crate::parsing::data_table::ServerClass;
use crate::parsing::game_events;
use crate::parsing::parser::JobResult;
use crate::parsing::parser::ParsingMaps;
pub use crate::parsing::variants::*;
use crate::Parser;
use ahash::HashMap;
use ahash::HashSet;
use polars::frame::DataFrame;
use polars::prelude::*;
use polars::series::Series;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::*;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

// Todo entid when disconnected
fn eid_sid_stack(eid: u32, max_ticks: i32, players: &Vec<&UserInfo>) -> Vec<u64> {
    let mut eids: HashMap<u32, Vec<(u64, i32)>> = HashMap::default();
    for player in players {
        eids.entry(player.entity_id)
            .or_insert(vec![])
            .push((player.xuid, player.tick));
    }

    let steamids = match eids.get(&eid) {
        None => return vec![0; max_ticks as usize],
        Some(steamids) => steamids,
    };
    // Most of the time it's this simple (>95%)
    if steamids.len() == 1 {
        return vec![steamids[0].0; max_ticks as usize];
    }
    // But can also be this complicated when entids map to different players
    let mut ticks = vec![];
    let mut player_idx = 0;
    for tick in 0..max_ticks {
        if tick < steamids[player_idx].1 {
            ticks.push(steamids[player_idx].0);
        } else {
            if player_idx == steamids.len() - 1 {
                ticks.push(steamids[player_idx].0);
            } else {
                ticks.push(steamids[player_idx].0);
                player_idx += 1;
            }
        }
    }
    ticks
}

pub fn filter_jobresults(
    jobs: &Vec<JobResult>,
) -> (Vec<&PacketEntsOutput>, Vec<&GameEvent>, Vec<&UserInfo>) {
    /*
    Groups jobresults by their type
    */
    let before = Instant::now();
    let mut packet_ents = vec![];
    let mut game_events = vec![];
    let mut stringtables = vec![];

    for j in jobs {
        match j {
            JobResult::PacketEntities(p) => match p {
                Some(p) => packet_ents.push(p),
                None => {}
            },
            JobResult::GameEvents(ge) => {
                game_events.extend(ge);
            }
            JobResult::StringTables(st) => {
                stringtables.extend(st);
            }
            _ => {}
        }
    }
    println!("FILTERING TOOK: {:2?}", before.elapsed());
    (packet_ents, game_events, stringtables)
}

impl Parser {
    pub fn insert_props_into_df(
        &self,
        packet_ents: Vec<&PacketEntsOutput>,
        max_ticks: usize,
        int_props: &Vec<i32>,
        df: &mut Vec<Option<f32>>,
        ecm: &EntColMapper,
    ) {
        let before = Instant::now();

        // Map prop idx into its column.
        let col_mapping = create_idx_col_mapping(&int_props);
        // For every packetEnt message during game

        for packet_ent_msg in packet_ents {
            // For every entity in the message
            for single_ent in &packet_ent_msg.data {
                // For every updated value for the entity
                for prop in single_ent {
                    match prop.data {
                        PropData::F32(f) => {
                            let prop_col = col_mapping[&prop.prop_inx];
                            let player_col = ecm.get_col(prop.ent_id as u32, packet_ent_msg.tick);
                            let tick = ecm.get_tick(packet_ent_msg.tick);
                            df[prop_col * player_col * tick] = Some(f as f32);
                        }
                        PropData::I32(i) => {
                            let prop_col = col_mapping[&prop.prop_inx];
                            let player_col = ecm.get_col(prop.ent_id as u32, packet_ent_msg.tick);
                            let tick = ecm.get_tick(packet_ent_msg.tick);
                            df[prop_col * player_col * tick] = Some(i as f32);
                        }
                        // Todo string columns
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn get_raw_df(
        &mut self,
        jobs: &Vec<JobResult>,
        parser_maps: Arc<RwLock<ParsingMaps>>,
        df: &mut Vec<Option<f32>>,
        max_ticks: usize,
    ) -> Vec<Series> {
        // Group jobs by type
        let (packet_ents, game_events, stringtables) = filter_jobresults(jobs);

        let ecm = EntColMapper::new(&stringtables, &self.settings.wanted_ticks);

        // let ent_mapping = ent_col_mapping(&stringtables);

        let ticks: Vec<i32> = (0..max_ticks).into_iter().map(|t| t as i32).collect();
        let int_props = str_props_to_int_props(&self.settings.wanted_props, parser_maps.clone());
        let str_names = col_str_mapping(&int_props, parser_maps.clone());

        self.insert_props_into_df(packet_ents, max_ticks, &int_props, df, &ecm);

        let mut series_players = vec![];
        let mut ticks_col: Vec<i32> = vec![];
        let mut steamids_col: Vec<u64> = vec![];
        let before = Instant::now();

        for (propcol, prop_name) in str_names.iter().enumerate() {
            let mut this_prop_col: Vec<Option<f32>> = Vec::with_capacity(10 * max_ticks);

            for entid in 0..10 {
                // Metadata
                let sids = eid_sid_stack(entid as u32, (max_ticks) as i32, &stringtables);
                if propcol == 0 {
                    ticks_col.extend(&ticks);
                    steamids_col.extend(sids);
                }
                // Props
                fill_none_with_most_recent(&mut df[propcol * entid..propcol * entid + max_ticks]);
                this_prop_col.extend(&df[propcol * entid..propcol * entid + max_ticks]);
            }
            let props = Series::new(prop_name, &this_prop_col);
            // props[44] = 4;
            series_players.push(props);
        }
        let steamids = Series::new("steamids", steamids_col);
        let ticks = Series::new("ticks", ticks_col);

        series_players.push(steamids);
        series_players.push(ticks);

        println!("{:2?}", before.elapsed());

        series_players
    }
}

#[inline(always)]
pub fn fill_none_with_most_recent(v: &mut [std::option::Option<f32>]) {
    /*
    Called ffil in pandas, not sure about polars.

    For example:
    Input: Vec![1, 2, 3, None, None, 6]
    Output: Vec![1, 2, 3, 3, 3, 6]
    */
    let mut last_val: Option<f32> = None;
    for v in v.iter_mut() {
        if v.is_some() {
            last_val = *v;
        }
        if v.is_none() {
            *v = last_val
        }
    }
}
fn create_idx_col_mapping(prop_indicies: &Vec<i32>) -> HashMap<i32, usize> {
    /*
    Create mapping from property index into column index.
    This is needed because prop indicies might be:
    24, 248, 354 and we can't be creating 354 columns so
    we map it into 0,1,2..
    */
    let mut idx_pos: HashMap<i32, usize> = HashMap::default();
    for (cnt, p_idx) in prop_indicies.iter().enumerate() {
        idx_pos.insert(*p_idx, cnt);
    }
    idx_pos
}
fn col_str_mapping(col_indicies: &Vec<i32>, parser_maps: Arc<RwLock<ParsingMaps>>) -> Vec<String> {
    /*
    Maps column index to it's human readable name
    */
    let parser_maps_read = parser_maps.read().unwrap();
    let serverclass_map = parser_maps_read.serverclass_map.as_ref().unwrap();

    let mut str_names = vec![];
    let props = &serverclass_map.player.props;
    for ci in col_indicies {
        let s = &props[*ci as usize];
        str_names.push(s.name.to_owned());
    }
    str_names
}

pub fn str_props_to_int_props(
    str_props: &Vec<String>,
    parser_maps: Arc<RwLock<ParsingMaps>>,
) -> Vec<i32> {
    let parser_maps_read = parser_maps.read().unwrap();
    let serverclass_map = parser_maps_read.serverclass_map.as_ref().unwrap();
    let int_props: Vec<i32> = str_props
        .iter()
        .map(|p| str_prop_to_int(p, &serverclass_map))
        .collect();
    int_props
}

fn str_prop_to_int(wanted_prop: &str, serverclass_map: &ServerClasses) -> i32 {
    /*
    Maps string names to their prop index (used in packet ents)
    For example m_angEyeAngles[0] --> 20
    Mainly need to pay attention to manager props comming from different
    serverclass
    */
    let sv_cls = &serverclass_map.player;

    match sv_cls.props.iter().position(|x| x.name == wanted_prop) {
        Some(idx) => idx as i32,
        None => panic!("Could not find prop idx for {}", wanted_prop),
    }
}
