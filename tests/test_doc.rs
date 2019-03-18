#[cfg(feature = "test-doc")]
use {
    docmatic,
    std::default::Default,
    std::path::{Path, PathBuf},
};

#[cfg(feature = "test-doc")]
fn assert_file<P: AsRef<Path>>(path: P) {
    let mut doc = docmatic::Assert::default();
    if cfg!(windows) {
        doc.library_path(
            option_env!("PYTHON")
                .map(|py| PathBuf::from(py).join("libs"))
                .unwrap(),
        );
    }
    doc.test_file(path.as_ref())
}

#[test]
#[cfg(feature = "test-doc")]
fn test_guide() {
    let guide_path = PathBuf::from("guide").join("src");
    for entry in guide_path.read_dir().unwrap() {
        assert_file(entry.unwrap().path())
    }
}

#[test]
#[cfg(feature = "test-doc")]
fn test_readme() {
    assert_file("README.md")
}
