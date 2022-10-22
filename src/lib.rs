mod parsing;
use arrow::ffi;
use flate2::read::GzDecoder;
use fxhash::FxHashMap;
use memmap::MmapOptions;
use parsing::entities::Entity;
use parsing::header::Header;
use parsing::parser::Demo;
use parsing::stringtables::UserInfo;
use parsing::variants::PropAtom;
use parsing::variants::PropData;
use polars::prelude::ArrowField;
use polars::prelude::NamedFrom;
use polars::series::Series;
use polars_arrow::export::arrow;
use polars_arrow::prelude::ArrayRef;
use pyo3::exceptions::PyFileNotFoundError;
use pyo3::ffi::Py_uintptr_t;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyDict;
use pyo3::Python;
use pyo3::{PyAny, PyObject, PyResult};
use std::collections::HashMap;
use std::fs::File;
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

pub fn parse_kwargs(kwargs: Option<&PyDict>) -> (Vec<u64>, Vec<i32>) {
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
            return (players, ticks);
        }
        None => (vec![], vec![]),
    }
}

pub fn rm_user_friendly_names(names: Vec<String>) -> Vec<String> {
    let mut unfriendly_names = vec![];
    for name in names {
        match &name[..] {
            "X" => unfriendly_names.push("m_vecOrigin_X".to_string()),
            "Y" => unfriendly_names.push("m_vecOrigin_Y".to_string()),
            "Z" => unfriendly_names.push("m_vecOrigin[2]".to_string()),

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

            _ => unfriendly_names.push(name),
        }
    }
    unfriendly_names
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

    pub fn parse_events(&self, py: Python<'_>, event_name: String) -> PyResult<Py<PyAny>> {
        let file = File::open(self.path.clone()).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        let parser = Demo::new_mmap(
            mmap,
            false,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            event_name,
            false,
            false,
        );
        match parser {
            Err(e) => Err(PyFileNotFoundError::new_err("ERROR READING FILE")),
            Ok(mut parser) => {
                let _: Header = parser.parse_demo_header();

                let _ = parser.start_parsing(&vec!["".to_owned()]);
                let mut game_evs: Vec<FxHashMap<String, PyObject>> = Vec::new();

                // Create Hashmap with <string, pyobject> to be able to convert to python dict
                for ge in parser.game_events {
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

    #[args(py_kwargs = "**")]
    pub fn parse_ticks(
        &self,
        py: Python,
        mut wanted_props: Vec<String>,
        py_kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        let mut real_props = rm_user_friendly_names(wanted_props);

        let file = File::open(self.path.clone()).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        let (wanted_players, wanted_ticks) = parse_kwargs(py_kwargs);
        let parser = Demo::new_mmap(
            mmap,
            true,
            wanted_ticks,
            wanted_players,
            real_props.clone(),
            "".to_string(),
            false,
            false,
        );

        match parser {
            Err(e) => Err(PyFileNotFoundError::new_err("Demo file not found!")),
            Ok(mut parser) => {
                let h: Header = parser.parse_demo_header();
                parser.playback_frames = h.playback_frames as usize;

                let data = parser.start_parsing(&real_props);

                real_props.push("tick".to_string());
                real_props.push("steamid".to_string());
                real_props.push("name".to_string());
                let mut all_series = vec![];

                match data.get("tick") {
                    Some(d) => {
                        let df_len = d.data.get_len();
                        for prop_name in &real_props {
                            if data.contains_key(prop_name) {
                                if let parsing::parser::VarVec::F32(data) = &data[prop_name].data {
                                    let s = Series::new(prop_name, data);
                                    let py_series = rust_series_to_py_series(&s).unwrap();
                                    all_series.push(py_series);
                                }
                                if let parsing::parser::VarVec::String(data) = &data[prop_name].data
                                {
                                    let s = Series::new(prop_name, data);
                                    let py_series = rust_series_to_py_series(&s).unwrap();
                                    all_series.push(py_series);
                                }
                                if let parsing::parser::VarVec::I32(data) = &data[prop_name].data {
                                    let s = Series::new(prop_name, data);
                                    let py_series = rust_series_to_py_series(&s).unwrap();
                                    all_series.push(py_series);
                                }
                                if let parsing::parser::VarVec::U64(data) = &data[prop_name].data {
                                    let s = Series::new(prop_name, data);
                                    let py_series = rust_series_to_py_series(&s).unwrap();
                                    all_series.push(py_series);
                                }
                            } else {
                                let mut empty_col: Vec<Option<i32>> = vec![];
                                for _ in 0..df_len {
                                    empty_col.push(None);
                                }
                                let s = Series::new(prop_name, empty_col);
                                let py_series = rust_series_to_py_series(&s).unwrap();
                                all_series.push(py_series);
                            }
                        }
                        let polars = py.import("polars")?;
                        let all_series_py = all_series.to_object(py);
                        let df = polars.call_method1("DataFrame", (all_series_py,))?;
                        df.setattr("columns", real_props.to_object(py)).unwrap();
                        let pandas_df = df.call_method0("to_pandas").unwrap();
                        Ok(pandas_df.to_object(py))
                    }
                    None => {
                        return {
                            let pandas = py.import("pandas")?;
                            let mut dict = HashMap::new();
                            dict.insert("columns", real_props.to_object(py));
                            let py_dict = dict.into_py_dict(py);
                            let df = pandas.call_method("DataFrame", (), Some(py_dict)).unwrap();
                            Ok(df.to_object(py))
                        };
                    }
                }
            }
        }
    }

    pub fn parse_players(&self, py: Python<'_>) -> PyResult<(Py<PyAny>)> {
        let file = File::open(self.path.clone()).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        let parser = Demo::new_mmap(
            mmap,
            true,
            vec![],
            vec![],
            vec![],
            "".to_string(),
            true,
            false,
        );
        match parser {
            Err(e) => Err(PyFileNotFoundError::new_err("Demo file not found!")),
            Ok(mut parser) => {
                let _: Header = parser.parse_demo_header();
                let _ = parser.start_parsing(&vec![]);
                let players = parser.players;
                let mut py_players = vec![];
                let ent_manager = &parser.entities[parser.manager_id.unwrap() as usize].1;
                for (_, player) in players {
                    let team = get_manager_i32_prop(&ent_manager, &player, "m_iTeam");
                    let mut hm = player.to_hashmap(py);
                    let team = match team {
                        2 => "CT",
                        3 => "T",
                        _ => "Missing",
                    };

                    let rank_id =
                        get_manager_i32_prop(&ent_manager, &player, "m_iCompetitiveRanking");
                    let rank_name = rank_id_to_name(rank_id);

                    let crosshair_code =
                        get_manager_str_prop(&ent_manager, &player, "m_szCrosshairCodes");

                    let comp_wins =
                        get_manager_i32_prop(&ent_manager, &player, "m_iCompetitiveWins");

                    hm.insert("staring_side".to_string(), team.to_string().to_object(py));
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

    pub fn parse_header(&self) -> PyResult<(Py<PyAny>)> {
        let file = File::open(self.path.clone()).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        let parser = Demo::new_mmap(
            mmap,
            false,
            vec![],
            vec![],
            vec![],
            "".to_string(),
            true,
            false,
        );
        match parser {
            Err(e) => Err(PyFileNotFoundError::new_err("Demo file not found!")),
            Ok(mut parser) => {
                let h: Header = parser.parse_demo_header();
                let dict = h.to_py_hashmap();
                Ok(dict)
            }
        }
    }
}

pub fn get_manager_i32_prop(manager: &Entity, player: &UserInfo, prop_name: &str) -> i32 {
    let key = if player.entity_id < 10 {
        prop_name.to_string() + "00" + &player.entity_id.to_string()
    } else if player.entity_id < 100 {
        prop_name.to_string() + "0" + &player.entity_id.to_string()
    } else {
        panic!("Entity id 100 ????: id:{}", player.entity_id);
    };
    match manager.props.get(&key) {
        Some(p) => match p.data {
            PropData::I32(x) => {
                return x;
            }
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
        panic!("Entity id 100 ????: id:{}", player.entity_id);
    };
    match manager.props.get(&key) {
        Some(p) => match &p.data {
            PropData::String(x) => {
                return x.to_string();
            }
            _ => "".to_string(),
        },
        None => "".to_string(),
    }
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

#[pymodule]
fn demoparser(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DemoParser>()?;
    return Ok(());
}
