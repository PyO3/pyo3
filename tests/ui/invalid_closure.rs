use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyTuple};

fn main() {
    let fun: Py<PyCFunction> = Python::with_gil(|py| {
        let local_data = vec![0, 1, 2, 3, 4];
        let ref_: &[u8] = &local_data;

        let closure_fn =
            |_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>| -> PyResult<()> {
                println!("This is five: {:?}", ref_.len());
                Ok(())
            };
        PyCFunction::new_closure(py, None, None, closure_fn)
            .unwrap()
            .into()
    });

    Python::with_gil(|py| {
        fun.call0(py).unwrap();
    });
}
