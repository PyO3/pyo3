use std::fs::File;

/// This is a structure to deal with python file
pub struct Pyo3File {
    /// Our rust file
    pub file: File,
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
    ///
    pub fn new(file: File, name: String, mode: String, encoding: String) -> Self {
        Self {
            file,
            name,
            mode,
            encoding,
        }
    }

    ///
    pub fn getfile(&self) -> &File {
        &self.file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_create_pyo3file() {
        let temp_file = NamedTempFile::new().expect("");
        let name: String = String::from("name");
        let mode: String = String::from("r");
        let encoding: String = String::from("utf-8");
        let pyo3_file: Pyo3File = Pyo3File::new(
            temp_file.into_file(),
            name.clone(),
            mode.clone(),
            encoding.clone());

        assert_eq!(pyo3_file.name, name.clone());
        assert_eq!(pyo3_file.mode, mode.clone());
        assert_eq!(pyo3_file.encoding, encoding.clone())
    }
}
