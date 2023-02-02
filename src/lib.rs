mod parsing;
use crate::parsing::game_events::GameEvent;
use crate::parsing::game_events::NameDataPair;
use ahash::RandomState;
use arrow::ffi;
use flate2::read::GzDecoder;
use fxhash::FxHashMap;
use parsing::entities::Entity;
use parsing::game_events::KeyData;
use parsing::parser::Parser;
use parsing::stringtables::UserInfo;
//use parsing::tick_cache::gather_props_backwards;
use parsing::utils::Header;
use parsing::variants::PropData;
use phf::phf_map;
use polars::prelude::ArrowField;
use polars::prelude::NamedFrom;
use polars::series::Series;
use polars_arrow::export::arrow;
use polars_arrow::prelude::ArrayRef;
use protobuf::Message;
use pyo3::exceptions::PyFileNotFoundError;
use pyo3::exceptions::PyKeyError;
use pyo3::ffi::Py_uintptr_t;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyDict;
use pyo3::{PyAny, PyObject, PyResult};
use pyo3::{PyErr, Python};
use std::collections::HashMap;
use std::io::prelude::*;
use std::path::Path;
use std::time::Instant;
use std::vec;

/// https://github.com/pola-rs/polars/blob/master/examples/python_rust_compiled_function/src/ffi.rs
pub(crate) fn to_py_array(py: Python, pyarrow: &PyModule, array: ArrayRef) -> PyResult<PyObject> {
    let schema = Box::new(ffi::export_field_to_c(&ArrowField::new(
        "",
        array.data_type().clone(),
        true,
    )));
    let array = Box::new(ffi::export_array_to_c(array));
    let schema_ptr: *const ffi::ArrowSchema = &*schema;
    let array_ptr: *const ffi::ArrowArray = &*array;
    let array = pyarrow.getattr("Array")?.call_method1(
        "_import_from_c",
        (array_ptr as Py_uintptr_t, schema_ptr as Py_uintptr_t),
    )?;

    Ok(array.to_object(py))
}
/// https://github.com/pola-rs/polars/blob/master/examples/python_rust_compiled_function/src/ffi.rs
pub fn rust_series_to_py_series(series: &Series) -> PyResult<PyObject> {
    let series = series.rechunk();
    let array = series.to_arrow(0);
    let gil = Python::acquire_gil();
    let py = gil.python();
    let pyarrow = py.import("pyarrow")?;
    let pyarrow_array = to_py_array(py, pyarrow, array)?;
    let polars = py.import("polars")?;
    let out = polars.call_method1("from_arrow", (pyarrow_array,))?;
    Ok(out.to_object(py))
}

pub fn decompress_gz(bytes: Vec<u8>) -> Vec<u8> {
    let mut gz = GzDecoder::new(&bytes[..]);
    let mut out: Vec<u8> = vec![];
    gz.read_to_end(&mut out).unwrap();
    out
}

pub fn read_file(demo_path: String) -> Result<Vec<u8>, std::io::Error> {
    let result = std::fs::read(&demo_path);
    match result {
        // FILE COULD NOT BE READ
        Err(e) => {
            println!("{}", e);
            Err(e)
        } //panic!("The demo could not be found. Error: {}", e),
        Ok(bytes) => {
            let extension = Path::new(&demo_path).extension().unwrap();
            match extension.to_str().unwrap() {
                "gz" => Ok(decompress_gz(bytes)),
                _ => Ok(bytes),
            }
        }
    }
}

pub fn parse_kwargs_ticks(kwargs: Option<&PyDict>) -> (Vec<u64>, Vec<i32>) {
    match kwargs {
        Some(k) => {
            let mut players: Vec<u64> = vec![];
            let mut ticks: Vec<i32> = vec![];
            match k.get_item("players") {
                Some(p) => {
                    players = p.extract().unwrap();
                }
                None => {}
            }
            match k.get_item("ticks") {
                Some(t) => {
                    ticks = t.extract().unwrap();
                }
                None => {}
            }
            (players, ticks)
        }
        None => (vec![], vec![]),
    }
}

pub fn parse_kwargs_event(kwargs: Option<&PyDict>) -> (bool, Vec<String>) {
    match kwargs {
        Some(k) => {
            let mut rounds = false;
            let mut props: Vec<String> = vec![];
            match k.get_item("rounds") {
                Some(p) => {
                    rounds = p.extract().unwrap();
                }
                None => {}
            }
            match k.get_item("props") {
                Some(t) => {
                    props = t.extract().unwrap();
                }
                None => {}
            }
            (rounds, props)
        }
        None => (false, vec![]),
    }
}

#[pyclass]
struct DemoParser {
    path: String,
}

#[pymethods]
impl DemoParser {
    #[new]
    pub fn py_new(demo_path: String) -> PyResult<Self> {
        Ok(DemoParser { path: demo_path })
    }
    /*
    #[args(py_kwargs = "**")]
    pub fn parse_events(
        &self,
        py: Python<'_>,
        event_name: String,
        py_kwargs: Option<&PyDict>,
    ) -> PyResult<Py<PyAny>> {
        let (rounds, wanted_props) = parse_kwargs_event(py_kwargs);
        let real_props = rm_user_friendly_names(&wanted_props);
        let unk_props = check_validity_props(&real_props);
        if !unk_props.is_empty() {
            return Err(PyKeyError::new_err(format!(
                "Unknown fields: {:?}",
                unk_props
            )));
        }
        let parse_props = !wanted_props.is_empty() || rounds;
        let parser = Parser::new(
            self.path.clone(),
            parse_props,
            vec![],
            vec![],
            real_props,
            event_name,
            false,
            false,
            false,
            9999999,
            wanted_props,
        );
        match parser {
            Err(e) => Err(PyFileNotFoundError::new_err(format!(
                "Couldnt read demo file. Error: {}",
                e
            ))),
            Ok(mut parser) => {
                let _: Header = parser.parse_demo_header();

                let game_events = parser.start_parsing();
                let mut game_evs: Vec<FxHashMap<String, PyObject>> = Vec::new();

                // Create Hashmap with <string, pyobject> to be able to convert to python dict
                for ge in game_events {
                    if ge.id != 24 {
                        continue;
                    }
                    let mut hm: FxHashMap<String, PyObject> = FxHashMap::default();
                    let tuples = ge.to_py_tuples(py);
                    for (k, v) in tuples {
                        hm.insert(k, v);
                    }
                    game_evs.push(hm);
                }
                let dict = pyo3::Python::with_gil(|py| game_evs.to_object(py));
                Ok(dict)
            }
        }
    }
    */

    #[args(py_kwargs = "**")]
    pub fn parse_ticks(
        &self,
        py: Python,
        mut wanted_props: Vec<String>,
        py_kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        //println!("WANTED PROPS {:?}", wanted_props);
        let mut real_props = rm_user_friendly_names(&wanted_props);
        //println!("REAL PROPS {:?}", real_props);

        let unk_props = check_validity_props(&real_props);
        if !unk_props.is_empty() {
            return Err(PyKeyError::new_err(format!(
                "Unknown fields: {:?}",
                unk_props
            )));
        }
        let (wanted_players, wanted_ticks) = parse_kwargs_ticks(py_kwargs);
        let wanted_ticks_len = wanted_ticks.len();
        let biggest_wanted_tick = if wanted_ticks_len > 0 {
            wanted_ticks.iter().max().unwrap()
        } else {
            &99999999
        };

        let parser = Parser::new(
            self.path.clone(),
            true,
            wanted_ticks.clone(),
            wanted_players,
            real_props.clone(),
            "".to_string(),
            false,
            false,
            true,
            *biggest_wanted_tick,
            wanted_props.clone(),
        );

        match parser {
            Err(e) => Err(PyFileNotFoundError::new_err(format!(
                "Couldnt read demo file. Error: {}",
                e
            ))),
            Ok(mut parser) => {
                let h: Header = parser.parse_demo_header();

                parser.settings.playback_frames = if wanted_ticks_len == 0 {
                    h.playback_frames as usize
                } else {
                    wanted_ticks_len
                };
                parser.settings.playback_frames = (h.playback_ticks + 100) as usize;
                let mut ss = vec![];
                let series_vec = parser.start_parsing();
                for s in series_vec {
                    let py_series = rust_series_to_py_series(&s).unwrap();
                    ss.push(py_series);
                }
                wanted_props.push("steamid".to_string());
                wanted_props.push("tick".to_string());
                let polars = py.import("polars").unwrap();
                // let all_series_py = ss.to_object(py);
                let df = polars.call_method1("DataFrame", (ss,)).unwrap();
                // df.setattr("columns", wanted_props.to_object(py)).unwrap();
                let pandas_df = df.call_method0("to_pandas").unwrap();
                Ok(pandas_df.to_object(py))
            }
        }
    }

    /*
    pub fn parse_players(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let parser = Parser::new(
            self.path.clone(),
            true,
            vec![],
            vec![],
            vec!["m_iTeamNum".to_string()],
            "".to_string(),
            true,
            false,
            true,
            9999999,
            vec![],
        );
        match parser {
            Err(e) => Err(PyFileNotFoundError::new_err(format!(
                "Couldnt read demo file. Error: {}",
                e
            ))),
            Ok(mut parser) => {
                let _: Header = parser.parse_demo_header();
                let _ = parser.start_parsing(&vec![]);
                let players = parser.maps.players;
                let mut py_players = vec![];
                let ent_manager = &parser.state.entities[70 as usize].1;
                for (_, player) in players {
                    let team = get_player_team(&parser.state.entities[player.entity_id as usize].1);
                    let mut hm = player.to_hashmap(py);
                    let team = match team {
                        2 => "T",
                        3 => "CT",
                        _ => "Missing",
                    };

                    let rank_id =
                        get_manager_i32_prop(ent_manager, &player, "m_iCompetitiveRanking");
                    let rank_name = rank_id_to_name(rank_id);

                    let crosshair_code =
                        get_manager_str_prop(ent_manager, &player, "m_szCrosshairCodes");

                    let comp_wins =
                        get_manager_i32_prop(ent_manager, &player, "m_iCompetitiveWins");

                    hm.insert("starting_side".to_string(), team.to_string().to_object(py));
                    hm.insert(
                        "crosshair_code".to_string(),
                        crosshair_code.to_string().to_object(py),
                    );
                    hm.insert("rank_name".to_string(), rank_name.to_string().to_object(py));
                    hm.insert("rank_id".to_string(), rank_id.to_object(py));
                    hm.insert("comp_wins".to_string(), comp_wins.to_object(py));

                    let dict = pyo3::Python::with_gil(|py| hm.to_object(py));
                    if player.xuid > 76500000000000000 && player.xuid < 76600000000000000 {
                        py_players.push(dict);
                    }
                }
                let dict = pyo3::Python::with_gil(|py| py_players.to_object(py));
                Ok(dict)
            }
        }
    }
    */
    pub fn parse_header(&self) -> PyResult<Py<PyAny>> {
        let parser = Parser::new(
            self.path.clone(),
            false,
            vec![],
            vec![],
            vec![],
            "".to_string(),
            true,
            false,
            true,
            9999999,
            vec![],
        );
        match parser {
            Err(e) => Err(PyFileNotFoundError::new_err(format!(
                "Couldnt read demo file. Error: {}",
                e
            ))),
            Ok(mut parser) => {
                let h: Header = parser.parse_demo_header();
                let dict = h.to_py_hashmap();
                Ok(dict)
            }
        }
    }
}

pub fn get_player_team(ent: &Entity) -> i32 {
    match ent.props.get("m_iTeamNum") {
        Some(p) => match p.data {
            PropData::I32(x) => x,
            _ => -1,
        },
        None => -1,
    }
}

pub fn get_manager_i32_prop(manager: &Entity, player: &UserInfo, prop_name: &str) -> i32 {
    let key = if player.entity_id < 10 {
        prop_name.to_string() + "00" + &player.entity_id.to_string()
    } else if player.entity_id < 100 {
        prop_name.to_string() + "0" + &player.entity_id.to_string()
    } else {
        panic!("Entity id > 100 ????: id:{}", player.entity_id);
    };
    match manager.props.get(&key) {
        Some(p) => match p.data {
            PropData::I32(x) => x,
            _ => -1,
        },
        None => -1,
    }
}

pub fn get_manager_str_prop(manager: &Entity, player: &UserInfo, prop_name: &str) -> String {
    let key = if player.entity_id < 10 {
        prop_name.to_string() + "00" + &player.entity_id.to_string()
    } else if player.entity_id < 100 {
        prop_name.to_string() + "0" + &player.entity_id.to_string()
    } else {
        panic!("Entity id > 100 ????: id:{}", player.entity_id);
    };
    match manager.props.get(&key) {
        Some(p) => match &p.data {
            PropData::String(x) => x.to_string(),
            _ => "".to_string(),
        },
        None => "".to_string(),
    }
}

pub fn check_validity_props(names: &Vec<String>) -> Vec<String> {
    let mut unkown_props = vec![];
    for name in names {
        match TYPEHM.contains_key(name) {
            true => {}
            false => unkown_props.push(name.to_string()),
        }
    }
    unkown_props
}
pub fn rank_id_to_name(id: i32) -> String {
    match id {
        1 => "Silver 1".to_string(),
        2 => "Silver 2".to_string(),
        3 => "Silver 3".to_string(),
        4 => "Silver 4".to_string(),
        5 => "Silver elite".to_string(),
        6 => "Silver elite master".to_string(),
        7 => "Nova 1".to_string(),
        8 => "Nova 2".to_string(),
        9 => "Nova 3".to_string(),
        10 => "Nova 4".to_string(),
        11 => "MG1".to_string(),
        12 => "MG2".to_string(),
        13 => "MGE".to_string(),
        14 => "DMG".to_string(),
        15 => "LE".to_string(),
        16 => "LEM".to_string(),
        17 => "Supreme".to_string(),
        18 => "Global elite".to_string(),
        _ => "Unranked".to_string(),
    }
}
pub fn rm_user_friendly_names(names: &Vec<String>) -> Vec<String> {
    let mut unfriendly_names = vec![];
    for name in names {
        match &name[..] {
            "X" => unfriendly_names.push("m_vecOrigin_X".to_string()),
            "Y" => unfriendly_names.push("m_vecOrigin_Y".to_string()),
            "Z" => unfriendly_names.push("m_vecOrigin[2]".to_string()),
            "ammo" => unfriendly_names.push("m_iClip1".to_string()),
            "velocity_X" => unfriendly_names.push("m_vecVelocity[0]".to_string()),
            "velocity_Y" => unfriendly_names.push("m_vecVelocity[1]".to_string()),
            "velocity_Z" => unfriendly_names.push("m_vecVelocity[2]".to_string()),
            "viewangle_pitch" => unfriendly_names.push("m_angEyeAngles[0]".to_string()),
            "viewangle_yaw" => unfriendly_names.push("m_angEyeAngles[1]".to_string()),
            "ducked" => unfriendly_names.push("m_bDucked".to_string()),
            "in_buy_zone" => unfriendly_names.push("m_bInBuyZone".to_string()),
            "scoped" => unfriendly_names.push("m_bIsScoped".to_string()),
            "health" => unfriendly_names.push("m_iHealth".to_string()),
            "flash_duration" => unfriendly_names.push("m_flFlashDuration".to_string()),
            "aimpunch_X" => unfriendly_names.push("m_aimPunchAngle_X".to_string()),
            "aimpunch_Y" => unfriendly_names.push("m_aimPunchAngle_Y".to_string()),
            "aimpunch_Z" => unfriendly_names.push("m_aimPunchAngle_Z".to_string()),
            "aimpunch_vel_X" => unfriendly_names.push("m_aimPunchAngleVel_X".to_string()),
            "aimpunch_vel_Y" => unfriendly_names.push("m_aimPunchAngleVel_Y".to_string()),
            "aimpunch_vel_Z" => unfriendly_names.push("m_aimPunchAngleVel_Z".to_string()),
            "balance" => unfriendly_names.push("m_iAccount".to_string()),
            "ping" => unfriendly_names.push("m_iPing".to_string()),
            "score" => unfriendly_names.push("m_iScore".to_string()),
            "deaths" => unfriendly_names.push("m_iDeaths".to_string()),
            "kills" => unfriendly_names.push("m_iKills".to_string()),
            "assists" => unfriendly_names.push("m_iAssists".to_string()),
            "mvps" => unfriendly_names.push("m_iMVPs".to_string()),
            "armor" => unfriendly_names.push("m_iArmor".to_string()),
            "silencer_on" => unfriendly_names.push("m_bSilencerOn".to_string()),
            "place_name" => unfriendly_names.push("m_szLastPlaceName".to_string()),
            "total_enemies_flashed" => {
                unfriendly_names.push("m_iMatchStats_EnemiesFlashed_Total".to_string())
            }
            "total_util_damage" => {
                unfriendly_names.push("m_iMatchStats_UtilityDamage_Total".to_string())
            }
            "total_cash_earned" => {
                unfriendly_names.push("m_iMatchStats_CashEarned_Total".to_string())
            }
            "total_objective_total" => {
                unfriendly_names.push("m_iMatchStats_Objective_Total".to_string())
            }
            "total_headshots" => {
                unfriendly_names.push("m_iMatchStats_HeadShotKills_Total".to_string())
            }
            "total_assists" => unfriendly_names.push("m_iMatchStats_Assists_Total".to_string()),
            "total_deaths" => unfriendly_names.push("m_iMatchStats_Deaths_Total".to_string()),
            "total_live_time" => unfriendly_names.push("m_iMatchStats_LiveTime_Total".to_string()),
            "total_kill_reward" => {
                unfriendly_names.push("m_iMatchStats_KillReward_Total".to_string())
            }
            "total_equipment_value" => {
                unfriendly_names.push("m_iMatchStats_EquipmentValue_Total".to_string())
            }
            "total_damage" => unfriendly_names.push("m_iMatchStats_Damage_Total".to_string()),
            "3ks" => unfriendly_names.push("m_iMatchStats_3k_Total".to_string()),
            "4ks" => unfriendly_names.push("m_iMatchStats_4k_Total".to_string()),
            "5ks" => unfriendly_names.push("m_iMatchStats_5k_Total".to_string()),
            "total_kills" => unfriendly_names.push("m_iMatchStats_Kills_Total".to_string()),
            "is_auto_muted" => unfriendly_names.push("m_bHasCommunicationAbuseMute".to_string()),
            "friendly_honors" => {
                unfriendly_names.push("m_nPersonaDataPublicCommendsFriendly".to_string())
            }
            "teacher_honors" => {
                unfriendly_names.push("m_nPersonaDataPublicCommendsTeacher".to_string())
            }
            "leader_honors" => {
                unfriendly_names.push("m_nPersonaDataPublicCommendsLeader".to_string())
            }
            "public_level" => unfriendly_names.push("m_nPersonaDataPublicLevel".to_string()),
            "active_coin_rank" => unfriendly_names.push("m_nActiveCoinRank".to_string()),
            "cash_spent_this_round" => unfriendly_names.push("m_iCashSpentThisRound".to_string()),
            "total_cash_spent" => unfriendly_names.push("m_iTotalCashSpent".to_string()),
            "controlled_by_player" => unfriendly_names.push("m_iControlledByPlayer".to_string()),
            "controlled_player" => unfriendly_names.push("m_iControlledPlayer".to_string()),
            "controlling_bot" => unfriendly_names.push("m_bControllingBot".to_string()),
            "lifetime_start" => unfriendly_names.push("m_iLifetimeStart".to_string()),
            "lifetime_end" => unfriendly_names.push("m_iLifetimeEnd".to_string()),
            "connected" => unfriendly_names.push("m_bConnected".to_string()),
            "holding_look_weapon" => unfriendly_names.push("m_bIsHoldingLookAtWeapon".to_string()),
            "looking_at_weapon" => unfriendly_names.push("m_bIsLookingAtWeapon".to_string()),
            "headshots_this_round" => {
                unfriendly_names.push("m_iNumRoundKillsHeadshots".to_string())
            }
            "concurrent_killed" => unfriendly_names.push("m_nLastConcurrentKilled".to_string()),
            "freeze_end_eq_val" => {
                unfriendly_names.push("m_unFreezetimeEndEquipmentValue".to_string())
            }
            "round_start_eq_val" => {
                unfriendly_names.push("m_unRoundStartEquipmentValue".to_string())
            }
            "equipment_value" => unfriendly_names.push("m_unCurrentEquipmentValue".to_string()),
            "flash_alpha" => unfriendly_names.push("m_flFlashMaxAlpha".to_string()),
            "has_helmet" => unfriendly_names.push("m_bHasHelmet".to_string()),
            "has_heavy_armor" => unfriendly_names.push("m_bHasHeavyArmor".to_string()),
            "detected_enemy_sensor" => {
                unfriendly_names.push("m_flDetectedByEnemySensorTime".to_string())
            }
            "is_rescuing" => unfriendly_names.push("m_bIsRescuing".to_string()),
            "molotov_dmg_time" => unfriendly_names.push("m_fMolotovDamageTime".to_string()),
            "molotov_use_time" => unfriendly_names.push("m_fMolotovUseTime".to_string()),
            "moved_since_spawn" => unfriendly_names.push("m_bHasMovedSinceSpawn".to_string()),
            "resume_zoom" => unfriendly_names.push("m_bResumeZoom".to_string()),
            "is_walking" => unfriendly_names.push("m_bIsWalking".to_string()),
            "is_defusing" => unfriendly_names.push("m_bIsDefusing".to_string()),
            "has_defuser" => unfriendly_names.push("m_bHasDefuser".to_string()),
            "in_bomb_zone" => unfriendly_names.push("m_bInBombZone".to_string()),
            "granade_counter" => unfriendly_names.push("m_iThrowGrenadeCounter".to_string()),
            "last_made_noise_time" => unfriendly_names.push("m_flLastMadeNoiseTime".to_string()),
            "spotted" => unfriendly_names.push("m_bSpotted".to_string()),
            "elasticity" => unfriendly_names.push("m_flElasticity".to_string()),
            "team_num" => unfriendly_names.push("m_iTeamNum".to_string()),
            "velocity_modifier" => unfriendly_names.push("m_flVelocityModifier".to_string()),
            "next_think_tick" => unfriendly_names.push("m_nNextThinkTick".to_string()),
            "friction" => unfriendly_names.push("m_flFriction".to_string()),
            "on_target" => unfriendly_names.push("m_fOnTarget".to_string()),
            "vec_view_offset0" => unfriendly_names.push("m_vecViewOffset[0]".to_string()),
            "vec_view_offset1" => unfriendly_names.push("m_vecViewOffset[1]".to_string()),
            "is_wearing_suit" => unfriendly_names.push("m_bWearingSuit".to_string()),
            "jump_time_msecs" => unfriendly_names.push("m_nJumpTimeMsecs".to_string()),
            "duck_time_msecs" => unfriendly_names.push("m_nDuckJumpTimeMsecs".to_string()),
            "in_duck_jump" => unfriendly_names.push("m_bInDuckJump".to_string()),
            "last_duck_time" => unfriendly_names.push("m_flLastDuckTime".to_string()),
            "is_ducking" => unfriendly_names.push("m_bDucking".to_string()),

            _ => unfriendly_names.push(name.to_string()),
        }
    }
    unfriendly_names
}
pub static TYPEHM: phf::Map<&'static str, i32> = phf_map! {
    "DT_CSLocalPlayerExclusive.m_vecOrigin[2]" => 1,
    "DT_LocalPlayerExclusive.m_vecVelocity[0]" => 1,
    "DT_LocalPlayerExclusive.m_vecVelocity[1]" => 1,
    "DT_LocalPlayerExclusive.m_vecVelocity[2]" => 1,
    "DT_CSNonLocalPlayerExclusive.m_vecOrigin" => 3,
    "DT_CSNonLocalPlayerExclusive.m_vecOrigin[2]" => 1,
    "DT_Local.m_flFallVelocity" => 1,
    "DT_Local.m_viewPunchAngle" => 2,
    "DT_Local.m_aimPunchAngle" => 2,
    "DT_Local.m_aimPunchAngleVel" => 2,
    "DT_LocalPlayerExclusive.m_vecViewOffset[2]" => 1,
    "DT_BasePlayer.m_fFlags" => 0,
    "DT_BasePlayer.m_iFOV" => 0,
    "DT_BasePlayer.m_flFOVTime" => 1,
    "DT_BasePlayer.m_flDuckAmount" => 1,
    "DT_BasePlayer.m_flDuckSpeed" => 1,
    "DT_CSPlayer.m_angEyeAngles[0]" => 1,
    "DT_CSPlayer.m_angEyeAngles[1]" => 1,
    "DT_CSPlayer.m_iMoveState" => 0,
    "DT_CSPlayer.m_flGroundAccelLinearFracLastTime" => 1,
    "DT_Animationlayer.m_nOrder" => 0,
    "DT_Animationlayer.m_nSequence" => 0,
    "DT_Animationlayer.m_flPlaybackRate" => 1,
    "DT_Animationlayer.m_flWeight" => 1,
    "DT_BCCLocalPlayerExclusive.m_flNextAttack" => 1,
    "m_hMyWeapons.000" => 0,
    "m_hMyWeapons.001" => 0,
    "m_hMyWeapons.002" => 0,
    "m_hMyWeapons.003" => 0,
    "m_hMyWeapons.004" => 0,
    "m_hMyWeapons.005" => 0,
    "m_hMyWeapons.006" => 0,
    "m_hMyWeapons.007" => 0,
    "m_hMyWearables.000" => 0,
    "DT_PlayerState.deadflag" => 0,
    "m_iAmmo.014" => 0,
    "m_iAmmo.015" => 0,
    "m_iAmmo.016" => 0,
    "m_iAmmo.017" => 0,
    "m_iAmmo.018" => 0,
    "m_chAreaBits.000" => 0,
    "m_chAreaBits.001" => 0,
    "m_chAreaBits.002" => 0,
    "m_chAreaPortalBits.000" => 0,
    "m_chAreaPortalBits.001" => 0,
    "m_chAreaPortalBits.002" => 0,
    "m_chAreaPortalBits.003" => 0,
    "m_chAreaPortalBits.004" => 0,
    "DT_Local.m_iHideHUD" => 0,
    "DT_Local.m_flFOVRate" => 1,
    "DT_Local.m_bDucked" => 0,
    "DT_Local.m_bDucking" => 0,
    "DT_Local.m_flLastDuckTime" => 1,
    "DT_Local.m_bWearingSuit" => 0,
    "DT_Local.m_skybox3d.scale" => 0,
    "DT_Local.m_skybox3d.origin" => 2,
    "DT_Local.m_skybox3d.area" => 0,
    "DT_Local.m_skybox3d.fog.enable" => 0,
    "DT_Local.m_skybox3d.fog.dirPrimary" => 2,
    "DT_Local.m_skybox3d.fog.colorPrimary" => 0,
    "DT_Local.m_skybox3d.fog.colorSecondary" => 0,
    "DT_Local.m_skybox3d.fog.start" => 1,
    "DT_Local.m_skybox3d.fog.end" => 1,
    "DT_Local.m_skybox3d.fog.maxdensity" => 1,
    "DT_Local.m_skybox3d.fog.HDRColorScale" => 1,
    "DT_Local.m_audio.soundscapeIndex" => 0,
    "DT_Local.m_audio.entIndex" => 0,
    "DT_CollisionProperty.m_vecMaxs" => 2,
    "DT_LocalPlayerExclusive.m_nNextThinkTick" => 0,
    "DT_LocalPlayerExclusive.m_hLastWeapon" => 0,
    "DT_LocalPlayerExclusive.m_vecBaseVelocity" => 2,
    "DT_LocalPlayerExclusive.m_flDeathTime" => 1,
    "DT_LocalPlayerExclusive.m_flNextDecalTime" => 1,
    "DT_LocalPlayerExclusive.m_hTonemapController" => 0,
    "m_iWeaponPurchasesThisRound.001" => 0,
    "m_iWeaponPurchasesThisRound.003" => 0,
    "m_iWeaponPurchasesThisRound.007" => 0,
    "m_iWeaponPurchasesThisRound.008" => 0,
    "m_iWeaponPurchasesThisRound.009" => 0,
    "m_iWeaponPurchasesThisRound.010" => 0,
    "m_iWeaponPurchasesThisRound.011" => 0,
    "m_iWeaponPurchasesThisRound.013" => 0,
    "m_iWeaponPurchasesThisRound.016" => 0,
    "m_iWeaponPurchasesThisRound.017" => 0,
    "m_iWeaponPurchasesThisRound.019" => 0,
    "m_iWeaponPurchasesThisRound.024" => 0,
    "m_iWeaponPurchasesThisRound.027" => 0,
    "m_iWeaponPurchasesThisRound.030" => 0,
    "m_iWeaponPurchasesThisRound.031" => 0,
    "m_iWeaponPurchasesThisRound.033" => 0,
    "m_iWeaponPurchasesThisRound.034" => 0,
    "m_iWeaponPurchasesThisRound.036" => 0,
    "m_iWeaponPurchasesThisRound.038" => 0,
    "m_iWeaponPurchasesThisRound.039" => 0,
    "m_iWeaponPurchasesThisRound.040" => 0,
    "m_iWeaponPurchasesThisRound.043" => 0,
    "m_iWeaponPurchasesThisRound.044" => 0,
    "m_iWeaponPurchasesThisRound.045" => 0,
    "m_iWeaponPurchasesThisRound.046" => 0,
    "m_iWeaponPurchasesThisRound.047" => 0,
    "m_iWeaponPurchasesThisRound.048" => 0,
    "m_iWeaponPurchasesThisRound.050" => 0,
    "m_iWeaponPurchasesThisRound.051" => 0,
    "m_iWeaponPurchasesThisRound.055" => 0,
    "m_iWeaponPurchasesThisRound.060" => 0,
    "m_iWeaponPurchasesThisRound.061" => 0,
    "m_iWeaponPurchasesThisRound.064" => 0,
    "DT_CollisionProperty.m_usSolidFlags" => 0,
    "DT_CSLocalPlayerExclusive.m_flStamina" => 1,
    "DT_CSLocalPlayerExclusive.m_iShotsFired" => 0,
    "DT_CSLocalPlayerExclusive.m_bDuckOverride" => 0,
    "DT_CSLocalPlayerExclusive.m_flVelocityModifier" => 1,
    "DT_CSLocalPlayerExclusive.m_nQuestProgressReason" => 0,
    "m_bSpottedByMask.000" => 0,
    "m_iWeaponPurchasesThisMatch.001" => 0,
    "m_iWeaponPurchasesThisMatch.003" => 0,
    "m_iWeaponPurchasesThisMatch.007" => 0,
    "m_iWeaponPurchasesThisMatch.008" => 0,
    "m_iWeaponPurchasesThisMatch.009" => 0,
    "m_iWeaponPurchasesThisMatch.010" => 0,
    "m_iWeaponPurchasesThisMatch.011" => 0,
    "m_iWeaponPurchasesThisMatch.013" => 0,
    "m_iWeaponPurchasesThisMatch.016" => 0,
    "m_iWeaponPurchasesThisMatch.017" => 0,
    "m_iWeaponPurchasesThisMatch.019" => 0,
    "m_iWeaponPurchasesThisMatch.024" => 0,
    "m_iWeaponPurchasesThisMatch.027" => 0,
    "m_iWeaponPurchasesThisMatch.030" => 0,
    "m_iWeaponPurchasesThisMatch.031" => 0,
    "m_iWeaponPurchasesThisMatch.033" => 0,
    "m_iWeaponPurchasesThisMatch.034" => 0,
    "m_iWeaponPurchasesThisMatch.036" => 0,
    "m_iWeaponPurchasesThisMatch.038" => 0,
    "m_iWeaponPurchasesThisMatch.039" => 0,
    "m_iWeaponPurchasesThisMatch.040" => 0,
    "m_iWeaponPurchasesThisMatch.043" => 0,
    "m_iWeaponPurchasesThisMatch.044" => 0,
    "m_iWeaponPurchasesThisMatch.045" => 0,
    "m_iWeaponPurchasesThisMatch.046" => 0,
    "m_iWeaponPurchasesThisMatch.047" => 0,
    "m_iWeaponPurchasesThisMatch.048" => 0,
    "m_iWeaponPurchasesThisMatch.050" => 0,
    "m_iWeaponPurchasesThisMatch.051" => 0,
    "m_iWeaponPurchasesThisMatch.055" => 0,
    "m_iWeaponPurchasesThisMatch.060" => 0,
    "m_iWeaponPurchasesThisMatch.061" => 0,
    "m_iWeaponPurchasesThisMatch.064" => 0,
    "m_EquippedLoadoutItemDefIndices.000" => 0,
    "m_EquippedLoadoutItemDefIndices.001" => 0,
    "m_EquippedLoadoutItemDefIndices.002" => 0,
    "m_EquippedLoadoutItemDefIndices.003" => 0,
    "m_EquippedLoadoutItemDefIndices.004" => 0,
    "m_EquippedLoadoutItemDefIndices.005" => 0,
    "m_EquippedLoadoutItemDefIndices.006" => 0,
    "m_EquippedLoadoutItemDefIndices.008" => 0,
    "m_EquippedLoadoutItemDefIndices.009" => 0,
    "m_EquippedLoadoutItemDefIndices.010" => 0,
    "m_EquippedLoadoutItemDefIndices.011" => 0,
    "m_EquippedLoadoutItemDefIndices.012" => 0,
    "m_EquippedLoadoutItemDefIndices.014" => 0,
    "m_EquippedLoadoutItemDefIndices.015" => 0,
    "m_EquippedLoadoutItemDefIndices.016" => 0,
    "m_EquippedLoadoutItemDefIndices.017" => 0,
    "m_EquippedLoadoutItemDefIndices.018" => 0,
    "m_EquippedLoadoutItemDefIndices.019" => 0,
    "m_EquippedLoadoutItemDefIndices.020" => 0,
    "m_EquippedLoadoutItemDefIndices.021" => 0,
    "m_EquippedLoadoutItemDefIndices.022" => 0,
    "m_EquippedLoadoutItemDefIndices.023" => 0,
    "m_EquippedLoadoutItemDefIndices.024" => 0,
    "m_EquippedLoadoutItemDefIndices.025" => 0,
    "m_EquippedLoadoutItemDefIndices.026" => 0,
    "m_EquippedLoadoutItemDefIndices.027" => 0,
    "m_EquippedLoadoutItemDefIndices.028" => 0,
    "m_EquippedLoadoutItemDefIndices.029" => 0,
    "m_EquippedLoadoutItemDefIndices.030" => 0,
    "m_EquippedLoadoutItemDefIndices.032" => 0,
    "m_EquippedLoadoutItemDefIndices.033" => 0,
    "m_EquippedLoadoutItemDefIndices.034" => 0,
    "m_EquippedLoadoutItemDefIndices.035" => 0,
    "m_EquippedLoadoutItemDefIndices.054" => 0,
    "m_rank.005" => 0,
    "m_vecPlayerPatchEconIndices.000" => 0,
    "m_vecPlayerPatchEconIndices.001" => 0,
    "m_vecPlayerPatchEconIndices.002" => 0,
    "DT_CollisionProperty.m_vecMins" => 2,
    "DT_BaseEntity.m_nModelIndex" => 0,
    "DT_BaseEntity.m_fEffects" => 0,
    "DT_BaseEntity.m_iTeamNum" => 0,
    "DT_BaseEntity.m_iPendingTeamNum" => 0,
    "DT_BaseEntity.movetype" => 0,
    "DT_BaseEntity.m_bAnimatedEveryTick" => 0,
    "DT_BaseEntity.m_bSpotted" => 0,
    "DT_BaseEntity.m_flLastMadeNoiseTime" => 1,
    "DT_BaseAnimating.m_nForceBone" => 0,
    "DT_BaseAnimating.m_vecForce" => 2,
    "DT_BaseAnimating.m_nBody" => 0,
    "DT_BaseAnimating.m_bClientSideRagdoll" => 0,
    "DT_BaseCombatCharacter.m_LastHitGroup" => 0,
    "DT_BaseCombatCharacter.m_hActiveWeapon" => 0,
    "DT_BaseCombatCharacter.m_flTimeOfLastInjury" => 1,
    "DT_BaseCombatCharacter.m_nRelativeDirectionOfLastInjury" => 0,
    "DT_BasePlayer.m_afPhysicsFlags" => 0,
    "DT_BasePlayer.m_hGroundEntity" => 0,
    "DT_BasePlayer.m_iHealth" => 0,
    "DT_BasePlayer.m_lifeState" => 0,
    "DT_BasePlayer.m_iObserverMode" => 0,
    "DT_BasePlayer.m_hObserverTarget" => 0,
    "DT_BasePlayer.m_iFOVStart" => 0,
    "DT_BasePlayer.m_hZoomOwner" => 0,
    "DT_BasePlayer.m_hViewModel" => 5,
    "DT_BasePlayer.m_szLastPlaceName" => 4,
    "DT_BasePlayer.m_ubEFNoInterpParity" => 0,
    "DT_BasePlayer.m_iDeathPostEffect" => 0,
    "DT_BasePlayer.m_hPostProcessCtrl" => 0,
    "DT_BasePlayer.m_hColorCorrectionCtrl" => 0,
    "DT_CSPlayer.m_iAddonBits" => 0,
    "DT_CSPlayer.m_iPrimaryAddon" => 0,
    "DT_CSPlayer.m_iSecondaryAddon" => 0,
    "DT_CSPlayer.m_bWaitForNoAttack" => 0,
    "DT_CSPlayer.m_iPlayerState" => 0,
    "DT_CSPlayer.m_iAccount" => 0,
    "DT_CSPlayer.m_iStartAccount" => 0,
    "DT_CSPlayer.m_totalHitsOnServer" => 0,
    "DT_CSPlayer.m_bInBombZone" => 0,
    "DT_CSPlayer.m_bInBuyZone" => 0,
    "DT_CSPlayer.m_iClass" => 0,
    "DT_CSPlayer.m_ArmorValue" => 0,
    "DT_CSPlayer.m_bHasDefuser" => 0,
    "DT_CSPlayer.m_bIsDefusing" => 0,
    "DT_CSPlayer.m_bIsScoped" => 0,
    "DT_CSPlayer.m_bIsWalking" => 0,
    "DT_CSPlayer.m_bResumeZoom" => 0,
    "DT_CSPlayer.m_bHasMovedSinceSpawn" => 0,
    "DT_CSPlayer.m_iNumRoundKills" => 0,
    "DT_CSPlayer.m_fMolotovUseTime" => 1,
    "DT_CSPlayer.m_fMolotovDamageTime" => 1,
    "DT_CSPlayer.m_unMusicID" => 0,
    "DT_CSPlayer.m_bHasHelmet" => 0,
    "DT_CSPlayer.m_nHeavyAssaultSuitCooldownRemaining" => 0,
    "DT_CSPlayer.m_flFlashDuration" => 1,
    "DT_CSPlayer.m_flFlashMaxAlpha" => 1,
    "DT_CSPlayer.m_iProgressBarDuration" => 0,
    "DT_CSPlayer.m_flProgressBarStartTime" => 1,
    "DT_CSPlayer.m_hRagdoll" => 0,
    "DT_CSPlayer.m_hPlayerPing" => 0,
    "DT_CSPlayer.m_unCurrentEquipmentValue" => 0,
    "DT_CSPlayer.m_unRoundStartEquipmentValue" => 0,
    "DT_CSPlayer.m_unFreezetimeEndEquipmentValue" => 0,
    "DT_CSPlayer.m_nLastKillerIndex" => 0,
    "DT_CSPlayer.m_nLastConcurrentKilled" => 0,
    "DT_CSPlayer.m_bIsLookingAtWeapon" => 0,
    "DT_CSPlayer.m_bIsHoldingLookAtWeapon" => 0,
    "DT_CSPlayer.m_iNumRoundKillsHeadshots" => 0,
    "DT_CSPlayer.m_unTotalRoundDamageDealt" => 0,
    "DT_CSPlayer.m_flLowerBodyYawTarget" => 1,
    "DT_CSPlayer.m_bStrafing" => 0,
    "m_iMatchStats_Deaths.011" => 0,
    "m_iMatchStats_Deaths.012" => 0,
    "m_iMatchStats_Deaths.013" => 0,
    "m_iMatchStats_Deaths.014" => 0,
    "m_iMatchStats_Deaths.015" => 0,
    "m_iMatchStats_Deaths.016" => 0,
    "m_iMatchStats_Deaths.017" => 0,
    "m_iMatchStats_Deaths.018" => 0,
    "m_iMatchStats_Deaths.019" => 0,
    "m_iMatchStats_Deaths.020" => 0,
    "m_iMatchStats_Deaths.021" => 0,
    "m_iMatchStats_Deaths.022" => 0,
    "m_iMatchStats_Deaths.023" => 0,
    "m_iMatchStats_Deaths.024" => 0,
    "m_iMatchStats_Deaths.025" => 0,
    "m_iMatchStats_Deaths.026" => 0,
    "m_iMatchStats_Deaths.027" => 0,
    "m_iMatchStats_Deaths.028" => 0,
    "m_iMatchStats_Deaths.029" => 0,
    "m_iMatchStats_Assists.001" => 0,
    "m_iMatchStats_Assists.002" => 0,
    "m_iMatchStats_Assists.003" => 0,
    "m_iMatchStats_Assists.005" => 0,
    "m_iMatchStats_Assists.006" => 0,
    "m_iMatchStats_Assists.007" => 0,
    "m_iMatchStats_Assists.008" => 0,
    "m_iMatchStats_Assists.009" => 0,
    "m_iMatchStats_Assists.010" => 0,
    "m_iMatchStats_Assists.012" => 0,
    "m_iMatchStats_Assists.013" => 0,
    "m_iMatchStats_Assists.016" => 0,
    "m_iMatchStats_Assists.017" => 0,
    "m_iMatchStats_Assists.018" => 0,
    "m_iMatchStats_Assists.019" => 0,
    "m_iMatchStats_Assists.020" => 0,
    "m_iMatchStats_Assists.021" => 0,
    "m_iMatchStats_Assists.022" => 0,
    "m_iMatchStats_Assists.023" => 0,
    "m_iMatchStats_Assists.024" => 0,
    "m_iMatchStats_Assists.025" => 0,
    "m_iMatchStats_Assists.026" => 0,
    "m_iMatchStats_Assists.027" => 0,
    "m_iMatchStats_HeadShotKills.000" => 0,
    "m_iMatchStats_HeadShotKills.001" => 0,
    "m_iMatchStats_HeadShotKills.002" => 0,
    "m_iMatchStats_HeadShotKills.003" => 0,
    "m_iMatchStats_HeadShotKills.004" => 0,
    "m_iMatchStats_HeadShotKills.006" => 0,
    "m_iMatchStats_HeadShotKills.007" => 0,
    "m_iMatchStats_HeadShotKills.008" => 0,
    "m_iMatchStats_HeadShotKills.009" => 0,
    "m_iMatchStats_HeadShotKills.010" => 0,
    "m_iMatchStats_HeadShotKills.011" => 0,
    "m_iMatchStats_HeadShotKills.012" => 0,
    "m_iMatchStats_HeadShotKills.013" => 0,
    "m_iMatchStats_HeadShotKills.014" => 0,
    "m_iMatchStats_HeadShotKills.015" => 0,
    "m_iMatchStats_HeadShotKills.016" => 0,
    "m_iMatchStats_HeadShotKills.017" => 0,
    "m_iMatchStats_HeadShotKills.018" => 0,
    "m_iMatchStats_HeadShotKills.019" => 0,
    "m_iMatchStats_HeadShotKills.020" => 0,
    "m_iMatchStats_HeadShotKills.022" => 0,
    "m_iMatchStats_HeadShotKills.023" => 0,
    "m_iMatchStats_HeadShotKills.024" => 0,
    "m_iMatchStats_HeadShotKills.025" => 0,
    "m_iMatchStats_HeadShotKills.026" => 0,
    "m_iMatchStats_HeadShotKills.027" => 0,
    "m_iMatchStats_HeadShotKills.029" => 0,
    "m_iMatchStats_Objective.003" => 0,
    "m_iMatchStats_Objective.005" => 0,
    "m_iMatchStats_Objective.006" => 0,
    "m_iMatchStats_Objective.007" => 0,
    "m_iMatchStats_Objective.008" => 0,
    "m_iMatchStats_Objective.015" => 0,
    "m_iMatchStats_Objective.016" => 0,
    "m_iMatchStats_Objective.024" => 0,
    "m_iMatchStats_CashEarned.000" => 0,
    "m_iMatchStats_CashEarned.001" => 0,
    "m_iMatchStats_CashEarned.002" => 0,
    "m_iMatchStats_CashEarned.003" => 0,
    "m_iMatchStats_CashEarned.004" => 0,
    "m_iMatchStats_CashEarned.005" => 0,
    "m_iMatchStats_CashEarned.006" => 0,
    "m_iMatchStats_CashEarned.007" => 0,
    "m_iMatchStats_CashEarned.008" => 0,
    "m_iMatchStats_CashEarned.009" => 0,
    "m_iMatchStats_CashEarned.010" => 0,
    "m_iMatchStats_CashEarned.011" => 0,
    "m_iMatchStats_CashEarned.012" => 0,
    "m_iMatchStats_CashEarned.013" => 0,
    "m_iMatchStats_CashEarned.014" => 0,
    "m_iMatchStats_CashEarned.015" => 0,
    "m_iMatchStats_CashEarned.016" => 0,
    "m_iMatchStats_CashEarned.017" => 0,
    "m_iMatchStats_CashEarned.018" => 0,
    "m_iMatchStats_CashEarned.019" => 0,
    "m_iMatchStats_CashEarned.020" => 0,
    "m_iMatchStats_CashEarned.021" => 0,
    "m_iMatchStats_CashEarned.022" => 0,
    "m_iMatchStats_CashEarned.023" => 0,
    "m_iMatchStats_CashEarned.024" => 0,
    "m_iMatchStats_CashEarned.025" => 0,
    "m_iMatchStats_CashEarned.026" => 0,
    "m_iMatchStats_CashEarned.027" => 0,
    "m_iMatchStats_CashEarned.028" => 0,
    "m_iMatchStats_CashEarned.029" => 0,
    "m_iMatchStats_UtilityDamage.002" => 0,
    "m_iMatchStats_UtilityDamage.003" => 0,
    "m_iMatchStats_UtilityDamage.004" => 0,
    "m_iMatchStats_UtilityDamage.005" => 0,
    "m_iMatchStats_UtilityDamage.006" => 0,
    "m_iMatchStats_UtilityDamage.008" => 0,
    "m_iMatchStats_UtilityDamage.011" => 0,
    "m_iMatchStats_UtilityDamage.012" => 0,
    "m_iMatchStats_UtilityDamage.013" => 0,
    "m_iMatchStats_UtilityDamage.017" => 0,
    "m_iMatchStats_UtilityDamage.018" => 0,
    "m_iMatchStats_UtilityDamage.019" => 0,
    "m_iMatchStats_UtilityDamage.020" => 0,
    "m_iMatchStats_UtilityDamage.025" => 0,
    "m_iMatchStats_UtilityDamage.026" => 0,
    "m_iMatchStats_UtilityDamage.027" => 0,
    "m_iMatchStats_UtilityDamage.029" => 0,
    "m_iMatchStats_EnemiesFlashed.000" => 0,
    "m_iMatchStats_EnemiesFlashed.001" => 0,
    "m_iMatchStats_EnemiesFlashed.003" => 0,
    "m_iMatchStats_EnemiesFlashed.005" => 0,
    "m_iMatchStats_EnemiesFlashed.007" => 0,
    "m_iMatchStats_EnemiesFlashed.008" => 0,
    "m_iMatchStats_EnemiesFlashed.009" => 0,
    "m_iMatchStats_EnemiesFlashed.010" => 0,
    "m_iMatchStats_EnemiesFlashed.011" => 0,
    "m_iMatchStats_EnemiesFlashed.013" => 0,
    "m_iMatchStats_EnemiesFlashed.014" => 0,
    "m_iMatchStats_EnemiesFlashed.016" => 0,
    "m_iMatchStats_EnemiesFlashed.017" => 0,
    "m_iMatchStats_EnemiesFlashed.018" => 0,
    "m_iMatchStats_EnemiesFlashed.019" => 0,
    "m_iMatchStats_EnemiesFlashed.020" => 0,
    "m_iMatchStats_EnemiesFlashed.021" => 0,
    "m_iMatchStats_EnemiesFlashed.022" => 0,
    "m_iMatchStats_EnemiesFlashed.024" => 0,
    "m_iMatchStats_EnemiesFlashed.025" => 0,
    "m_iMatchStats_EnemiesFlashed.026" => 0,
    "m_iMatchStats_EnemiesFlashed.027" => 0,
    "m_iMatchStats_EnemiesFlashed.029" => 0,
    "m_iMatchStats_Kills.000" => 0,
    "m_iMatchStats_Kills.001" => 0,
    "m_iMatchStats_Kills.002" => 0,
    "m_iMatchStats_Kills.003" => 0,
    "m_iMatchStats_Kills.004" => 0,
    "m_iMatchStats_Kills.005" => 0,
    "m_iMatchStats_Kills.006" => 0,
    "m_iMatchStats_Kills.007" => 0,
    "m_iMatchStats_Kills.008" => 0,
    "m_iMatchStats_Kills.009" => 0,
    "m_iMatchStats_Kills.010" => 0,
    "m_iMatchStats_Kills.011" => 0,
    "m_iMatchStats_Kills.012" => 0,
    "m_iMatchStats_Kills.013" => 0,
    "m_iMatchStats_Kills.014" => 0,
    "m_iMatchStats_Kills.015" => 0,
    "m_iMatchStats_Kills.016" => 0,
    "m_iMatchStats_Kills.017" => 0,
    "m_iMatchStats_Kills.018" => 0,
    "m_iMatchStats_Kills.019" => 0,
    "m_iMatchStats_Kills.020" => 0,
    "m_iMatchStats_Kills.021" => 0,
    "m_iMatchStats_Kills.022" => 0,
    "m_iMatchStats_Kills.023" => 0,
    "m_iMatchStats_Kills.024" => 0,
    "m_iMatchStats_Kills.025" => 0,
    "m_iMatchStats_Kills.026" => 0,
    "m_iMatchStats_Kills.027" => 0,
    "m_iMatchStats_Kills.028" => 0,
    "m_iMatchStats_Kills.029" => 0,
    "m_iMatchStats_Damage.000" => 0,
    "m_iMatchStats_Damage.001" => 0,
    "m_iMatchStats_Damage.002" => 0,
    "m_iMatchStats_Damage.003" => 0,
    "m_iMatchStats_Damage.004" => 0,
    "m_iMatchStats_Damage.005" => 0,
    "m_iMatchStats_Damage.006" => 0,
    "m_iMatchStats_Damage.007" => 0,
    "m_iMatchStats_Damage.008" => 0,
    "m_iMatchStats_Damage.009" => 0,
    "m_iMatchStats_Damage.010" => 0,
    "m_iMatchStats_Damage.011" => 0,
    "m_iMatchStats_Damage.012" => 0,
    "m_iMatchStats_Damage.013" => 0,
    "m_iMatchStats_Damage.014" => 0,
    "m_iMatchStats_Damage.015" => 0,
    "m_iMatchStats_Damage.016" => 0,
    "m_iMatchStats_Damage.017" => 0,
    "m_iMatchStats_Damage.018" => 0,
    "m_iMatchStats_Damage.019" => 0,
    "m_iMatchStats_Damage.020" => 0,
    "m_iMatchStats_Damage.021" => 0,
    "m_iMatchStats_Damage.022" => 0,
    "m_iMatchStats_Damage.023" => 0,
    "m_iMatchStats_Damage.024" => 0,
    "m_iMatchStats_Damage.025" => 0,
    "m_iMatchStats_Damage.026" => 0,
    "m_iMatchStats_Damage.027" => 0,
    "m_iMatchStats_Damage.028" => 0,
    "m_iMatchStats_Damage.029" => 0,
    "m_iMatchStats_EquipmentValue.000" => 0,
    "m_iMatchStats_EquipmentValue.001" => 0,
    "m_iMatchStats_EquipmentValue.002" => 0,
    "m_iMatchStats_EquipmentValue.003" => 0,
    "m_iMatchStats_EquipmentValue.004" => 0,
    "m_iMatchStats_EquipmentValue.005" => 0,
    "m_iMatchStats_EquipmentValue.006" => 0,
    "m_iMatchStats_EquipmentValue.007" => 0,
    "m_iMatchStats_EquipmentValue.008" => 0,
    "m_iMatchStats_EquipmentValue.009" => 0,
    "m_iMatchStats_EquipmentValue.010" => 0,
    "m_iMatchStats_EquipmentValue.011" => 0,
    "m_iMatchStats_EquipmentValue.012" => 0,
    "m_iMatchStats_EquipmentValue.013" => 0,
    "m_iMatchStats_EquipmentValue.014" => 0,
    "m_iMatchStats_EquipmentValue.015" => 0,
    "m_iMatchStats_EquipmentValue.016" => 0,
    "m_iMatchStats_EquipmentValue.017" => 0,
    "m_iMatchStats_EquipmentValue.018" => 0,
    "m_iMatchStats_EquipmentValue.019" => 0,
    "m_iMatchStats_EquipmentValue.020" => 0,
    "m_iMatchStats_EquipmentValue.021" => 0,
    "m_iMatchStats_EquipmentValue.022" => 0,
    "m_iMatchStats_EquipmentValue.023" => 0,
    "m_iMatchStats_EquipmentValue.024" => 0,
    "m_iMatchStats_EquipmentValue.025" => 0,
    "m_iMatchStats_EquipmentValue.026" => 0,
    "m_iMatchStats_EquipmentValue.027" => 0,
    "m_iMatchStats_EquipmentValue.028" => 0,
    "m_iMatchStats_EquipmentValue.029" => 0,
    "m_iMatchStats_MoneySaved.000" => 0,
    "m_iMatchStats_MoneySaved.001" => 0,
    "m_iMatchStats_MoneySaved.002" => 0,
    "m_iMatchStats_MoneySaved.003" => 0,
    "m_iMatchStats_MoneySaved.004" => 0,
    "m_iMatchStats_MoneySaved.005" => 0,
    "m_iMatchStats_MoneySaved.006" => 0,
    "m_iMatchStats_MoneySaved.007" => 0,
    "m_iMatchStats_MoneySaved.008" => 0,
    "m_iMatchStats_MoneySaved.009" => 0,
    "m_iMatchStats_MoneySaved.010" => 0,
    "m_iMatchStats_MoneySaved.011" => 0,
    "m_iMatchStats_MoneySaved.012" => 0,
    "m_iMatchStats_MoneySaved.013" => 0,
    "m_iMatchStats_MoneySaved.014" => 0,
    "m_iMatchStats_MoneySaved.015" => 0,
    "m_iMatchStats_MoneySaved.016" => 0,
    "m_iMatchStats_MoneySaved.017" => 0,
    "m_iMatchStats_MoneySaved.018" => 0,
    "m_iMatchStats_MoneySaved.019" => 0,
    "m_iMatchStats_MoneySaved.020" => 0,
    "m_iMatchStats_MoneySaved.021" => 0,
    "m_iMatchStats_MoneySaved.022" => 0,
    "m_iMatchStats_MoneySaved.023" => 0,
    "m_iMatchStats_MoneySaved.024" => 0,
    "m_iMatchStats_MoneySaved.025" => 0,
    "m_iMatchStats_MoneySaved.026" => 0,
    "m_iMatchStats_MoneySaved.027" => 0,
    "m_iMatchStats_MoneySaved.028" => 0,
    "m_iMatchStats_MoneySaved.029" => 0,
    "m_iMatchStats_KillReward.000" => 0,
    "m_iMatchStats_KillReward.001" => 0,
    "m_iMatchStats_KillReward.002" => 0,
    "m_iMatchStats_KillReward.003" => 0,
    "m_iMatchStats_KillReward.004" => 0,
    "m_iMatchStats_KillReward.005" => 0,
    "m_iMatchStats_KillReward.006" => 0,
    "m_iMatchStats_KillReward.007" => 0,
    "m_iMatchStats_KillReward.008" => 0,
    "m_iMatchStats_KillReward.009" => 0,
    "m_iMatchStats_KillReward.010" => 0,
    "m_iMatchStats_KillReward.011" => 0,
    "m_iMatchStats_KillReward.012" => 0,
    "m_iMatchStats_KillReward.013" => 0,
    "m_iMatchStats_KillReward.014" => 0,
    "m_iMatchStats_KillReward.015" => 0,
    "m_iMatchStats_KillReward.016" => 0,
    "m_iMatchStats_KillReward.017" => 0,
    "m_iMatchStats_KillReward.018" => 0,
    "m_iMatchStats_KillReward.019" => 0,
    "m_iMatchStats_KillReward.020" => 0,
    "m_iMatchStats_KillReward.021" => 0,
    "m_iMatchStats_KillReward.022" => 0,
    "m_iMatchStats_KillReward.023" => 0,
    "m_iMatchStats_KillReward.024" => 0,
    "m_iMatchStats_KillReward.025" => 0,
    "m_iMatchStats_KillReward.026" => 0,
    "m_iMatchStats_KillReward.027" => 0,
    "m_iMatchStats_KillReward.028" => 0,
    "m_iMatchStats_KillReward.029" => 0,
    "m_iMatchStats_LiveTime.000" => 0,
    "m_iMatchStats_LiveTime.001" => 0,
    "m_iMatchStats_LiveTime.002" => 0,
    "m_iMatchStats_LiveTime.003" => 0,
    "m_iMatchStats_LiveTime.004" => 0,
    "m_iMatchStats_LiveTime.005" => 0,
    "m_iMatchStats_LiveTime.006" => 0,
    "m_iMatchStats_LiveTime.007" => 0,
    "m_iMatchStats_LiveTime.008" => 0,
    "m_iMatchStats_LiveTime.009" => 0,
    "m_iMatchStats_LiveTime.010" => 0,
    "m_iMatchStats_LiveTime.011" => 0,
    "m_iMatchStats_LiveTime.012" => 0,
    "m_iMatchStats_LiveTime.013" => 0,
    "m_iMatchStats_LiveTime.014" => 0,
    "m_iMatchStats_LiveTime.015" => 0,
    "m_iMatchStats_LiveTime.016" => 0,
    "m_iMatchStats_LiveTime.017" => 0,
    "m_iMatchStats_LiveTime.018" => 0,
    "m_iMatchStats_LiveTime.019" => 0,
    "m_iMatchStats_LiveTime.020" => 0,
    "m_iMatchStats_LiveTime.021" => 0,
    "m_iMatchStats_LiveTime.022" => 0,
    "m_iMatchStats_LiveTime.023" => 0,
    "m_iMatchStats_LiveTime.024" => 0,
    "m_iMatchStats_LiveTime.025" => 0,
    "m_iMatchStats_LiveTime.026" => 0,
    "m_iMatchStats_LiveTime.027" => 0,
    "m_iMatchStats_LiveTime.028" => 0,
    "m_iMatchStats_LiveTime.029" => 0,
    "m_iMatchStats_Deaths.000" => 0,
    "m_iMatchStats_Deaths.001" => 0,
    "m_iMatchStats_Deaths.002" => 0,
    "m_iMatchStats_Deaths.003" => 0,
    "m_iMatchStats_Deaths.004" => 0,
    "m_iMatchStats_Deaths.005" => 0,
    "m_iMatchStats_Deaths.006" => 0,
    "m_iMatchStats_Deaths.007" => 0,
    "m_iMatchStats_Deaths.008" => 0,
    "m_iMatchStats_Deaths.009" => 0,
    "m_iMatchStats_Deaths.010" => 0,
};

#[pymodule]
fn demoparser(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DemoParser>()?;
    Ok(())
}
