mod parsing;
use arrow::ffi;
use flate2::read::GzDecoder;
use fxhash::FxHashMap;
use memmap::MmapOptions;
use parsing::header::Header;
use parsing::parser::Demo;
use polars::prelude::ArrowField;
use polars::prelude::NamedFrom;
use polars::series::Series;
use polars_arrow::export::arrow;
use polars_arrow::prelude::ArrayRef;
use pyo3::exceptions::PyFileNotFoundError;
use pyo3::ffi::Py_uintptr_t;
use pyo3::prelude::*;
use pyo3::Python;
use pyo3::{PyAny, PyObject, PyResult};
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

    pub fn parse_props(
        &self,
        py: Python,
        mut wanted_props: Vec<String>,
        wanted_ticks: Vec<i32>,
        wanted_players: Vec<u64>,
    ) -> PyResult<PyObject> {
        let bytes = read_file(self.path.clone()).unwrap();
        let parser = Demo::new(
            bytes,
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
                let h: Header = parser.parse_demo_header();
                parser.playback_frames = h.playback_frames as usize;

                let data = parser.start_parsing(&wanted_props);
                wanted_props.push("tick".to_string());
                wanted_props.push("steamid".to_string());
                wanted_props.push("name".to_string());
                let mut all_series = vec![];
                // println!("{:?}", wanted_props);
                // println!("{:?}", data.keys());
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
                        println!(
                            "{:?} Column not found. Maybe your prop is incorrect?",
                            prop_name
                        );
                    }
                }
                //println!("{:?}", data.get("steamid"));
                let polars = py.import("polars")?;
                let all_series_py = all_series.to_object(py);
                let df = polars.call_method1("DataFrame", (all_series_py,))?;
                df.setattr("columns", wanted_props.to_object(py)).unwrap();
                let pandas_df = df.call_method0("to_pandas").unwrap();
                pandas_df.call_method1("replace", (-1, polars)).unwrap();
                Ok(pandas_df.to_object(py))
            }
        }
    }

    pub fn parse_players(&self, py: Python<'_>) -> PyResult<(Py<PyAny>)> {
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
                let _: Header = parser.parse_demo_header();
                let _ = parser.start_parsing(&vec![]);
                let players = parser.players;
                let mut py_players = vec![];
                for (_, player) in players {
                    if player.xuid > 76500000000000000 && player.xuid < 76600000000000000 {
                        py_players.push(player.to_py_hashmap(py));
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

#[pymodule]
fn demoparser(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DemoParser>()?;
    return Ok(());
}
