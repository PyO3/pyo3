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

    /// The two variants overlap: `String` accepts `str` and `PathBuf` accepts
    /// `str | os.PathLike[str]`, so the union the derive builds repeats `str`.
    #[derive(FromPyObject)]
    enum NameOrPath {
        Name(String),
        Path(PathBuf),
    }

    #[pyfunction]
    fn take_name_or_path(value: NameOrPath) -> PathBuf {
        match value {
            NameOrPath::Name(name) => PathBuf::from(name),
            NameOrPath::Path(path) => path,
        }
    }
}
