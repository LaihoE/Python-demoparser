use std::time::Instant;

use crate::parsing::cache::cache_reader::ReadCache;
use crate::parsing::create_output::create_output::ExtraEventRequest;
use crate::parsing::demo_parsing::entities::PacketEntsOutput;
use crate::parsing::demo_parsing::KeyData;
use crate::parsing::demo_parsing::*;
use crate::parsing::parser::*;
use crate::parsing::players::Players;
use crate::parsing::utils::TYPEHM;
pub use crate::parsing::variants::*;
use derive_more::TryInto;
use itertools::Itertools;
use polars::df;
use polars::export::regex::internal::Inst;
use polars::prelude::{DataFrame, Int64Type, NamedFrom, NamedFromOwned};
use polars::series::Series;

impl Parser {
    fn uid_to_sid_vec(v: &Vec<&NameDataPair>, players: &Players) -> Option<Series> {
        let mut uids = vec![];
        // Player death events have this
        let mut attackers = vec![];
        for name_data_pair in v {
            match name_data_pair.name.as_str() {
                "userid" => match name_data_pair.data {
                    KeyData::Short(uid) => uids.push(uid),
                    _ => {}
                },
                "attacker" => match name_data_pair.data {
                    KeyData::Short(uid) => attackers.push(uid),
                    _ => {}
                },
                _ => {}
            }
        }
        if uids.len() > 0 {
            let steamids: Vec<u64> = uids
                .iter()
                .map(|uid| players.uid_to_steamid(*uid as u32).unwrap_or(0))
                .collect();
            return Some(Series::from_vec("steamid", steamids));
        }
        if attackers.len() > 0 {
            let steamids: Vec<u64> = attackers
                .iter()
                .map(|uid| players.uid_to_steamid(*uid as u32).unwrap_or(0))
                .collect();
            return Some(Series::from_vec("attacker", steamids));
        }
        None
    }

    fn filter_to_vec<Wanted>(v: impl IntoIterator<Item = impl TryInto<Wanted>>) -> Vec<Wanted> {
        v.into_iter().filter_map(|x| x.try_into().ok()).collect()
    }

    fn series_from_pairs(pairs: Vec<&NameDataPair>, name: &String, players: &Players) -> Series {
        if name == "userid" || name == "attacker" {
            match Self::uid_to_sid_vec(&pairs, players) {
                Some(s) => {
                    return s;
                }
                _ => {}
            };
        }
        let only_data: Vec<KeyData> = pairs.iter().map(|x| x.data.clone()).collect();
        let s = match pairs[0].data_type {
            1 => Series::new(name, &Parser::filter_to_vec::<String>(only_data)),
            2 => Series::new(name, &Parser::filter_to_vec::<f32>(only_data)),
            3 => Series::new(name, &Parser::filter_to_vec::<i64>(only_data)),
            4 => Series::new(name, &Parser::filter_to_vec::<i64>(only_data)),
            5 => Series::new(name, &Parser::filter_to_vec::<i64>(only_data)),
            6 => Series::new(name, &Parser::filter_to_vec::<bool>(only_data)),
            7 => Series::new(name, &Parser::filter_to_vec::<u64>(only_data)),
            _ => panic!("Keydata got unknown type: {}", pairs[0].data_type),
        };
        s
    }

    fn series_from_events(&self, events: Vec<GameEvent>, players: &Players) -> Vec<Series> {
        // Example [Hashmap<"distance": 21.0>, Hashmap<"distance": 24.0>, Hashmap<"name": "Steve">]
        // ->
        // Hashmap<"distance": [21.0, 24.0], "name": ["Steve"]>,
        // -> Series::new("distance", [21.0, 24.0]) <-- needs to be mapped as "f32" not as enum(KeyData)
        let pairs: Vec<NameDataPair> = events.iter().map(|x| x.fields.clone()).flatten().collect();
        let per_key_name = pairs.iter().into_group_map_by(|x| &x.name);
        let mut series = vec![];
        for (name, vals) in per_key_name {
            series.push(Parser::series_from_pairs(vals, name, players));
        }
        series
    }
    fn convert_to_requests(
        &self,
        ticks: Vec<i64>,
        userids: Vec<u64>,
        attackers: Vec<u64>,
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
        let mut userids: Vec<u64> = vec![];
        let mut attackers: Vec<u64> = vec![];

        for s in series {
            match s.name() {
                "tick" => ticks.extend(s.i64().unwrap().into_no_null_iter()),
                "steamid" => userids.extend(s.u64().unwrap().into_no_null_iter()),
                // "attacker" => attackers.extend(s.u64().unwrap().into_no_null_iter()),
                _ => {}
            }
        }
        self.convert_to_requests(ticks, userids, attackers, &wanted_props)
    }

    pub fn get_game_events(
        &mut self,
        results: &Vec<JobResult>,
        players: &Players,
        cache: &mut ReadCache,
    ) -> Vec<Series> {
        let before = Instant::now();
        let event_id = cache
            .event_name_to_id(&self.settings.event_name.to_string(), &self.maps.event_map)
            .unwrap();

        let mut v = vec![];
        for x in results {
            if let JobResult::GameEvents(ge) = x {
                if ge.id == event_id {
                    v.push(ge.clone());
                }
            }
        }
        let eid_cls_map = cache.eid_cls_map.clone();
        let mut series = self.series_from_events(v, players);
        let requests = self.fill_wanted_extra_props(&series, &self.settings.wanted_props.clone());
        let extra_bytes = cache.find_request_bytes(&requests, &self.maps.serverclass_map, players);
        if extra_bytes.len() > 0 {
            self.parse_bytes(extra_bytes);
        }

        let (results, _) = self.parse_blueprints(false, Some(eid_cls_map));
        let s = self.find_requested_vals(requests, &results, &players);

        series.extend(s);
        // Sort game events so that columns are always in the same order
        series.sort_by_key(|s| s.name().to_string());
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
        series
    }
}
