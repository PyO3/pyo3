/// Represents the major, minor, and patch (if any) versions of this interpreter.
///
/// This struct is usually created with [`Python::version`].
///
/// # Examples
///
/// ```rust
/// # use pyo3::Python;
/// Python::attach(|py| {
///     // PyO3 supports Python 3.7 and up.
///     assert!(py.version_info() >= (3, 7));
///     assert!(py.version_info() >= (3, 7, 0));
/// });
/// ```
///
/// [`Python::version`]: crate::marker::Python::version
#[derive(Debug)]
pub struct PythonVersionInfo<'a> {
    /// Python major version (e.g. `3`).
    pub major: u8,
    /// Python minor version (e.g. `11`).
    pub minor: u8,
    /// Python patch version (e.g. `0`).
    pub patch: u8,
    /// Python version suffix, if applicable (e.g. `a0`).
    pub suffix: Option<&'a str>,
}

impl<'a> PythonVersionInfo<'a> {
    /// Parses a hard-coded Python interpreter version string (e.g. 3.9.0a4+).
    pub(crate) fn from_str(version_number_str: &'a str) -> Result<PythonVersionInfo<'a>, &'a str> {
        fn split_and_parse_number(version_part: &str) -> (u8, Option<&str>) {
            match version_part.find(|c: char| !c.is_ascii_digit()) {
                None => (version_part.parse().unwrap(), None),
                Some(version_part_suffix_start) => {
                    let (version_part, version_part_suffix) =
                        version_part.split_at(version_part_suffix_start);
                    (version_part.parse().unwrap(), Some(version_part_suffix))
                }
            }
        }

        let mut parts = version_number_str.splitn(3, '.');
        let major_str = parts.next().ok_or("Python major version missing")?;
        let minor_str = parts.next().ok_or("Python minor version missing")?;
        let patch_str = parts.next();

        let major = major_str
            .parse()
            .map_err(|_| "Python major version not an integer")?;
        let (minor, suffix) = split_and_parse_number(minor_str);
        if suffix.is_some() {
            assert!(patch_str.is_none());
            return Ok(PythonVersionInfo {
                major,
                minor,
                patch: 0,
                suffix,
            });
        }

        let (patch, suffix) = patch_str.map(split_and_parse_number).unwrap_or_default();
        Ok(PythonVersionInfo {
            major,
            minor,
            patch,
            suffix,
        })
    }
}

impl PartialEq<(u8, u8)> for PythonVersionInfo<'_> {
    fn eq(&self, other: &(u8, u8)) -> bool {
        self.major == other.0 && self.minor == other.1
    }
}

impl PartialEq<(u8, u8, u8)> for PythonVersionInfo<'_> {
    fn eq(&self, other: &(u8, u8, u8)) -> bool {
        self.major == other.0 && self.minor == other.1 && self.patch == other.2
    }
}

impl PartialOrd<(u8, u8)> for PythonVersionInfo<'_> {
    fn partial_cmp(&self, other: &(u8, u8)) -> Option<std::cmp::Ordering> {
        (self.major, self.minor).partial_cmp(other)
    }
}

impl PartialOrd<(u8, u8, u8)> for PythonVersionInfo<'_> {
    fn partial_cmp(&self, other: &(u8, u8, u8)) -> Option<std::cmp::Ordering> {
        (self.major, self.minor, self.patch).partial_cmp(other)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Python;
    #[test]
    fn test_python_version_info() {
        Python::attach(|py| {
            let version = py.version_info();
            #[cfg(Py_3_7)]
            assert!(version >= (3, 7));
            #[cfg(Py_3_7)]
            assert!(version >= (3, 7, 0));
            #[cfg(Py_3_8)]
            assert!(version >= (3, 8));
            #[cfg(Py_3_8)]
            assert!(version >= (3, 8, 0));
            #[cfg(Py_3_9)]
            assert!(version >= (3, 9));
            #[cfg(Py_3_9)]
            assert!(version >= (3, 9, 0));
            #[cfg(Py_3_10)]
            assert!(version >= (3, 10));
            #[cfg(Py_3_10)]
            assert!(version >= (3, 10, 0));
            #[cfg(Py_3_11)]
            assert!(version >= (3, 11));
            #[cfg(Py_3_11)]
            assert!(version >= (3, 11, 0));
        });
    }

    #[test]
    fn test_python_version_info_parse() {
        assert!(PythonVersionInfo::from_str("3.5.0a1").unwrap() >= (3, 5, 0));
        assert!(PythonVersionInfo::from_str("3.5+").unwrap() >= (3, 5, 0));
        assert!(PythonVersionInfo::from_str("3.5+").unwrap() == (3, 5, 0));
        assert!(PythonVersionInfo::from_str("3.5+").unwrap() != (3, 5, 1));
        assert!(PythonVersionInfo::from_str("3.5.2a1+").unwrap() < (3, 5, 3));
        assert!(PythonVersionInfo::from_str("3.5.2a1+").unwrap() == (3, 5, 2));
        assert!(PythonVersionInfo::from_str("3.5.2a1+").unwrap() == (3, 5));
        assert!(PythonVersionInfo::from_str("3.5+").unwrap() == (3, 5));
        assert!(PythonVersionInfo::from_str("3.5.2a1+").unwrap() < (3, 6));
        assert!(PythonVersionInfo::from_str("3.5.2a1+").unwrap() > (3, 4));
        assert!(PythonVersionInfo::from_str("3.11.3+chromium.29").unwrap() >= (3, 11, 3));
        assert_eq!(
            PythonVersionInfo::from_str("3.11.3+chromium.29")
                .unwrap()
                .suffix,
            Some("+chromium.29")
        );
    }
}
