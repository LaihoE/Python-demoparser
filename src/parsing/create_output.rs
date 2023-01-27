use super::entities::PacketEntsOutput;
use super::game_events::GameEvent;
use super::stringtables::UserInfo;
use crate::parsing::columnmapper::EntColMapper;
use crate::parsing::game_events;
use crate::parsing::game_events::KeyData;
use crate::parsing::game_events::NameDataPair;
use crate::parsing::parser::JobResult;
use crate::parsing::players::Players;
pub use crate::parsing::variants::*;
use crate::Parser;
use ahash::HashMap;
use ndarray::s;
use ndarray::ArrayBase;
use ndarray::Dim;
use ndarray::OwnedRepr;
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
) -> (Vec<&PacketEntsOutput>, Vec<GameEvent>, Vec<&UserInfo>) {
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
                if ge[0].id == 24 {
                    game_events.extend(ge.clone());
                }
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
        game_events: &mut Vec<GameEvent>,
        ecm: EntColMapper,
        df: &mut ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>>,
        players: &Players,
    ) {
        for ev in game_events {
            let mut x = 0.0;
            for field in &ev.fields {
                if field.name == "attacker" {
                    //println!("GAME EVENT {}", ev.tick);
                    match &field.data {
                        Some(f) => match f {
                            game_events::KeyData::Short(ui) => {
                                // println!("USERID: {}", ui);
                                let entid = players.uid_to_entid(*ui as u32, ev.byte);
                                // let ent_id = ecm.entid_from_uid(&field.data);

                                let prop_col = ecm.get_prop_col(&21);
                                let player_col = ecm.get_player_col(entid.unwrap() as u32, ev.tick);
                                // let tick = ecm.get_tick(packet_ent_msg.tick);
                                x = df[[player_col, prop_col, ev.tick as usize]];

                                //println!("F {:?}", x);
                                //println!("{:?}", entid);
                            }
                            _ => {}
                        },
                        None => {}
                    }
                    //let ent_id = ecm.entid_from_uid(&field.data);
                    /*
                    let prop_col = col_mapping[&prop.prop_inx];
                    let player_col = ecm.get_col(ent_id, ev.tick);
                    df[[player_col, prop_col, packet_ent_msg.tick as usize]] = i as f32;
                    */
                }
                //println!("{:?}", f);
            }
            ev.fields.push(NameDataPair {
                name: "DATA".to_string(),
                data: Some(game_events::KeyData::Float(x)),
            });
        }

        // let prop_col = col_mapping[&prop.prop_inx];
        // let player_col = ecm.get_col(prop.ent_id as u32, packet_ent_msg.tick);
        // df[[player_col, prop_col, packet_ent_msg.tick as usize]] = i as f32;
        // let tick = ev.tick;
        // ecm.g
        // println!("{:?}", df[[]])
    }

    pub fn bins(
        &self,
        game_events: &mut Vec<GameEvent>,
        packet_ents: &Vec<&PacketEntsOutput>,
        players: &Players,
    ) {
        /*

        {'weapon_fauxitemid': '17293822569102704656', 'noscope': False, 'player_steamid': 76561198048924300,
        'event_name': 'player_death', 'revenge': 0, 'penetrated': 0, 'weapon_itemid': '0', 'noreplay': False,
        'player_name': 'Bo-Krister', 'attacker_m_angEyeAngles[0]': 1.42822265625, 'attackerblind': False,
        'dominated': 0, 'tick': 5455, 'event_id': 24, 'round': 0, 'assister': 0, 'attacker_name': 'rEVILS_tex',
        'assistedflash': False, 'headshot': False, 'wipe': 0, 'attacker_steamid': 76561197997241560, 'thrusmoke': False,
        'weapon': 'm4a1', 'weapon_originalowner_xuid': '76561197997241560', 'player_m_angEyeAngles[0]': 17.2650146484375}

        */

        // println!("{:?}", packet_ents[0]);
        for x in packet_ents {
            //println!("{:?}", x.tick);
        }
        let mut v = vec![];
        let mut packet_cnt = 0;
        'outer: for (idx, event) in game_events.iter().rev().enumerate() {
            if event.id == 24 {
                //panic!("");
                match event.get_attacker_uid() {
                    Some(uid) => {
                        let attacker = players
                            .uid_to_entid(uid as u32, event.tick as usize)
                            .unwrap_or(69);
                        let attacker_name =
                            players.uid_to_name(uid as u32).unwrap_or(69.to_string());

                        let vic_name = players
                            .uid_to_name(event.get_player_uid().unwrap() as u32)
                            .unwrap_or(69.to_string());

                        for (packet_idx, pe) in packet_ents[..packet_ents.len() - packet_cnt]
                            .iter()
                            .rev()
                            .enumerate()
                        {
                            for x in &pe.data {
                                if x.ent_id == attacker as i32
                                    && x.prop_inx == 21
                                    && pe.tick < event.tick
                                {
                                    /*
                                                                        println!(
                                                                            "{} {:?} {} {} {:?}",
                                                                            attacker_name, vic_name, pe.tick, event.tick, x
                                                                        );
                                    */
                                    v.push((
                                        idx,
                                        NameDataPair {
                                            name: "DATA".to_string(),
                                            data: Some(KeyData::from_pdata(&x.data)),
                                        },
                                    ));
                                    v.push((
                                        idx,
                                        NameDataPair {
                                            name: "attacker_name".to_string(),
                                            data: Some(KeyData::Str(attacker_name)),
                                        },
                                    ));
                                    v.push((
                                        idx,
                                        NameDataPair {
                                            name: "victim_name".to_string(),
                                            data: Some(KeyData::Str(vic_name)),
                                        },
                                    ));
                                    v.push((
                                        idx,
                                        NameDataPair {
                                            name: "tick".to_string(),
                                            data: Some(KeyData::Long(event.tick)),
                                        },
                                    ));

                                    packet_cnt += packet_idx;
                                    continue 'outer;
                                }
                            }
                        }
                    }
                    None => {
                        println!("NO FOUND");
                    }
                }
            }
        }
        for (idx, val) in v {
            //println!("{} {:?}", idx, val);
            let event = &mut game_events[idx];
            event.fields.push(val);
        }
    }

    pub fn get_raw_df(
        &mut self,
        jobs: &Vec<JobResult>,
        //parser_maps: Arc<RwLock<ParsingMaps>>,
        df: &mut ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>>,
        max_ticks: usize,
        players: &Players,
    ) -> Vec<GameEvent> {
        // Group jobs by type
        let (packet_ents, mut game_events, stringtables) = filter_jobresults(jobs);
        //println!("{:?}", players.players);
        game_events.sort_by_key(|x| x.tick);
        let mut gs: Vec<GameEvent> = game_events
            .iter()
            .filter(|g| g.id == 24)
            .map(|x| x.clone())
            .collect();
        // println!("{:?}", gs);
        // game_events.dedup_by_key(|x| x.tick);
        // println!("{:?}", game_events);

        for player in &players.players {
            // println!("{} {}", player.name, player.user_id);
        }
        self.bins(&mut gs, &packet_ents, players);
        return gs;
        /*
        panic!("done");

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
            // println!("{:?} {}", props, max_ticks);
            series_players.push(props);
        }

        // println!("SERIES: {:2?}", before.elapsed());

        let steamids = Series::new("steamids", steamids_col);
        let ticks = Series::new("ticks", ticks_col);

        series_players.push(steamids);
        series_players.push(ticks);

        self.create_game_events(&mut game_events, ecm, df, players);

        //println!("{:?}", game_events);

        series_players
        */
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
