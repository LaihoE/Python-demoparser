use numpy::ndarray::{Array1, ArrayD, ArrayView1, ArrayViewD, ArrayViewMutD, Zip};
use numpy::{
    datetime::{units, Timedelta},
    Complex64, IntoPyArray, PyArray1, PyArrayDyn, PyReadonlyArray1, PyReadonlyArrayDyn,
    PyReadwriteArray1, PyReadwriteArrayDyn,
};
mod parsing;
use parsing::header::Header;
use parsing::parser::Demo;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use std::collections::HashMap;
use std::convert::TryInto;
use std::time::Instant;

#[pyfunction]
pub fn parse_events(
    py: Python<'_>,
    demo_name: String,
    event_name: String,
    //mut out_arr: ArrayViewMutD<'_, f64>,
) -> PyResult<(Py<PyAny>)> {
    let now = Instant::now();
    let mut d = Demo {
        bytes: std::fs::read(demo_name).unwrap(),
        fp: 0,
        cmd: 0,
        tick: 0,
        event_list: None,
        event_vec: None,
        dt_map: Some(HashMap::new()),
        class_bits: 0,
        serverclass_map: HashMap::new(),
        entities: Some(HashMap::new()),
        bad: Vec::new(),
        stringtables: Vec::new(),
        players: Vec::new(),
        parse_props: false,
        game_events: Vec::new(),
        event_name: event_name,
    };
    let props_names = vec!["".to_owned()];
    let h: Header = d.parse_header();
    let data = d.parse_frame(&props_names);
    let mut cnt = 0;
    let mut game_evs: Vec<HashMap<String, Vec<PyObject>>> = Vec::new();

    for ge in d.game_events {
        let mut hm: HashMap<String, Vec<PyObject>> = HashMap::new();
        let tuples = ge.to_py_tuples(py);
        for (k, v) in tuples {
            hm.entry(k).or_insert_with(Vec::new).push(v);
        }

        game_evs.push(hm);
    }

    let dict = pyo3::Python::with_gil(|py| game_evs.to_object(py));
    Ok(dict)
}

#[pyfunction]
pub fn parse_props(
    demo_name: String,
    mut props_names: Vec<String>,
    mut out_arr: PyReadwriteArrayDyn<f64>,
) -> PyResult<Vec<i32>> {
    let mut out_arr = out_arr.as_array_mut();
    let mut d = Demo {
        bytes: std::fs::read(demo_name).unwrap(),
        fp: 0,
        cmd: 0,
        tick: 0,
        event_list: None,
        event_vec: None,
        dt_map: Some(HashMap::new()),
        class_bits: 0,
        serverclass_map: HashMap::new(),
        entities: Some(HashMap::new()),
        bad: Vec::new(),
        stringtables: Vec::new(),
        players: Vec::new(),
        parse_props: true,
        game_events: Vec::new(),
        event_name: "".to_string(),
    };

    let h: Header = d.parse_header();
    let mut event_names: Vec<String> = Vec::new();

    let data = d.parse_frame(&props_names);
    let mut cnt = 0;
    let mut col_len = 1;

    for prop_name in &props_names {
        let v = &data[prop_name];
        col_len = v.len();
        println!("{}", col_len);
        for prop in v {
            out_arr[cnt] = *prop as f64;
            cnt += 1
        }
    }

    let mut result: Vec<i32> = vec![
        cnt.try_into().unwrap(),
        col_len.try_into().unwrap(),
        props_names.len().try_into().unwrap(),
    ];

    for player in d.players {
        println!("{} {}", player.name, player.entity_id)
    }
    Ok(result)
}

#[pymodule]
fn demoparser(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_events, m)?)?;
    m.add_function(wrap_pyfunction!(parse_props, m)?)?;
    //parse(py, demo_name, props_names, out_arr);
    return Ok(());
}
