use pyo3::prelude::*;

#[pymodule]
pub mod path {
    use pyo3::prelude::*;
    use std::path::{Path, PathBuf};

    #[pyfunction]
    fn make_path() -> PathBuf {
        Path::new("/root").to_owned()
    }

    #[pyfunction]
    fn take_pathbuf(path: PathBuf) -> PathBuf {
        path
    }
}
