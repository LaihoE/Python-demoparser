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
use std::collections::HashMap;
use std::convert::TryInto;
use std::time::Instant;

#[pyfunction]
pub fn parse_props(
    py: Python<'_>,
    demo_name: String,
    props_names: Vec<String>,
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
        parse_props: true,
        game_events: Vec::new(),
    };

    let h: Header = d.parse_header();
    let data = d.parse_frame(&props_names);
    let mut cnt = 0;
    /*
    for prop_name in &props_names {
        let v = &data[prop_name];
        for prop in v {
            out_arr[cnt] = *prop as f64;
            cnt += 1
        }
    }
    */
    //println!("{:?}", &d.game_events);
    let ge = &d.game_events[5];
    let k = &ge.name;
    let v = &ge.fields[0].data;

    let mut key_vals: Vec<(String, PyObject)> = Vec::new();
    key_vals.push(("k".to_string(), v.to_string_py().to_object(py)));
    let dict = key_vals.into_py_dict(py);

    Ok(pyo3::Python::with_gil(|py| dict.to_object(py)))
}

pub fn get_game_events(demo_name: String, event_name: String) -> PyResult<Vec<f32>> {
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
    };

    let h: Header = d.parse_header();

    let mut event_names: Vec<String> = Vec::new();
    event_names.push("lol".to_string());
    let data = d.parse_frame(&event_names);

    let mut result = vec![];
    Ok(result)
}

#[pymodule]
fn demoparser(_py: Python, m: &PyModule) -> PyResult<()> {
    //let out_arr = out_arr.as_array_mut();
    m.add_function(wrap_pyfunction!(parse_props, m)?)?;
    //parse(py, demo_name, props_names, out_arr);
    return Ok(());
}
