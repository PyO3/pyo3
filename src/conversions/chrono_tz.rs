#![cfg(all(Py_3_9, feature = "chrono-tz"))]

//! Conversions to and from [chrono-tz](https://docs.rs/chrono-tz/)’s `Tz`.
//!
//! This feature requires at least Python 3.9.
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! chrono-tz = "0.8"
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"chrono-tz\"] }")]
//! ```
//!
//! Note that you must use compatible versions of chrono, chrono-tz and PyO3.
//! The required chrono version may vary based on the version of PyO3.
//!
//! # Example: Convert a `zoneinfo.ZoneInfo` to chrono-tz's `Tz`
//!
//! ```rust,no_run
//! use chrono_tz::Tz;
//! use pyo3::{Python, PyResult, IntoPyObject, types::PyAnyMethods};
//!
//! fn main() -> PyResult<()> {
//!     pyo3::prepare_freethreaded_python();
//!     Python::with_gil(|py| {
//!         // Convert to Python
//!         let py_tzinfo = Tz::Europe__Paris.into_pyobject(py)?;
//!         // Convert back to Rust
//!         assert_eq!(py_tzinfo.extract::<Tz>()?, Tz::Europe__Paris);
//!         Ok(())
//!     })
//! }
//! ```
use crate::conversion::IntoPyObject;
use crate::exceptions::PyValueError;
use crate::pybacked::PyBackedStr;
use crate::sync::GILOnceCell;
use crate::types::{any::PyAnyMethods, PyType};
use crate::{intern, Bound, FromPyObject, Py, PyAny, PyErr, PyObject, PyResult, Python};
#[allow(deprecated)]
use crate::{IntoPy, ToPyObject};
use chrono_tz::Tz;
use std::str::FromStr;

#[allow(deprecated)]
impl ToPyObject for Tz {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().unbind()
    }
}

#[allow(deprecated)]
impl IntoPy<PyObject> for Tz {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().unbind()
    }
}

impl<'py> IntoPyObject<'py> for Tz {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        static ZONE_INFO: GILOnceCell<Py<PyType>> = GILOnceCell::new();
        ZONE_INFO
            .import(py, "zoneinfo", "ZoneInfo")
            .and_then(|obj| obj.call1((self.name(),)))
    }
}

impl<'py> IntoPyObject<'py> for &Tz {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl FromPyObject<'_> for Tz {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Tz> {
        Tz::from_str(
            &ob.getattr(intern!(ob.py(), "key"))?
                .extract::<PyBackedStr>()?,
        )
        .map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

#[cfg(all(test, not(windows)))] // Troubles loading timezones on Windows
mod tests {
    use super::*;
    use crate::prelude::PyAnyMethods;
    use crate::Python;
    use chrono::{DateTime, Utc};
    use chrono_tz::Tz;

    #[test]
    fn test_frompyobject() {
        Python::with_gil(|py| {
            assert_eq!(
                new_zoneinfo(py, "Europe/Paris").extract::<Tz>().unwrap(),
                Tz::Europe__Paris
            );
            assert_eq!(new_zoneinfo(py, "UTC").extract::<Tz>().unwrap(), Tz::UTC);
            assert_eq!(
                new_zoneinfo(py, "Etc/GMT-5").extract::<Tz>().unwrap(),
                Tz::Etc__GMTMinus5
            );
        });
    }

    #[test]
    fn test_ambiguous_datetime_to_pyobject() {
        let dates = [
            DateTime::<Utc>::from_str("2020-10-24 23:00:00 UTC").unwrap(),
            DateTime::<Utc>::from_str("2020-10-25 00:00:00 UTC").unwrap(),
            DateTime::<Utc>::from_str("2020-10-25 01:00:00 UTC").unwrap(),
        ];

        let dates = dates.map(|dt| dt.with_timezone(&Tz::Europe__London));

        assert_eq!(
            dates.map(|dt| dt.to_string()),
            [
                "2020-10-25 00:00:00 BST",
                "2020-10-25 01:00:00 BST",
                "2020-10-25 01:00:00 GMT"
            ]
        );

        let dates = Python::with_gil(|py| {
            let pydates = dates.map(|dt| dt.into_pyobject(py).unwrap());
            assert_eq!(
                pydates
                    .clone()
                    .map(|dt| dt.getattr("hour").unwrap().extract::<usize>().unwrap()),
                [0, 1, 1]
            );

            assert_eq!(
                pydates
                    .clone()
                    .map(|dt| dt.getattr("fold").unwrap().extract::<usize>().unwrap() > 0),
                [false, false, true]
            );

            pydates.map(|dt| dt.extract::<DateTime<Tz>>().unwrap())
        });

        assert_eq!(
            dates.map(|dt| dt.to_string()),
            [
                "2020-10-25 00:00:00 BST",
                "2020-10-25 01:00:00 BST",
                "2020-10-25 01:00:00 GMT"
            ]
        );
    }

    #[test]
    #[cfg(not(Py_GIL_DISABLED))] // https://github.com/python/cpython/issues/116738#issuecomment-2404360445
    fn test_into_pyobject() {
        Python::with_gil(|py| {
            let assert_eq = |l: Bound<'_, PyAny>, r: Bound<'_, PyAny>| {
                assert!(l.eq(&r).unwrap(), "{:?} != {:?}", l, r);
            };

            assert_eq(
                Tz::Europe__Paris.into_pyobject(py).unwrap(),
                new_zoneinfo(py, "Europe/Paris"),
            );
            assert_eq(Tz::UTC.into_pyobject(py).unwrap(), new_zoneinfo(py, "UTC"));
            assert_eq(
                Tz::Etc__GMTMinus5.into_pyobject(py).unwrap(),
                new_zoneinfo(py, "Etc/GMT-5"),
            );
        });
    }

    fn new_zoneinfo<'py>(py: Python<'py>, name: &str) -> Bound<'py, PyAny> {
        zoneinfo_class(py).call1((name,)).unwrap()
    }

    fn zoneinfo_class(py: Python<'_>) -> Bound<'_, PyAny> {
        py.import("zoneinfo").unwrap().getattr("ZoneInfo").unwrap()
    }
}
