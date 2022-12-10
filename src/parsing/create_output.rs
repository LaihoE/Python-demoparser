use crate::parsing::data_table::ServerClass;
use crate::parsing::parser::JobResult;
use crate::parsing::parser::ParsingMaps;
pub use crate::parsing::variants::*;
use crate::Parser;
use ahash::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

use super::entities::PacketEntsOutput;

// Todo entid when disconnected
fn eid_sid_stack(eid: u32, max_ticks: i32, players: &HashMap<u32, Vec<(u64, i32)>>) -> Vec<u64> {
    /*
        let mut eids: HashMap<u32, Vec<(u64, i32)>> = HashMap::default();
        for player in players {
            //println!("({} {} {})", player.name, player.entity_id, player.tick);
            eids.entry(player.entity_id)
                .or_insert(vec![])
                .push((player.xuid, player.tick));
        }
    */
    let steamids = match players.get(&eid) {
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

pub fn filter_entity_messages(jobs: &Vec<JobResult>) -> Vec<&PacketEntsOutput> {
    let mut data = vec![];

    for j in jobs {
        match j {
            JobResult::PacketEntities(p) => match p {
                Some(p) => data.push(p),
                None => {}
            },
            _ => {}
        }
    }
    data
}

impl Parser {
    pub fn insert_props_into_df(
        &self,
        packet_ents: Vec<&PacketEntsOutput>,
        max_ticks: usize,
        int_props: &Vec<i32>,
    ) -> Vec<Vec<Vec<std::option::Option<f32>>>> {
        // Allocate the BIG outgoing DF
        let mut df = vec![vec![vec![None; max_ticks]; 3]; 3];
        // Map prop idx into its column.
        let col_mapping = create_idx_col_mapping(&int_props);
        // For every packetEnt message during game
        for packet_ent_msg in packet_ents {
            // For every entity in the message
            for single_ent in &packet_ent_msg.data {
                // For every updated value in the entity
                for prop in single_ent {
                    match prop.data {
                        PropData::F32(f) => {
                            let col_idx = col_mapping[&prop.prop_inx];
                            df[col_idx][(0 as usize)][packet_ent_msg.tick as usize] = Some(f);
                        }
                        PropData::I32(i) => {
                            let col_idx = col_mapping[&prop.prop_inx];
                            df[col_idx][(0 as usize)][packet_ent_msg.tick as usize] =
                                Some(i as f32);
                        }
                        // Todo string columns
                        _ => {}
                    }
                }
            }
        }
        df
    }

    pub fn get_raw_df(&mut self, jobs: &Vec<JobResult>, parser_maps: Arc<RwLock<ParsingMaps>>) {
        let before = Instant::now();

        let max_ticks = self.settings.playback_frames;
        let ticks: Vec<i32> = (0..max_ticks).into_iter().map(|t| t as i32).collect();

        //let svc_map = parser_maps.read().unwrap().serverclass_map.unwrap();

        let int_props = str_props_to_int_props(&self.settings.wanted_props, parser_maps.clone());

        let str_names = col_str_mapping(&int_props, parser_maps.clone());

        let packet_ents = filter_entity_messages(jobs);
        let mut df = self.insert_props_into_df(packet_ents, max_ticks, &int_props);

        use polars::frame::DataFrame;
        let mut dfs = vec![];
        for entid in 0..3 {
            let mut serieses = vec![];
            let entity_id_vec = vec![entid as i32; max_ticks];

            //let sids = eid_sid_stack(entid as u32, max_ticks as i32, &eids);
            //let s = Series::new("steamid", &sids);
            //serieses.push(s);

            let s = Series::new("tick", &ticks);
            serieses.push(s);
            for (propcol, prop_name) in str_names.iter().enumerate() {
                fill_none_with_most_recent(&mut df[propcol][entid]);
                let s = Series::new(&format!("{prop_name}"), &df[propcol][entid]);
                serieses.push(s);
            }
            let df: DataFrame = DataFrame::new(serieses).unwrap();
            dfs.push(df);
        }

        use polars::prelude::*;
        let chonker = polars::functions::diag_concat_df(&dfs).unwrap();
        println!("{:?}", chonker);
        //polars::functions::diag_concat_df([df1, df2]);
        use polars::prelude::ArrowField;
        use polars::prelude::NamedFrom;
        use polars::series::Series;
        use polars_arrow::export::arrow;
        use polars_arrow::prelude::ArrayRef;

        // let s = Series::new("viewangle_X", ent_data);
        // println!("{}", s);
    }
}

#[inline(always)]
pub fn fill_none_with_most_recent(v: &mut Vec<Option<f32>>) {
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
    let props = &serverclass_map[&40].props;
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

fn str_prop_to_int(wanted_prop: &str, serverclass_map: &HashMap<u16, ServerClass>) -> i32 {
    /*
    Maps string names to their prop index (used in packet ents)
    For example m_angEyeAngles[0] --> 20
    Mainly need to pay attention to manager props comming from different
    serverclass
    */
    let sv_cls = match serverclass_map.get(&40) {
        Some(props) => props,
        None => panic!("no svc"),
    };

    match sv_cls.props.iter().position(|x| x.name == wanted_prop) {
        Some(idx) => idx as i32,
        None => panic!("Could not find prop idx for {}", wanted_prop),
    }
}
