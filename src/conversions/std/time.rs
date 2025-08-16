use crate::conversion::IntoPyObject;
use crate::exceptions::{PyOverflowError, PyValueError};
#[cfg(Py_LIMITED_API)]
use crate::intern;
use crate::sync::PyOnceLock;
use crate::types::any::PyAnyMethods;
#[cfg(not(Py_LIMITED_API))]
use crate::types::PyDeltaAccess;
use crate::types::{PyDateTime, PyDelta, PyTzInfo};
use crate::{Borrowed, Bound, FromPyObject, Py, PyAny, PyErr, PyResult, Python};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

impl FromPyObject<'_> for Duration {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let delta = obj.cast::<PyDelta>()?;
        #[cfg(not(Py_LIMITED_API))]
        let (days, seconds, microseconds) = {
            (
                delta.get_days(),
                delta.get_seconds(),
                delta.get_microseconds(),
            )
        };
        #[cfg(Py_LIMITED_API)]
        let (days, seconds, microseconds): (i32, i32, i32) = {
            let py = delta.py();
            (
                delta.getattr(intern!(py, "days"))?.extract()?,
                delta.getattr(intern!(py, "seconds"))?.extract()?,
                delta.getattr(intern!(py, "microseconds"))?.extract()?,
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

impl<'py> IntoPyObject<'py> for Duration {
    type Target = PyDelta;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let days = self.as_secs() / SECONDS_PER_DAY;
        let seconds = self.as_secs() % SECONDS_PER_DAY;
        let microseconds = self.subsec_micros();

        PyDelta::new(
            py,
            days.try_into()?,
            seconds.try_into()?,
            microseconds.try_into()?,
            false,
        )
    }
}

impl<'py> IntoPyObject<'py> for &Duration {
    type Target = PyDelta;
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
        let duration_since_unix_epoch: Duration = obj.sub(unix_epoch_py(obj.py())?)?.extract()?;
        UNIX_EPOCH
            .checked_add(duration_since_unix_epoch)
            .ok_or_else(|| {
                PyOverflowError::new_err("Overflow error when converting the time to Rust")
            })
    }
}

impl<'py> IntoPyObject<'py> for SystemTime {
    type Target = PyDateTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let duration_since_unix_epoch =
            self.duration_since(UNIX_EPOCH).unwrap().into_pyobject(py)?;
        unix_epoch_py(py)?
            .add(duration_since_unix_epoch)?
            .cast_into()
            .map_err(Into::into)
    }
}

impl<'py> IntoPyObject<'py> for &SystemTime {
    type Target = PyDateTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

fn unix_epoch_py(py: Python<'_>) -> PyResult<Borrowed<'_, '_, PyDateTime>> {
    static UNIX_EPOCH: PyOnceLock<Py<PyDateTime>> = PyOnceLock::new();
    Ok(UNIX_EPOCH
        .get_or_try_init(py, || {
            let utc = PyTzInfo::utc(py)?;
            Ok::<_, PyErr>(PyDateTime::new(py, 1970, 1, 1, 0, 0, 0, 0, Some(&utc))?.into())
        })?
        .bind_borrowed(py))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PyDict;

    #[test]
    fn test_duration_frompyobject() {
        Python::attach(|py| {
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
        Python::attach(|py| {
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
    fn test_duration_into_pyobject() {
        Python::attach(|py| {
            let assert_eq = |l: Bound<'_, PyAny>, r: Bound<'_, PyAny>| {
                assert!(l.eq(r).unwrap());
            };

            assert_eq(
                Duration::new(0, 0).into_pyobject(py).unwrap().into_any(),
                new_timedelta(py, 0, 0, 0),
            );
            assert_eq(
                Duration::new(86400, 0)
                    .into_pyobject(py)
                    .unwrap()
                    .into_any(),
                new_timedelta(py, 1, 0, 0),
            );
            assert_eq(
                Duration::new(1, 0).into_pyobject(py).unwrap().into_any(),
                new_timedelta(py, 0, 1, 0),
            );
            assert_eq(
                Duration::new(0, 1_000)
                    .into_pyobject(py)
                    .unwrap()
                    .into_any(),
                new_timedelta(py, 0, 0, 1),
            );
            assert_eq(
                Duration::new(0, 1).into_pyobject(py).unwrap().into_any(),
                new_timedelta(py, 0, 0, 0),
            );
            assert_eq(
                Duration::new(86401, 1_000)
                    .into_pyobject(py)
                    .unwrap()
                    .into_any(),
                new_timedelta(py, 1, 1, 1),
            );
            assert_eq(
                Duration::new(86399999999999, 999999000)
                    .into_pyobject(py)
                    .unwrap()
                    .into_any(),
                timedelta_class(py).getattr("max").unwrap(),
            );
        });
    }

    #[test]
    fn test_duration_into_pyobject_overflow() {
        Python::attach(|py| {
            assert!(Duration::MAX.into_pyobject(py).is_err());
        })
    }

    #[test]
    fn test_time_frompyobject() {
        Python::attach(|py| {
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
        Python::attach(|py| {
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
    fn test_time_intopyobject() {
        Python::attach(|py| {
            let assert_eq = |l: Bound<'_, PyDateTime>, r: Bound<'_, PyDateTime>| {
                assert!(l.eq(r).unwrap());
            };

            assert_eq(
                UNIX_EPOCH
                    .checked_add(Duration::new(1580702706, 7123))
                    .unwrap()
                    .into_pyobject(py)
                    .unwrap(),
                new_datetime(py, 2020, 2, 3, 4, 5, 6, 7),
            );
            assert_eq(
                UNIX_EPOCH
                    .checked_add(Duration::new(253402300799, 999999000))
                    .unwrap()
                    .into_pyobject(py)
                    .unwrap(),
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
    ) -> Bound<'_, PyDateTime> {
        let utc = PyTzInfo::utc(py).unwrap();
        PyDateTime::new(
            py,
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond,
            Some(&utc),
        )
        .unwrap()
    }

    fn max_datetime(py: Python<'_>) -> Bound<'_, PyDateTime> {
        let naive_max = datetime_class(py).getattr("max").unwrap();
        let kargs = PyDict::new(py);
        kargs
            .set_item("tzinfo", PyTzInfo::utc(py).unwrap())
            .unwrap();
        naive_max
            .call_method("replace", (), Some(&kargs))
            .unwrap()
            .cast_into()
            .unwrap()
    }

    #[test]
    fn test_time_intopyobject_overflow() {
        let big_system_time = UNIX_EPOCH
            .checked_add(Duration::new(300000000000, 0))
            .unwrap();
        Python::attach(|py| {
            assert!(big_system_time.into_pyobject(py).is_err());
        })
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
