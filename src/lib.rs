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
use arrow::ffi;
use polars::prelude::*;
use polars_arrow::export::arrow;
use pyo3::exceptions::PyFileNotFoundError;
use pyo3::exceptions::PyValueError;
use pyo3::ffi::Py_uintptr_t;
use pyo3::prelude::*;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use pyo3::{PyAny, PyObject, PyResult};
use pyo3::{PyErr, Python};
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::time::Instant;
use std::{io, result, vec};

/// Arrow array to Python.
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
    wanted_ticks: Vec<i32>,
    wanted_players: Vec<u64>,
) -> PyResult<Vec<PyObject>> {
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

            wanted_props.push("tick".to_string());
            wanted_props.push("steamid".to_string());
            wanted_props.push("name".to_string());
            let mut all_series = vec![];
            //println!("{:?}", data);

            for prop_name in &wanted_props {
                if data.contains_key(prop_name) {
                    if let parsing::parser::VarVec::F32(data) = &data[prop_name].data {
                        let s = Series::new(prop_name, data);
                        let py_series = rust_series_to_py_series(&s).unwrap();
                        all_series.push(py_series);
                    }
                    if let parsing::parser::VarVec::String(data) = &data[prop_name].data {
                        let s = Series::new(prop_name, data);
                        let py_series = rust_series_to_py_series(&s).unwrap();
                        all_series.push(py_series);
                    }
                }
            }

            Ok(all_series)
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
