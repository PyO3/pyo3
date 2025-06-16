use std::fs::File;

pub struct Pyo3File {
    pub file: File,
    pub path: String,
    pub name: String,
    pub mode: String,
    pub encoding: String,
}

impl Pyo3File {
    pub fn new(file: File, path: String, name: String, mode: String, encoding: String) -> Self {
        Self {
            file,
            path,
            name,
            mode,
            encoding,
        }
    }

    pub fn getfile(&self) -> &File {
        &self.file
    }
}
