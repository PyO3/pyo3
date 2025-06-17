use std::fs::File;

/// This is a structure to deal with python file
pub struct Pyo3File {
    /// Our rust file
    pub file: File,
    /// For python file, do nothing in rust
    pub path: String,
    /// For python file, do nothing in rust
    pub name: String,
    /// For python file, do nothing in rust
    ///
    /// See the Python `open` function:
    /// <https://docs.python.org/3/library/functions.html#open>
    pub mode: String,
    /// For python file, do nothing in rust
    ///
    /// See
    /// <https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html>
    pub encoding: String,
}

impl Pyo3File {
    /// [Pyo3File]\[new\]: create a Pyo3File
    pub fn new(file: File, path: String, name: String, mode: String, encoding: String) -> Self {
        Self {
            file,
            path,
            name,
            mode,
            encoding,
        }
    }

    ///
    /// [Pyo3File]\[getfile\]
    ///
    pub fn getfile(&self) -> &File {
        &self.file
    }
}
