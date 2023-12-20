use crate::exceptions::PyValueError;
#[cfg(Py_LIMITED_API)]
use crate::sync::GILOnceCell;
#[cfg(Py_LIMITED_API)]
use crate::types::PyType;
#[cfg(not(Py_LIMITED_API))]
use crate::types::{PyDelta, PyDeltaAccess};
#[cfg(Py_LIMITED_API)]
use crate::{intern, Py};
use crate::{FromPyObject, IntoPy, PyAny, PyObject, PyResult, Python, ToPyObject};
use std::time::Duration;

const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

impl FromPyObject<'_> for Duration {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        let (days, seconds, microseconds) = {
            let delta: &PyDelta = obj.downcast()?;
            (
                delta.get_days(),
                delta.get_seconds(),
                delta.get_microseconds(),
            )
        };
        #[cfg(Py_LIMITED_API)]
        let (days, seconds, microseconds): (i32, i32, i32) = {
            (
                obj.getattr(intern!(obj.py(), "days"))?.extract()?,
                obj.getattr(intern!(obj.py(), "seconds"))?.extract()?,
                obj.getattr(intern!(obj.py(), "microseconds"))?.extract()?,
            )
        };

        // We cast
        let days = u64::try_from(days).map_err(|_| {
            PyValueError::new_err(
                "It is not possible to convert a negative timedelta to a Rust Duration",
            )
        })?;
        let seconds = u64::try_from(seconds).unwrap(); // 0 <= seconds < 3600*24
        let microseconds = u32::try_from(microseconds).unwrap(); // 0 <= microseconds < 1000000

        // We convert
        let total_seconds = days * SECONDS_PER_DAY + seconds; // We casted from i32, this can't overflow
        let nanoseconds = microseconds.checked_mul(1_000).unwrap(); // 0 <= microseconds < 1000000

        Ok(Duration::new(total_seconds, nanoseconds))
    }
}

impl ToPyObject for Duration {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let days = self.as_secs() / SECONDS_PER_DAY;
        let seconds = self.as_secs() % SECONDS_PER_DAY;
        let microseconds = self.subsec_micros();

        #[cfg(not(Py_LIMITED_API))]
        {
            PyDelta::new(
                py,
                days.try_into()
                    .expect("Too large Rust duration for timedelta"),
                seconds.try_into().unwrap(),
                microseconds.try_into().unwrap(),
                false,
            )
            .expect("failed to construct timedelta (overflow?)")
            .into()
        }
        #[cfg(Py_LIMITED_API)]
        {
            static TIMEDELTA: GILOnceCell<Py<PyType>> = GILOnceCell::new();
            TIMEDELTA
                .get_or_try_init_type_ref(py, "datetime", "timedelta")
                .unwrap()
                .call1((days, seconds, microseconds))
                .unwrap()
                .into()
        }
    }
}

impl IntoPy<PyObject> for Duration {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    #[test]
    fn test_frompyobject() {
        Python::with_gil(|py| {
            assert_eq!(
                new_timedelta(py, 0, 0, 0).extract::<Duration>().unwrap(),
                Duration::new(0, 0)
            );
            assert_eq!(
                new_timedelta(py, 1, 0, 0).extract::<Duration>().unwrap(),
                Duration::new(86400, 0)
            );
            assert_eq!(
                new_timedelta(py, 0, 1, 0).extract::<Duration>().unwrap(),
                Duration::new(1, 0)
            );
            assert_eq!(
                new_timedelta(py, 0, 0, 1).extract::<Duration>().unwrap(),
                Duration::new(0, 1_000)
            );
            assert_eq!(
                new_timedelta(py, 1, 1, 1).extract::<Duration>().unwrap(),
                Duration::new(86401, 1_000)
            );
            assert_eq!(
                timedelta_class(py)
                    .getattr("max")
                    .unwrap()
                    .extract::<Duration>()
                    .unwrap(),
                Duration::new(86399999999999, 999999000)
            );
        });
    }

    #[test]
    fn test_frompyobject_negative() {
        Python::with_gil(|py| {
            assert_eq!(
                new_timedelta(py, 0, -1, 0)
                    .extract::<Duration>()
                    .unwrap_err()
                    .to_string(),
                "ValueError: It is not possible to convert a negative timedelta to a Rust Duration"
            );
        })
    }

    #[test]
    fn test_topyobject() {
        Python::with_gil(|py| {
            let assert_eq = |l: PyObject, r: &PyAny| {
                assert!(l.as_ref(py).eq(r).unwrap());
            };

            assert_eq(
                Duration::new(0, 0).to_object(py),
                new_timedelta(py, 0, 0, 0),
            );
            assert_eq(
                Duration::new(86400, 0).to_object(py),
                new_timedelta(py, 1, 0, 0),
            );
            assert_eq(
                Duration::new(1, 0).to_object(py),
                new_timedelta(py, 0, 1, 0),
            );
            assert_eq(
                Duration::new(0, 1_000).to_object(py),
                new_timedelta(py, 0, 0, 1),
            );
            assert_eq(
                Duration::new(0, 1).to_object(py),
                new_timedelta(py, 0, 0, 0),
            );
            assert_eq(
                Duration::new(86401, 1_000).to_object(py),
                new_timedelta(py, 1, 1, 1),
            );
            assert_eq(
                Duration::new(86399999999999, 999999000).to_object(py),
                timedelta_class(py).getattr("max").unwrap(),
            );
        });
    }

    #[test]
    fn test_topyobject_overflow() {
        Python::with_gil(|py| {
            assert!(panic::catch_unwind(|| Duration::MAX.to_object(py)).is_err());
        })
    }

    fn new_timedelta(py: Python<'_>, days: i32, seconds: i32, microseconds: i32) -> &PyAny {
        timedelta_class(py)
            .call1((days, seconds, microseconds))
            .unwrap()
    }

    fn timedelta_class(py: Python<'_>) -> &PyAny {
        py.import("datetime").unwrap().getattr("timedelta").unwrap()
    }
}
