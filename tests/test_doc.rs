extern crate docmatic;

use std::path::{Path, PathBuf};
use docmatic::assert_file_with;

fn pypath() -> Option<PathBuf> {
    option_env!("PYTHON").map(|py| PathBuf::from(py).join("libs"))
}


fn test_file<P: AsRef<Path>>(path: P) {
    let args = if cfg!(windows) {
        vec![("--library-path", pypath().unwrap())]
    } else {
        Vec::new()
    };
    docmatic::assert_file_with(path, &args);
}



#[test]
fn test_guide() {
    let mut guide_path = PathBuf::from("guide").join("src");
    for entry in guide_path.read_dir().unwrap() {
        test_file(entry.unwrap().path())
    }
}

#[test]
fn test_readme() {
    test_file(PathBuf::from("README.md"));
}
