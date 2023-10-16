// This is a very fake example of how to check __getitem__ parameter and handle appropriately
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PySlice;
use std::os::raw::c_long;

#[derive(FromPyObject)]
enum IntOrSlice<'py> {
    Int(i32),
    Slice(&'py PySlice),
}

#[pyclass]
struct ExampleContainer {
    // represent the maximum length our container is pretending to be
    max_length: i32,
}

#[pymethods]
impl ExampleContainer {
    #[new]
    fn new() -> Self {
        ExampleContainer { max_length: 100 }
    }

    fn __getitem__(&self, key: &PyAny) -> PyResult<i32> {
        if let Ok(position) = key.extract::<i32>() {
            return Ok(position);
        } else if let Ok(slice) = key.downcast::<PySlice>() {
            // METHOD 1 - the use PySliceIndices to help with bounds checking and for cases when only start or end are provided
            // in this case the start/stop/step all filled in to give valid values based on the max_length given
            let index = slice.indices(self.max_length as c_long).unwrap();
            let _delta = index.stop - index.start;

            // METHOD 2 - Do the getattr manually really only needed if you have some special cases for stop/_step not being present
            // convert to indices and this will help you deal with stop being the max length
            let start: i32 = slice.getattr("start")?.extract()?;
            // This particular example assumes stop is present, but note that if not present, this will cause us to return due to the
            // extract failing. Not needing custom code to deal with this is a good reason to use the Indices method.
            let stop: i32 = slice.getattr("stop")?.extract()?;
            // example of grabbing step since it is not always present
            let _step: i32 = match slice.getattr("step")?.extract() {
                // if no value found assume step is 1
                Ok(v) => v,
                Err(_) => 1 as i32,
            };

            // Use something like this if you don't support negative stepping and want to give users
            // leeway on how they provide their ordering
            let (start, stop) = if start > stop {
                (stop, start)
            } else {
                (start, stop)
            };
            let delta = stop - start;

            return Ok(delta);
        } else {
            return Err(PyTypeError::new_err("Unsupported type"));
        }
    }
    fn __setitem__(&self, idx: IntOrSlice, value: u32) -> PyResult<()> {
        match idx {
            IntOrSlice::Slice(slice) => {
                let index = slice.indices(self.max_length as c_long).unwrap();
                println!("Got a slice! {}-{}, step: {}, value: {}", index.start, index.stop, index.step, value);
            }
            IntOrSlice::Int(index) => {
                println!("Got an index! {} : value: {}", index, value);
            }
        }
        Ok(())
    }
}

#[pymodule]
#[pyo3(name = "getitem")]
fn example(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // ? -https://github.com/PyO3/maturin/issues/475
    m.add_class::<ExampleContainer>()?;
    Ok(())
}
