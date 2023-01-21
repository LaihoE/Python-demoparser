use super::entities::PacketEntsOutput;
use super::game_events::GameEvent;
use super::stringtables::StringTable;
use super::stringtables::UserInfo;
use crate::parsing::columnmapper::EntColMapper;
use crate::parsing::data_table::ServerClass;
use crate::parsing::game_events;
use crate::parsing::parser::JobResult;
pub use crate::parsing::variants::*;
use crate::Parser;
use ahash::HashMap;
use ahash::HashSet;
use ndarray::s;
use ndarray::ArrayBase;
use ndarray::Dim;
use ndarray::OwnedRepr;
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
            JobResult::PacketEntities(p) => packet_ents.push(p),

            JobResult::GameEvents(ge) => {
                game_events.extend(ge);
            }
            JobResult::StringTables(st) => {
                stringtables.extend(st);
            }
            _ => {}
        }
    }
    /*
    println!(
        "pe {} ge {} st {}",
        packet_ents.len(),
        game_events.len(),
        stringtables.len()
    );
    */
    (packet_ents, game_events, stringtables)
}

impl Parser {
    pub fn insert_props_into_df(
        &self,
        packet_ents: Vec<&PacketEntsOutput>,
        max_ticks: usize,
        df: &mut ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>>,
        ecm: &EntColMapper,
    ) {
        let before = Instant::now();
        // Map prop idx into its column.

        // For every packetEnt message during game
        for packet_ent_msg in packet_ents {
            // For every entity in the message
            for prop in &packet_ent_msg.data {
                // For every updated value for the entity
                // for prop in single_ent {
                match prop.data {
                    PropData::F32(f) => {
                        // println!("NOW IDX {:?} {:?}", prop.prop_inx, prop.data);
                        let prop_col = ecm.get_prop_col(&prop.prop_inx);
                        let player_col =
                            ecm.get_player_col(prop.ent_id as u32, packet_ent_msg.tick);
                        df[[player_col, prop_col, packet_ent_msg.tick as usize]] = f as f32;
                    }
                    PropData::I32(i) => {
                        let prop_col = ecm.get_prop_col(&prop.prop_inx);
                        let player_col =
                            ecm.get_player_col(prop.ent_id as u32, packet_ent_msg.tick);
                        // let tick = ecm.get_tick(packet_ent_msg.tick);
                        df[[player_col, prop_col, packet_ent_msg.tick as usize]] = i as f32;
                    }
                    // Todo string columns
                    _ => {}
                }
                // }
            }
        }
    }
    pub fn create_game_events(
        &mut self,
        game_events: &Vec<&GameEvent>,
        ecm: EntColMapper,
        df: &mut ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>>,
    ) {
        for ev in game_events {
            for field in &ev.fields {
                if field.name == "userid" {
                    //let ent_id = ecm.entid_from_uid(&field.data);
                    /*
                    let prop_col = col_mapping[&prop.prop_inx];
                    let player_col = ecm.get_col(ent_id, ev.tick);
                    df[[player_col, prop_col, packet_ent_msg.tick as usize]] = i as f32;
                    */
                }
                //println!("{:?}", f);
            }
        }
        // let prop_col = col_mapping[&prop.prop_inx];
        // let player_col = ecm.get_col(prop.ent_id as u32, packet_ent_msg.tick);

        //df[[player_col, prop_col, packet_ent_msg.tick as usize]] = i as f32;

        //let tick = ev.tick;
        //ecm.g
        //println!("{:?}", df[[]])
    }

    pub fn get_raw_df(
        &mut self,
        jobs: &Vec<JobResult>,
        //parser_maps: Arc<RwLock<ParsingMaps>>,
        df: &mut ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>>,
        max_ticks: usize,
    ) -> Vec<Series> {
        // Group jobs by type
        let (packet_ents, game_events, stringtables) = filter_jobresults(jobs);

        let ecm = EntColMapper::new(
            &stringtables,
            &self.settings.wanted_ticks,
            &self.settings.wanted_props,
            max_ticks,
            &self.maps,
        );

        self.insert_props_into_df(packet_ents, max_ticks, df, &ecm);
        //println!("{:?}", df);

        let mut series_players = vec![];
        let mut ticks_col: Vec<i32> = vec![];
        let mut steamids_col: Vec<u64> = vec![];

        fill_none_with_most_recent(df, self.settings.wanted_props.len());

        let all_player_cols: Vec<&usize> = ecm.col_sid_map.keys().into_iter().collect();
        let before = Instant::now();
        let str_names = ecm.idx_pos.clone();
        let ticks: Vec<i32> = (0..max_ticks).into_iter().map(|t| t as i32).collect();

        for (propcol, prop_name) in str_names.iter().enumerate() {
            //println!("PC {:?}", all_player_cols);
            let mut this_prop_col: Vec<f32> = Vec::with_capacity(15 * max_ticks);
            for player_col in 1..15 {
                // Metadata
                if !all_player_cols.contains(&&player_col) {
                    continue;
                }
                if propcol == 0 {
                    ticks_col.extend(&ticks);
                    steamids_col.extend(ecm.get_col_sid_vec(player_col, max_ticks));
                }
                //println!("SLICE {}", &df.slice(s![player_col, propcol, ..]));
                this_prop_col.extend(&df.slice(s![player_col, propcol, ..]));
            }
            let n = str_names[prop_name.0];
            let props = Series::new(&n.to_string(), &this_prop_col);
            //println!("{:?} {}", props, max_ticks);
            series_players.push(props);
        }

        // println!("SERIES: {:2?}", before.elapsed());

        let steamids = Series::new("steamids", steamids_col);
        let ticks = Series::new("ticks", ticks_col);

        series_players.push(steamids);
        series_players.push(ticks);

        self.create_game_events(&game_events, ecm, df);

        series_players
    }
}

#[inline(always)]
pub fn fill_none_with_most_recent(
    df: &mut ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>>,
    n_props: usize,
) {
    /*
    Called ffil in pandas, not sure about polars.

    For example:
    Input: Vec![1, 2, 3, None, None, 6]
    Output: Vec![1, 2, 3, 3, 3, 6]
    */
    let before = Instant::now();
    for propcol in 0..n_props {
        for entid in 0..12 {
            let mut last = 0.0;
            let s = &mut df.slice_mut(s![entid, propcol, ..]);

            for x in s.iter_mut() {
                if x != &0.0 {
                    //println!("{}", x);
                }
                if x == &0.0 {
                    *x = last;
                } else {
                    last = *x;
                }
            }
        }
    }
    //println!("FFIL TOOK: {:2?}", before.elapsed());
}
