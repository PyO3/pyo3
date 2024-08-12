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
//! use pyo3::{Python, ToPyObject};
//!
//! fn main() {
//!     pyo3::prepare_freethreaded_python();
//!     Python::with_gil(|py| {
//!         // Convert to Python
//!         let py_tzinfo = Tz::Europe__Paris.to_object(py);
//!         // Convert back to Rust
//!         assert_eq!(py_tzinfo.extract::<Tz>(py).unwrap(), Tz::Europe__Paris);
//!     });
//! }
//! ```
use crate::conversion::IntoPyObject;
use crate::exceptions::PyValueError;
use crate::pybacked::PyBackedStr;
use crate::sync::GILOnceCell;
use crate::types::{any::PyAnyMethods, PyType};
use crate::{
    intern, Bound, FromPyObject, IntoPy, Py, PyAny, PyErr, PyObject, PyResult, Python, ToPyObject,
};
use chrono_tz::Tz;
use std::str::FromStr;

impl ToPyObject for Tz {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        static ZONE_INFO: GILOnceCell<Py<PyType>> = GILOnceCell::new();
        ZONE_INFO
            .get_or_try_init_type_ref(py, "zoneinfo", "ZoneInfo")
            .unwrap()
            .call1((self.name(),))
            .unwrap()
            .unbind()
    }
}

impl IntoPy<PyObject> for Tz {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

impl<'py> IntoPyObject<'py> for Tz {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        static ZONE_INFO: GILOnceCell<Py<PyType>> = GILOnceCell::new();
        ZONE_INFO
            .get_or_try_init_type_ref(py, "zoneinfo", "ZoneInfo")
            .and_then(|obj| obj.call1((self.name(),)))
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
    fn test_topyobject() {
        Python::with_gil(|py| {
            let assert_eq = |l: PyObject, r: Bound<'_, PyAny>| {
                assert!(l.bind(py).eq(r).unwrap());
            };

            assert_eq(
                Tz::Europe__Paris.to_object(py),
                new_zoneinfo(py, "Europe/Paris"),
            );
            assert_eq(Tz::UTC.to_object(py), new_zoneinfo(py, "UTC"));
            assert_eq(
                Tz::Etc__GMTMinus5.to_object(py),
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
