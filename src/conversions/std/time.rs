use crate::conversion::IntoPyObject;
use crate::exceptions::{PyOverflowError, PyValueError};
use crate::sync::GILOnceCell;
use crate::types::any::PyAnyMethods;
#[cfg(Py_LIMITED_API)]
use crate::types::PyType;
#[cfg(not(Py_LIMITED_API))]
use crate::types::{timezone_utc_bound, PyDateTime, PyDelta, PyDeltaAccess};
#[cfg(Py_LIMITED_API)]
use crate::Py;
use crate::{
    intern, Bound, FromPyObject, IntoPy, PyAny, PyErr, PyObject, PyResult, Python, ToPyObject,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

impl FromPyObject<'_> for Duration {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        let (days, seconds, microseconds) = {
            let delta = obj.downcast::<PyDelta>()?;
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
            PyDelta::new_bound(
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

impl<'py> IntoPyObject<'py> for Duration {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDelta;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let days = self.as_secs() / SECONDS_PER_DAY;
        let seconds = self.as_secs() % SECONDS_PER_DAY;
        let microseconds = self.subsec_micros();

        #[cfg(not(Py_LIMITED_API))]
        {
            PyDelta::new_bound(
                py,
                days.try_into()?,
                seconds.try_into().unwrap(),
                microseconds.try_into().unwrap(),
                false,
            )
        }
        #[cfg(Py_LIMITED_API)]
        {
            static TIMEDELTA: GILOnceCell<Py<PyType>> = GILOnceCell::new();
            TIMEDELTA
                .get_or_try_init_type_ref(py, "datetime", "timedelta")?
                .call1((days, seconds, microseconds))
        }
    }
}

impl<'py> IntoPyObject<'py> for &Duration {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDelta;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

// Conversions between SystemTime and datetime do not rely on the floating point timestamp of the
// timestamp/fromtimestamp APIs to avoid possible precision loss but goes through the
// timedelta/std::time::Duration types by taking for reference point the UNIX epoch.
//
// TODO: it might be nice to investigate using timestamps anyway, at least when the datetime is a safe range.

impl FromPyObject<'_> for SystemTime {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let duration_since_unix_epoch: Duration = obj
            .call_method1(intern!(obj.py(), "__sub__"), (unix_epoch_py(obj.py()),))?
            .extract()?;
        UNIX_EPOCH
            .checked_add(duration_since_unix_epoch)
            .ok_or_else(|| {
                PyOverflowError::new_err("Overflow error when converting the time to Rust")
            })
    }
}

impl ToPyObject for SystemTime {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let duration_since_unix_epoch = self.duration_since(UNIX_EPOCH).unwrap().into_py(py);
        unix_epoch_py(py)
            .call_method1(py, intern!(py, "__add__"), (duration_since_unix_epoch,))
            .unwrap()
    }
}

impl IntoPy<PyObject> for SystemTime {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

impl<'py> IntoPyObject<'py> for SystemTime {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let duration_since_unix_epoch =
            self.duration_since(UNIX_EPOCH).unwrap().into_pyobject(py)?;
        unix_epoch_py(py)
            .bind(py)
            .call_method1(intern!(py, "__add__"), (duration_since_unix_epoch,))
    }
}

impl<'py> IntoPyObject<'py> for &SystemTime {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

fn unix_epoch_py(py: Python<'_>) -> &PyObject {
    static UNIX_EPOCH: GILOnceCell<PyObject> = GILOnceCell::new();
    UNIX_EPOCH
        .get_or_try_init(py, || {
            #[cfg(not(Py_LIMITED_API))]
            {
                Ok::<_, PyErr>(
                    PyDateTime::new_bound(
                        py,
                        1970,
                        1,
                        1,
                        0,
                        0,
                        0,
                        0,
                        Some(&timezone_utc_bound(py)),
                    )?
                    .into(),
                )
            }
            #[cfg(Py_LIMITED_API)]
            {
                let datetime = py.import("datetime")?;
                let utc = datetime.getattr("timezone")?.getattr("utc")?;
                Ok::<_, PyErr>(
                    datetime
                        .getattr("datetime")?
                        .call1((1970, 1, 1, 0, 0, 0, 0, utc))
                        .unwrap()
                        .into(),
                )
            }
        })
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PyDict;
    use std::panic;

    #[test]
    fn test_duration_frompyobject() {
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
    fn test_duration_frompyobject_negative() {
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
    fn test_duration_topyobject() {
        Python::with_gil(|py| {
            let assert_eq = |l: PyObject, r: Bound<'_, PyAny>| {
                assert!(l.bind(py).eq(r).unwrap());
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
    fn test_duration_topyobject_overflow() {
        Python::with_gil(|py| {
            assert!(panic::catch_unwind(|| Duration::MAX.to_object(py)).is_err());
        })
    }

    #[test]
    fn test_time_frompyobject() {
        Python::with_gil(|py| {
            assert_eq!(
                new_datetime(py, 1970, 1, 1, 0, 0, 0, 0)
                    .extract::<SystemTime>()
                    .unwrap(),
                UNIX_EPOCH
            );
            assert_eq!(
                new_datetime(py, 2020, 2, 3, 4, 5, 6, 7)
                    .extract::<SystemTime>()
                    .unwrap(),
                UNIX_EPOCH
                    .checked_add(Duration::new(1580702706, 7000))
                    .unwrap()
            );
            assert_eq!(
                max_datetime(py).extract::<SystemTime>().unwrap(),
                UNIX_EPOCH
                    .checked_add(Duration::new(253402300799, 999999000))
                    .unwrap()
            );
        });
    }

    #[test]
    fn test_time_frompyobject_before_epoch() {
        Python::with_gil(|py| {
            assert_eq!(
                new_datetime(py, 1950, 1, 1, 0, 0, 0, 0)
                    .extract::<SystemTime>()
                    .unwrap_err()
                    .to_string(),
                "ValueError: It is not possible to convert a negative timedelta to a Rust Duration"
            );
        })
    }

    #[test]
    fn test_time_topyobject() {
        Python::with_gil(|py| {
            let assert_eq = |l: PyObject, r: Bound<'_, PyAny>| {
                assert!(l.bind(py).eq(r).unwrap());
            };

            assert_eq(
                UNIX_EPOCH
                    .checked_add(Duration::new(1580702706, 7123))
                    .unwrap()
                    .into_py(py),
                new_datetime(py, 2020, 2, 3, 4, 5, 6, 7),
            );
            assert_eq(
                UNIX_EPOCH
                    .checked_add(Duration::new(253402300799, 999999000))
                    .unwrap()
                    .into_py(py),
                max_datetime(py),
            );
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn new_datetime(
        py: Python<'_>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
    ) -> Bound<'_, PyAny> {
        datetime_class(py)
            .call1((
                year,
                month,
                day,
                hour,
                minute,
                second,
                microsecond,
                tz_utc(py),
            ))
            .unwrap()
    }

    fn max_datetime(py: Python<'_>) -> Bound<'_, PyAny> {
        let naive_max = datetime_class(py).getattr("max").unwrap();
        let kargs = PyDict::new(py);
        kargs.set_item("tzinfo", tz_utc(py)).unwrap();
        naive_max.call_method("replace", (), Some(&kargs)).unwrap()
    }

    #[test]
    fn test_time_topyobject_overflow() {
        let big_system_time = UNIX_EPOCH
            .checked_add(Duration::new(300000000000, 0))
            .unwrap();
        Python::with_gil(|py| {
            assert!(panic::catch_unwind(|| big_system_time.into_py(py)).is_err());
        })
    }

    fn tz_utc(py: Python<'_>) -> Bound<'_, PyAny> {
        py.import("datetime")
            .unwrap()
            .getattr("timezone")
            .unwrap()
            .getattr("utc")
            .unwrap()
    }

    fn new_timedelta(
        py: Python<'_>,
        days: i32,
        seconds: i32,
        microseconds: i32,
    ) -> Bound<'_, PyAny> {
        timedelta_class(py)
            .call1((days, seconds, microseconds))
            .unwrap()
    }

    fn datetime_class(py: Python<'_>) -> Bound<'_, PyAny> {
        py.import("datetime").unwrap().getattr("datetime").unwrap()
    }

    fn timedelta_class(py: Python<'_>) -> Bound<'_, PyAny> {
        py.import("datetime").unwrap().getattr("timedelta").unwrap()
    }
}
