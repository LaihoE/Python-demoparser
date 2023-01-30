mod parsing;
use crate::parsing::game_events::GameEvent;
use crate::parsing::game_events::NameDataPair;
use ahash::RandomState;
use arrow::ffi;
use flate2::read::GzDecoder;
use fxhash::FxHashMap;
use parsing::entities::Entity;
use parsing::game_events::KeyData;
use parsing::header::Header;
use parsing::parser::Parser;
use parsing::stringtables::UserInfo;
//use parsing::tick_cache::gather_props_backwards;
use parsing::variants::PropData;
use parsing::variants::VarVec;
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
    "m_flNextAttack" => 1,
    "m_bDuckOverride" => 0,
    "m_flStamina" => 1,
    "m_flVelocityModifier" => 1,
    "m_iShotsFired" => 0,
    "m_nQuestProgressReason" => 0,
    "m_vecOrigin" => 2,
    "m_vecOrigin_X" => 1,
    "m_vecOrigin_Y" => 1,
    "m_vecOrigin[2]" => 1,
    "m_aimPunchAngle" => 2,
    "m_aimPunchAngle_X" => 1,
    "m_aimPunchAngle_Y" => 1,
    "m_aimPunchAngleVel" => 2,
    "m_aimPunchAngleVel_X" => 1,
    "m_aimPunchAngleVel_Y" => 1,
    "m_bDucked" => 0,
    "m_bDucking" => 0,
    "m_bWearingSuit" => 0,
    "m_chAreaBits.000" => 0,
    "m_chAreaBits.001" => 0,
    "m_chAreaPortalBits.002" => 0,
    "m_flFOVRate" => 1,
    "m_flFallVelocity" => 1,
    "m_flLastDuckTime" => 1,
    "m_viewPunchAngle" => 2,
    "m_viewPunchAngle_X" => 1,
    "m_viewPunchAngle_Y" => 1,
    "m_flDeathTime" => 1,
    "m_flNextDecalTime" => 1,
    "m_hLastWeapon" => 0,
    "m_hTonemapController" => 0,
    "m_nNextThinkTick" => 0,
    "m_nTickBase" => 0,
    "m_vecBaseVelocity" => 2,
    "m_vecBaseVelocity_X" => 1,
    "m_vecBaseVelocity_Y" => 1,
    "m_vecVelocity[0]" => 1,
    "m_vecVelocity[1]" => 1,
    "m_vecVelocity[2]" => 1,
    "m_vecViewOffset[2]" => 1,
    "m_ArmorValue" => 0,
    "m_usSolidFlags" => 0,
    "m_vecMaxs" => 2,
    "m_vecMaxs_X" => 1,
    "m_vecMaxs_Y" => 1,
    "m_vecMins" => 2,
    "m_vecMins_X" => 1,
    "m_vecMins_Y" => 1,
    "m_LastHitGroup" => 0,
    "m_afPhysicsFlags" => 0,
    "m_angEyeAngles[0]" => 1,
    "m_angEyeAngles[1]" => 1,
    "m_bAnimatedEveryTick" => 0,
    "m_bClientSideRagdoll" => 0,
    "m_bHasDefuser" => 0,
    "m_bHasHelmet" => 0,
    "m_bHasMovedSinceSpawn" => 0,
    "m_bInBombZone" => 0,
    "m_bInBuyZone" => 0,
    "m_bIsDefusing" => 0,
    "m_bIsHoldingLookAtWeapon" => 0,
    "m_bIsLookingAtWeapon" => 0,
    "m_bIsScoped" => 0,
    "m_bIsWalking" => 0,
    "m_bResumeZoom" => 0,
    "m_bSpotted" => 0,
    "m_bSpottedByMask.000" => 0,
    "m_bStrafing" => 0,
    "m_bWaitForNoAttack" => 0,
    "m_fEffects" => 0,
    "m_fFlags" => 0,
    "m_fMolotovDamageTime" => 1,
    "m_fMolotovUseTime" => 1,
    "m_flDuckAmount" => 1,
    "m_flDuckSpeed" => 1,
    "m_flFOVTime" => 1,
    "m_flFlashDuration" => 1,
    "m_flFlashMaxAlpha" => 1,
    "m_flGroundAccelLinearFracLastTime" => 1,
    "m_flLastMadeNoiseTime" => 1,
    "m_flLowerBodyYawTarget" => 1,
    "m_flProgressBarStartTime" => 1,
    "m_flSimulationTime" => 0,
    "m_flThirdpersonRecoil" => 1,
    "m_flTimeOfLastInjury" => 1,
    "m_hActiveWeapon" => -1,
    "m_hColorCorrectionCtrl" => 0,
    "m_hGroundEntity" => 0,
    "m_hObserverTarget" => 0,
    "m_hPlayerPing" => 0,
    "m_hPostProcessCtrl" => 0,
    "m_hRagdoll" => 0,
    "m_hViewModel" => 5,
    "m_hZoomOwner" => 0,
    "m_iAccount" => 0,
    "m_iAddonBits" => 0,
    "m_iAmmo.014" => 0,
    "m_iAmmo.015" => 0,
    "m_iAmmo.016" => 0,
    "m_iAmmo.017" => 0,
    "m_iAmmo.018" => 0,
    "m_iClass" => 0,
    "m_iDeathPostEffect" => 0,
    "m_iFOV" => 0,
    "m_iFOVStart" => 0,
    "m_iHealth" => 0,
    "m_iMoveState" => 0,
    "m_iNumRoundKills" => 0,
    "m_iNumRoundKillsHeadshots" => 0,
    "m_iObserverMode" => 0,
    "m_iPendingTeamNum" => 0,
    "m_iPlayerState" => 0,
    "m_iPrimaryAddon" => 0,
    "m_iProgressBarDuration" => 0,
    "m_iSecondaryAddon" => 0,
    "m_iStartAccount" => 0,
    "m_iTeamNum" => 0,
    "m_lifeState" => 0,
    "m_nForceBone" => 0,
    "m_nHeavyAssaultSuitCooldownRemaining" => 0,
    "m_nLastConcurrentKilled" => 0,
    "m_nLastKillerIndex" => 0,
    "m_nModelIndex" => 0,
    "m_nRelativeDirectionOfLastInjury" => 0,
    "m_nWaterLevel" => 0,
    "m_rank.005" => 0,
    "m_szLastPlaceName" => 4,
    "m_totalHitsOnServer" => 0,
    "m_ubEFNoInterpParity" => 0,
    "m_unCurrentEquipmentValue" => 0,
    "m_unFreezetimeEndEquipmentValue" => 0,
    "m_unMusicID" => 0,
    "m_unRoundStartEquipmentValue" => 0,
    "m_unTotalRoundDamageDealt" => 0,
    "m_vecForce" => 2,
    "m_vecForce_X" => 1,
    "m_vecForce_Y" => 1,
    "m_vecLadderNormal" => 2,
    "m_vecLadderNormal_X" => 1,
    "m_vecLadderNormal_Y" => 1,
    "m_vecPlayerPatchEconIndices.002" => 0,
    "movetype" => 0,
    "pl.deadflag" => 0,
    "m_bSilencerOn" => 0,
    "m_bReloadVisuallyComplete" => 1,
    "m_iCompetitiveRanking" => 10,
    "m_iPing" => 10,
    "m_iTeam" => 10,
    "m_iScore" => 10,
    "m_iDeaths" => 10,
    "m_iKills" => 10,
    "m_iAssists" => 10,
    "m_iMVPs" => 10,
    "m_iArmor" => 10,
    "m_iCompetitiveWins" => 10,
    "m_iMatchStats_UtilityDamage_Total" => 10,
    "m_iMatchStats_Damage_Total" => 10,
    "m_iLifetimeStart" => 10,
    "m_iLifetimeEnd" => 10,
    "m_bConnected" => 10,
    "m_bControllingBot" => 10,
    "m_iControlledPlayer"=> 10,
    "m_iControlledByPlayer"=> 10,
    "m_iTotalCashSpent"=> 10,
    "m_iCashSpentThisRound"=> 10,
    "m_nPersonaDataPublicCommendsLeader"=> 10,
    "m_nPersonaDataPublicCommendsTeacher"=> 10,
    "m_nPersonaDataPublicCommendsFriendly"=> 10,
    "m_bHasCommunicationAbuseMute"=> 10,
    "m_iMatchStats_Kills_Total"=> 10,
    "m_iMatchStats_5k_Total"=> 10,
    "m_iMatchStats_4k_Total"=> 10,
    "m_iMatchStats_3k_Total"=> 10,
    "m_iMatchStats_EquipmentValue_Total"=> 10,
    "m_iMatchStats_KillReward_Total"=> 10,
    "m_iMatchStats_LiveTime_Total"=> 10,
    "m_iMatchStats_Deaths_Total"=> 10,
    "m_iMatchStats_Assists_Total"=> 10,
    "m_iMatchStats_HeadShotKills_Total"=> 10,
    "m_iMatchStats_Objective_Total"=> 10,
    "m_iMatchStats_CashEarned_Total"=> 10,
    "m_iMatchStats_EnemiesFlashed_Total"=> 10,
    "m_bAlive" => 10,
    "m_bInDuckJump"=> 0,
    "m_nDuckJumpTimeMsecs"=> 0,
    "m_nJumpTimeMsecs"=> 0,
    "m_vecViewOffset[1]"=> 1,
    "m_vecViewOffset[0]"=> 1,
    "m_fOnTarget"=> 0,
    "m_flFriction" => 1,
    "m_flElasticity"=> 1,
    "m_iThrowGrenadeCounter" => 0,
    "m_bIsRescuing"=> 0,
    "m_flDetectedByEnemySensorTime"=> 1,
    "m_bHasHeavyArmor"=> 0,
    "m_nActiveCoinRank"=> 0,
    "m_nPersonaDataPublicLevel"=> 0,
    "m_iClip1" => 0,
    "weapon_name" => 99,
    "DT_CSLocalPlayerExclusive.m_vecOrigin[2]" => 0,
    "DT_BasePlayer.m_iHealth" => 0,
    "DT_BasePlayer.m_szLastPlaceName" => 0,
    "DT_LocalPlayerExclusive.m_vecViewOffset[2]" => 0,
};

#[pymodule]
fn demoparser(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DemoParser>()?;
    Ok(())
}
