use numpy::ndarray::{Array1, ArrayD, ArrayView1, ArrayViewD, ArrayViewMutD, Zip};
use numpy::{
    datetime::{units, Timedelta},
    Complex64, IntoPyArray, PyArray1, PyArrayDyn, PyReadonlyArray1, PyReadonlyArrayDyn,
    PyReadwriteArray1, PyReadwriteArrayDyn,
};
mod parsing;
use fxhash::FxHashMap;
use hashbrown::{HashMap, HashSet};
use parsing::header::Header;
use parsing::parser::Demo;
//use polars::prelude::*;
//use polars::series::Series;
use crate::parsing::stringtables::UserInfo;
use pyo3::exceptions::PyFileNotFoundError;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use pyo3::{PyErr, Python};
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::time::Instant;
use std::{io, result, vec};

#[pyfunction]
pub fn parse_events(
    py: Python<'_>,
    demo_path: String,
    event_name: String,
) -> PyResult<(Py<PyAny>)> {
    let now = Instant::now();

    let parser = Demo::new(
        demo_path,
        false,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        event_name,
        false,
        false,
    );
    match parser {
        Err(e) => Err(PyFileNotFoundError::new_err("FILE NOT FOUND")),
        Ok(mut parser) => {
            let h: Header = parser.parse_demo_header();
            let data = parser.parse_frame(&vec!["".to_owned()]);
            let mut cnt = 0;
            let mut game_evs: Vec<FxHashMap<String, Vec<PyObject>>> = Vec::new();

            // Create Hashmap with <string, pyobject> to be able to convert to python dict
            for ge in parser.game_events {
                let mut hm: FxHashMap<String, Vec<PyObject>> = FxHashMap::default();
                let tuples = ge.to_py_tuples(py);
                for (k, v) in tuples {
                    hm.entry(k).or_insert_with(Vec::new).push(v);
                }
                game_evs.push(hm);
            }

            let dict = pyo3::Python::with_gil(|py| game_evs.to_object(py));
            Ok(dict)
        }
    }
}

#[pyfunction]
pub fn parse_props(
    demo_path: String,
    mut wanted_props: Vec<String>,
    mut out_arr: PyReadwriteArrayDyn<f64>,
    wanted_ticks: Vec<i32>,
    wanted_players: Vec<u64>,
) -> PyResult<Vec<u64>> {
    let mut out_arr = out_arr.as_array_mut();
    let mut parser = Demo::new(
        demo_path,
        true,
        wanted_ticks,
        wanted_players,
        wanted_props.clone(),
        "".to_string(),
        false,
        false,
    );
    match parser {
        Err(e) => Err(PyFileNotFoundError::new_err("Demo file not found!")),
        Ok(mut parser) => {
            let _: Header = parser.parse_demo_header();
            let data = parser.parse_frame(&wanted_props);
            let mut cnt = 0;
            let mut col_len = 1;

            wanted_props.push("tick".to_string());
            wanted_props.push("ent_id".to_string());

            /*
            let mut all_series: Vec<Series> = Vec::new();


            for prop_name in &props_names {
                if data.contains_key(prop_name) {
                    let s = Series::new(prop_name, &data[prop_name]);
                    all_series.push(s);
                }
            }

            let df = DataFrame::new(all_series).unwrap();
            println!("{:?}", df);
            */
            for prop_name in &wanted_props {
                if data.contains_key(prop_name) {
                    let v = &data[prop_name];
                    col_len = v.len();

                    for prop in v {
                        out_arr[cnt] = *prop as f64;
                        cnt += 1
                    }
                }
            }

            let mut result: Vec<u64> = vec![
                cnt.try_into().unwrap(),
                col_len.try_into().unwrap(),
                wanted_props.len().try_into().unwrap(),
            ];

            for player in parser.players {
                result.push(player.xuid);
                result.push(player.entity_id as u64);
            }

            Ok(result)
        }
    }
}

#[pyfunction]
pub fn parse_players(py: Python<'_>, demo_path: String) -> PyResult<(Py<PyAny>)> {
    let mut parser = Demo::new(
        demo_path,
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
            let data = parser.parse_frame(&vec![]);
            let players = parser.players;
            let mut py_players = vec![];
            for player in players {
                if player.xuid > 76500000000000000 && player.xuid < 76600000000000000 {
                    py_players.push(player.to_py_hashmap(py));
                }
            }
            //let py_players = players[0].to_py_hashmap(py)
            let dict = pyo3::Python::with_gil(|py| py_players.to_object(py));
            Ok(dict)
        }
    }
}

#[pyfunction]
pub fn parse_header(py: Python<'_>, demo_path: String) -> PyResult<(Py<PyAny>)> {
    let mut parser = Demo::new(
        demo_path,
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
            let dict = h.to_py_hashmap(py);
            Ok(dict)
        }
    }
}

#[pymodule]
fn demoparser(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_events, m)?)?;
    m.add_function(wrap_pyfunction!(parse_props, m)?)?;
    m.add_function(wrap_pyfunction!(parse_players, m)?)?;
    m.add_function(wrap_pyfunction!(parse_header, m)?)?;
    //parse(py, demo_name, props_names, out_arr);
    return Ok(());
}
