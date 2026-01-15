use pyo3::prelude::*;

#[pymodule]
pub mod sequence {
    use pyo3::prelude::*;
    use pyo3::types::PyString;

    #[pyfunction]
    fn vec_to_vec_i32(vec: Vec<i32>) -> Vec<i32> {
        vec
    }

    #[pyfunction]
    fn array_to_array_i32(arr: [i32; 3]) -> [i32; 3] {
        arr
    }

    #[pyfunction]
    fn vec_to_vec_pystring(vec: Vec<Bound<'_, PyString>>) -> Vec<Bound<'_, PyString>> {
        vec
    }
}
