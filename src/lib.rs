mod parsing;

use arrow::ffi;
use flate2::read::GzDecoder;

use itertools::Itertools;
use parsing::demo_parsing::collect_data::AMMO_ID;
use parsing::demo_parsing::collect_data::NAME_ID;
use parsing::demo_parsing::collect_data::TICK_ID;
use parsing::demo_parsing::*;
use parsing::parser::Parser;
//use parsing::tick_cache::gather_props_backwards;
use ahash::HashMap;
use parsing::utils::Header;
use parsing::variants::PropData;
use phf::phf_map;
use polars::prelude::ArrowField;
use polars::prelude::NamedFrom;
use polars::series::Series;
use polars_arrow::export::arrow;
use polars_arrow::prelude::ArrayRef;

use pyo3::exceptions::PyFileNotFoundError;
use pyo3::exceptions::PyKeyError;
use pyo3::ffi::Py_uintptr_t;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::{PyAny, PyObject, PyResult};
use pyo3::{PyErr, Python};

use parsing::utils::CACHE_ID_MAP;
use parsing::variants::*;
use pyo3::types::IntoPyDict;
use std::io::prelude::*;
use std::path::Path;
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

    #[args(py_kwargs = "**")]
    pub fn parse_events(
        &self,
        py: Python<'_>,
        event_name: String,
        py_kwargs: Option<&PyDict>,
    ) -> PyResult<Py<PyAny>> {
        let (rounds, wanted_props) = parse_kwargs_event(py_kwargs);
        /*
        let real_props = rm_user_friendly_names(&wanted_props);
        let unk_props = check_validity_props(&real_props);
        if !unk_props.is_empty() {
            return Err(PyKeyError::new_err(format!(
                "Unknown fields: {:?}",
                unk_props
            )));
        }
        */
        let parse_props = !wanted_props.is_empty() || rounds;
        let mut parser = Parser::new(
            self.path.clone(),
            true,
            false,
            //vec![],
            (1000..1001).collect(),
            vec![],
            vec!["player@m_vecOrigin_X".to_string()],
            event_name.to_string(),
            false,
            false,
            false,
            1000000,
            vec![],
        );
        match parser {
            Err(e) => Err(PyFileNotFoundError::new_err(format!(
                "Couldnt read demo file. Error: {}",
                e
            ))),
            Ok(mut parser) => {
                let h: Header = parser.parse_demo_header();

                parser.settings.playback_frames = (h.playback_ticks + 100) as usize;
                let mut ss = vec![];
                parser.start_parsing();

                let event_series = parser.series_from_events(&parser.state.game_events);

                let column_names: Vec<&str> =
                    event_series.iter().map(|x| x.name().clone()).collect();
                let mut rows = 0;
                for s in &event_series {
                    rows = s.len().max(rows);
                    let py_series = rust_series_to_py_series(&s).unwrap();
                    ss.push(py_series);
                }

                if rows == 0 {
                    let pandas = py.import("pandas")?;
                    let mut dict = HashMap::default();
                    dict.insert("columns", ["EVENT NOT FOUND"].to_object(py));
                    let py_dict = dict.into_py_dict(py);
                    let df = pandas.call_method("DataFrame", (), Some(py_dict)).unwrap();
                    return Ok(df.to_object(py));
                }

                let polars = py.import("polars").unwrap();
                let df = polars.call_method1("DataFrame", (ss,)).unwrap();
                df.setattr("columns", column_names.to_object(py)).unwrap();
                let pandas_df = df.call_method0("to_pandas").unwrap();
                Ok(pandas_df.to_object(py))
            }
        }
    }

    #[args(py_kwargs = "**")]
    pub fn parse_ticks(
        &self,
        py: Python,
        mut wanted_props: Vec<String>,
        py_kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        let mut real_props = rm_user_friendly_names(&wanted_props);
        /*
        let unk_props = check_validity_props(&real_props);
        if !unk_props.is_empty() {
            return Err(PyKeyError::new_err(format!(
                "Unknown fields: {:?}",
                unk_props
            )));
        }
        */
        let (wanted_players, wanted_ticks) = parse_kwargs_ticks(py_kwargs);
        let wanted_ticks_len = wanted_ticks.len();
        let biggest_wanted_tick = if wanted_ticks_len > 0 {
            wanted_ticks.iter().max().unwrap()
        } else {
            &99999999
        };
        let mut parser = Parser::new(
            self.path.clone(),
            true,
            false,
            //vec![],
            (1000..100001).collect(),
            vec![],
            wanted_props.clone(),
            "player_death".to_string(),
            false,
            false,
            false,
            1000000,
            vec![],
        );

        match parser {
            Err(e) => Err(PyFileNotFoundError::new_err(format!(
                "Couldnt read demo file. Error: {}",
                e
            ))),
            Ok(mut parser) => {
                let h: Header = parser.parse_demo_header();

                parser.start_parsing();

                real_props.push("tick".to_string());
                real_props.push("steamid".to_string());
                real_props.push("name".to_string());
                wanted_props.push("tick".to_string());
                wanted_props.push("steamid".to_string());
                wanted_props.push("name".to_string());

                let mut series = vec![];

                let v = &parser.state.output[&-20];
                match &v.data {
                    VarVec::String(i) => {
                        let s = Series::new("weapon", i);
                        println!("{:?}", s);
                        let py_series = rust_series_to_py_series(&s).unwrap();
                        series.push(py_series);
                    }
                    _ => {}
                }
                let v = &parser.state.output[&NAME_ID];
                match &v.data {
                    VarVec::String(i) => {
                        let s = Series::new("name", i);
                        println!("{:?}", s);
                        let py_series = rust_series_to_py_series(&s).unwrap();
                        series.push(py_series);
                    }
                    _ => {}
                }
                let v = &parser.state.output[&TICK_ID];
                match &v.data {
                    VarVec::I32(i) => {
                        let s = Series::new("tick", i);
                        println!("{:?}", s);
                        let py_series = rust_series_to_py_series(&s).unwrap();
                        series.push(py_series);
                    }
                    _ => {}
                }
                let v = &parser.state.output[&AMMO_ID];
                match &v.data {
                    VarVec::I32(i) => {
                        let s = Series::new("ammo", i);
                        println!("{:?}", s);
                        let py_series = rust_series_to_py_series(&s).unwrap();
                        series.push(py_series);
                    }
                    _ => {}
                }

                let polars = py.import("polars")?;
                let all_series_py = series.to_object(py);
                let df = polars.call_method1("DataFrame", (all_series_py,))?;
                //df.setattr("columns", wanted_props.to_object(py)).unwrap();
                let pandas_df = df.call_method0("to_pandas").unwrap();
                Ok(pandas_df.to_object(py))
            }
        }
    }

    #[args(py_kwargs = "**")]
    pub fn parse_players(&self, py: Python) -> PyResult<PyObject> {
        let mut parser = Parser::new(
            self.path.clone(),
            true,
            false,
            //vec![],
            (1000..1001).collect(),
            vec![],
            vec!["player@m_vecOrigin_X".to_string()],
            "-".to_string(),
            false,
            false,
            false,
            1000000,
            vec![],
        );
        match parser {
            Err(e) => Err(PyFileNotFoundError::new_err(format!(
                "Couldnt read demo file. Error: {}",
                e
            ))),
            Ok(mut parser) => {
                let h: Header = parser.parse_demo_header();
                parser.start_parsing();

                let names: Vec<String> = parser
                    .maps
                    .players
                    .iter()
                    .map(|x| x.1.name.clone())
                    .collect_vec();
                let steamids: Vec<u64> = parser.maps.players.iter().map(|x| x.1.xuid).collect_vec();
                let name_series = rust_series_to_py_series(&Series::new("name", names)).unwrap();
                let steamids = rust_series_to_py_series(&Series::new("steamid", steamids)).unwrap();
                let all_series = vec![name_series, steamids];
                let column_names = vec!["name", "steamid"];

                let polars = py.import("polars").unwrap();
                let df = polars.call_method1("DataFrame", (all_series,)).unwrap();
                df.setattr("columns", column_names.to_object(py)).unwrap();
                let pandas_df = df.call_method0("to_pandas").unwrap();
                Ok(pandas_df.to_object(py))
            }
        }
    }

    pub fn parse_header(&self) -> PyResult<Py<PyAny>> {
        let parser = Parser::new(
            self.path.clone(),
            false,
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
/*
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
*/
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

#[pymodule]
fn demoparser(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DemoParser>()?;
    Ok(())
}
