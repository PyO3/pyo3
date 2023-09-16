#![cfg(all(feature = "chrono", not(Py_LIMITED_API)))]

//! Conversions to and from [chrono](https://docs.rs/chrono/)’s `Duration`,
//! `NaiveDate`, `NaiveTime`, `DateTime<Tz>`, `FixedOffset`, and `Utc`.
//!
//! Unavailable with the `abi3` feature.
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! # change * to the latest versions
//! pyo3 = { version = "*", features = ["chrono"] }
//! chrono = "0.4"
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"chrono\"] }")]
//! ```
//!
//! Note that you must use compatible versions of chrono and PyO3.
//! The required chrono version may vary based on the version of PyO3.
//!
//! # Example: Convert a `PyDateTime` to chrono's `DateTime<Utc>`
//!
//! ```rust
//! use chrono::{Utc, DateTime};
//! use pyo3::{Python, ToPyObject, types::PyDateTime};
//!
//! fn main() {
//!     pyo3::prepare_freethreaded_python();
//!     Python::with_gil(|py| {
//!         // Create an UTC datetime in python
//!         let py_tz = Utc.to_object(py);
//!         let py_tz = py_tz.downcast(py).unwrap();
//!         let py_datetime = PyDateTime::new(py, 2022, 1, 1, 12, 0, 0, 0, Some(py_tz)).unwrap();
//!         println!("PyDateTime: {}", py_datetime);
//!         // Now convert it to chrono's DateTime<Utc>
//!         let chrono_datetime: DateTime<Utc> = py_datetime.extract().unwrap();
//!         println!("DateTime<Utc>: {}", chrono_datetime);
//!     });
//! }
//! ```
use crate::exceptions::{PyTypeError, PyUserWarning, PyValueError};
use crate::types::{
    timezone_utc, PyDate, PyDateAccess, PyDateTime, PyDelta, PyDeltaAccess, PyTime, PyTimeAccess,
    PyTzInfo, PyTzInfoAccess, PyUnicode,
};
use crate::{FromPyObject, IntoPy, PyAny, PyErr, PyObject, PyResult, Python, ToPyObject};
use chrono::offset::{FixedOffset, Utc};
use chrono::{
    DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Timelike,
};
use pyo3_ffi::{PyDateTime_IMPORT, PyTimeZone_FromOffset};
use std::convert::TryInto;

impl ToPyObject for Duration {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        // Total number of days
        let days = self.num_days();
        // Remainder of seconds
        let secs_dur = *self - Duration::days(days);
        // .try_into() converts i64 to i32, but this should never overflow
        // since it's at most the number of seconds per day
        let secs = secs_dur.num_seconds().try_into().unwrap();
        // Fractional part of the microseconds
        let micros = (secs_dur - Duration::seconds(secs_dur.num_seconds()))
            .num_microseconds()
            // This should never panic since we are just getting the fractional
            // part of the total microseconds, which should never overflow.
            .unwrap()
            // Same for the conversion from i64 to i32
            .try_into()
            .unwrap();

        // We do not need to check i64 to i32 cast from rust because
        // python will panic with OverflowError.
        // We pass true as the `normalize` parameter since we'd need to do several checks here to
        // avoid that, and it shouldn't have a big performance impact.
        let delta = PyDelta::new(py, days.try_into().unwrap_or(i32::MAX), secs, micros, true)
            .expect("failed to construct delta");
        delta.into()
    }
}

impl IntoPy<PyObject> for Duration {
    fn into_py(self, py: Python<'_>) -> PyObject {
        ToPyObject::to_object(&self, py)
    }
}

impl FromPyObject<'_> for Duration {
    fn extract(ob: &PyAny) -> PyResult<Duration> {
        let delta: &PyDelta = ob.downcast()?;
        // Python size are much lower than rust size so we do not need bound checks.
        // 0 <= microseconds < 1000000
        // 0 <= seconds < 3600*24
        // -999999999 <= days <= 999999999
        Ok(Duration::days(delta.get_days().into())
            + Duration::seconds(delta.get_seconds().into())
            + Duration::microseconds(delta.get_microseconds().into()))
    }
}

impl ToPyObject for NaiveDate {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        (*self).into_py(py)
    }
}

impl IntoPy<PyObject> for NaiveDate {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let DateArgs { year, month, day } = self.into();
        PyDate::new(py, year, month, day)
            .expect("failed to construct date")
            .into()
    }
}

impl FromPyObject<'_> for NaiveDate {
    fn extract(ob: &PyAny) -> PyResult<NaiveDate> {
        let date: &PyDate = ob.downcast()?;
        py_date_to_naive_date(date)
    }
}

impl ToPyObject for NaiveTime {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        (*self).into_py(py)
    }
}

impl IntoPy<PyObject> for NaiveTime {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let TimeArgs {
            hour,
            min,
            sec,
            micro,
            truncated_leap_second,
        } = self.into();
        let time = PyTime::new(py, hour, min, sec, micro, None).expect("Failed to construct time");
        if truncated_leap_second {
            warn_truncated_leap_second(time);
        }
        time.into()
    }
}

impl FromPyObject<'_> for NaiveTime {
    fn extract(ob: &PyAny) -> PyResult<NaiveTime> {
        let time: &PyTime = ob.downcast()?;
        py_time_to_naive_time(time)
    }
}

impl ToPyObject for NaiveDateTime {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        naive_datetime_to_py_datetime(py, self, None)
            .expect("failed to construct datetime")
            .into()
    }
}

impl IntoPy<PyObject> for NaiveDateTime {
    fn into_py(self, py: Python<'_>) -> PyObject {
        ToPyObject::to_object(&self, py)
    }
}

impl FromPyObject<'_> for NaiveDateTime {
    fn extract(ob: &PyAny) -> PyResult<NaiveDateTime> {
        let dt: &PyDateTime = ob.downcast()?;
        // If the user tries to convert a timezone aware datetime into a naive one,
        // we return a hard error. We could silently remove tzinfo, or assume local timezone
        // and do a conversion, but better leave this decision to the user of the library.
        if dt.get_tzinfo().is_some() {
            return Err(PyTypeError::new_err("expected a datetime without tzinfo"));
        }

        let dt = NaiveDateTime::new(py_date_to_naive_date(dt)?, py_time_to_naive_time(dt)?);
        Ok(dt)
    }
}

impl<Tz: TimeZone> ToPyObject for DateTime<Tz> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        // FIXME: convert to better timezone representation here than just convert to fixed offset
        // See https://github.com/PyO3/pyo3/issues/3266
        let tz = self.offset().fix().to_object(py);
        let tz = tz.downcast(py).unwrap();
        naive_datetime_to_py_datetime(py, &self.naive_local(), Some(tz))
            .expect("failed to construct datetime")
            .into()
    }
}

impl<Tz: TimeZone> IntoPy<PyObject> for DateTime<Tz> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        ToPyObject::to_object(&self, py)
    }
}

impl FromPyObject<'_> for DateTime<FixedOffset> {
    fn extract(ob: &PyAny) -> PyResult<DateTime<FixedOffset>> {
        let dt: &PyDateTime = ob.downcast()?;
        let tz: FixedOffset = if let Some(tzinfo) = dt.get_tzinfo() {
            tzinfo.extract()?
        } else {
            return Err(PyTypeError::new_err(
                "expected a datetime with non-None tzinfo",
            ));
        };
        let dt = NaiveDateTime::new(py_date_to_naive_date(dt)?, py_time_to_naive_time(dt)?);
        // `FixedOffset` cannot have ambiguities so we don't have to worry about DST folds and such
        Ok(dt.and_local_timezone(tz).unwrap())
    }
}

impl FromPyObject<'_> for DateTime<Utc> {
    fn extract(ob: &PyAny) -> PyResult<DateTime<Utc>> {
        let dt: &PyDateTime = ob.downcast()?;
        let _: Utc = if let Some(tzinfo) = dt.get_tzinfo() {
            tzinfo.extract()?
        } else {
            return Err(PyTypeError::new_err(
                "expected a datetime with non-None tzinfo",
            ));
        };
        let dt = NaiveDateTime::new(py_date_to_naive_date(dt)?, py_time_to_naive_time(dt)?);
        Ok(dt.and_utc())
    }
}

// Utility function used to convert PyDelta to timezone
fn py_timezone_from_offset<'a>(py: &Python<'a>, td: &PyDelta) -> &'a PyAny {
    // Safety: py.from_owned_ptr needs the cast to be valid.
    // Since we are forcing a &PyDelta as input, the cast should always be valid.
    unsafe {
        PyDateTime_IMPORT();
        py.from_owned_ptr(PyTimeZone_FromOffset(td.as_ptr()))
    }
}

impl ToPyObject for FixedOffset {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let seconds_offset = self.local_minus_utc();
        let td =
            PyDelta::new(py, 0, seconds_offset, 0, true).expect("failed to construct timedelta");
        py_timezone_from_offset(&py, td).into()
    }
}

impl IntoPy<PyObject> for FixedOffset {
    fn into_py(self, py: Python<'_>) -> PyObject {
        ToPyObject::to_object(&self, py)
    }
}

impl FromPyObject<'_> for FixedOffset {
    /// Convert python tzinfo to rust [`FixedOffset`].
    ///
    /// Note that the conversion will result in precision lost in microseconds as chrono offset
    /// does not supports microseconds.
    fn extract(ob: &PyAny) -> PyResult<FixedOffset> {
        let py_tzinfo: &PyTzInfo = ob.downcast()?;
        // Passing `ob.py().None()` (so Python's None) to the `utcoffset` function will only
        // work for timezones defined as fixed offsets in Python.
        // Any other timezone would require a datetime as the parameter, and return
        // None if the datetime is not provided.
        // Trying to convert None to a PyDelta in the next line will then fail.
        let py_timedelta = py_tzinfo.call_method1("utcoffset", (ob.py().None(),))?;
        let py_timedelta: &PyDelta = py_timedelta.downcast().map_err(|_| {
            PyTypeError::new_err(format!(
                "{:?} is not a fixed offset timezone",
                py_tzinfo
                    .repr()
                    .unwrap_or_else(|_| PyUnicode::new(ob.py(), "repr failed"))
            ))
        })?;
        let days = py_timedelta.get_days() as i64;
        let seconds = py_timedelta.get_seconds() as i64;
        // Here we won't get microseconds as noted before
        // let microseconds = py_timedelta.get_microseconds() as i64;
        let total_seconds = Duration::days(days) + Duration::seconds(seconds);
        // This cast is safe since the timedelta is limited to -24 hours and 24 hours.
        let total_seconds = total_seconds.num_seconds() as i32;
        FixedOffset::east_opt(total_seconds)
            .ok_or_else(|| PyValueError::new_err("fixed offset out of bounds"))
    }
}

impl ToPyObject for Utc {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        timezone_utc(py).to_object(py)
    }
}

impl IntoPy<PyObject> for Utc {
    fn into_py(self, py: Python<'_>) -> PyObject {
        ToPyObject::to_object(&self, py)
    }
}

impl FromPyObject<'_> for Utc {
    fn extract(ob: &PyAny) -> PyResult<Utc> {
        let py_tzinfo: &PyTzInfo = ob.downcast()?;
        let py_utc = timezone_utc(ob.py());
        if py_tzinfo.eq(py_utc)? {
            Ok(Utc)
        } else {
            Err(PyValueError::new_err("expected datetime.timezone.utc"))
        }
    }
}

struct DateArgs {
    year: i32,
    month: u8,
    day: u8,
}

impl From<NaiveDate> for DateArgs {
    fn from(value: NaiveDate) -> Self {
        Self {
            year: value.year(),
            month: value.month() as u8,
            day: value.day() as u8,
        }
    }
}

struct TimeArgs {
    hour: u8,
    min: u8,
    sec: u8,
    micro: u32,
    truncated_leap_second: bool,
}

impl From<NaiveTime> for TimeArgs {
    fn from(value: NaiveTime) -> Self {
        let ns = value.nanosecond();
        let checked_sub = ns.checked_sub(1_000_000_000);
        let truncated_leap_second = checked_sub.is_some();
        let micro = checked_sub.unwrap_or(ns) / 1000;
        Self {
            hour: value.hour() as u8,
            min: value.minute() as u8,
            sec: value.second() as u8,
            micro,
            truncated_leap_second,
        }
    }
}

fn naive_datetime_to_py_datetime<'py>(
    py: Python<'py>,
    naive_datetime: &NaiveDateTime,
    tzinfo: Option<&PyTzInfo>,
) -> PyResult<&'py PyDateTime> {
    let DateArgs { year, month, day } = naive_datetime.date().into();
    let TimeArgs {
        hour,
        min,
        sec,
        micro,
        truncated_leap_second,
    } = naive_datetime.time().into();
    let datetime = PyDateTime::new(py, year, month, day, hour, min, sec, micro, tzinfo)?;
    if truncated_leap_second {
        warn_truncated_leap_second(datetime);
    }
    Ok(datetime)
}

fn warn_truncated_leap_second(obj: &PyAny) {
    let py = obj.py();
    if let Err(e) = PyErr::warn(
        py,
        py.get_type::<PyUserWarning>(),
        "ignored leap-second, `datetime` does not support leap-seconds",
        0,
    ) {
        e.write_unraisable(py, Some(obj))
    };
}

fn py_date_to_naive_date(py_date: &impl PyDateAccess) -> PyResult<NaiveDate> {
    NaiveDate::from_ymd_opt(
        py_date.get_year(),
        py_date.get_month().into(),
        py_date.get_day().into(),
    )
    .ok_or_else(|| PyValueError::new_err("invalid or out-of-range date"))
}

fn py_time_to_naive_time(py_time: &impl PyTimeAccess) -> PyResult<NaiveTime> {
    NaiveTime::from_hms_micro_opt(
        py_time.get_hour().into(),
        py_time.get_minute().into(),
        py_time.get_second().into(),
        py_time.get_microsecond(),
    )
    .ok_or_else(|| PyValueError::new_err("invalid or out-of-range time"))
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, panic};

    use crate::{tests::common::CatchWarnings, PyTypeInfo};

    use super::*;

    #[test]
    // Only Python>=3.9 has the zoneinfo package
    // We skip the test on windows too since we'd need to install
    // tzdata there to make this work.
    #[cfg(all(Py_3_9, not(target_os = "windows")))]
    fn test_zoneinfo_is_not_fixed_offset() {
        Python::with_gil(|py| {
            let locals = crate::types::PyDict::new(py);
            py.run(
                "import zoneinfo; zi = zoneinfo.ZoneInfo('Europe/London')",
                None,
                Some(locals),
            )
            .unwrap();
            let result: PyResult<FixedOffset> = locals.get_item("zi").unwrap().unwrap().extract();
            assert!(result.is_err());
            let res = result.err().unwrap();
            // Also check the error message is what we expect
            let msg = res.value(py).repr().unwrap().to_string();
            assert_eq!(msg, "TypeError('\"zoneinfo.ZoneInfo(key=\\'Europe/London\\')\" is not a fixed offset timezone')");
        });
    }

    #[test]
    fn test_timezone_aware_to_naive_fails() {
        // Test that if a user tries to convert a python's timezone aware datetime into a naive
        // one, the conversion fails.
        Python::with_gil(|py| {
            let utc = timezone_utc(py);
            let py_datetime = PyDateTime::new(py, 2022, 1, 1, 1, 0, 0, 0, Some(utc)).unwrap();
            // Now test that converting a PyDateTime with tzinfo to a NaiveDateTime fails
            let res: PyResult<NaiveDateTime> = py_datetime.extract();
            assert!(res.is_err());
            let res = res.err().unwrap();
            // Also check the error message is what we expect
            let msg = res.value(py).repr().unwrap().to_string();
            assert_eq!(msg, "TypeError('expected a datetime without tzinfo')");
        });
    }

    #[test]
    fn test_naive_to_timezone_aware_fails() {
        // Test that if a user tries to convert a python's timezone aware datetime into a naive
        // one, the conversion fails.
        Python::with_gil(|py| {
            let py_datetime = PyDateTime::new(py, 2022, 1, 1, 1, 0, 0, 0, None).unwrap();
            // Now test that converting a PyDateTime with tzinfo to a NaiveDateTime fails
            let res: PyResult<DateTime<Utc>> = py_datetime.extract();
            assert!(res.is_err());
            let res = res.err().unwrap();
            // Also check the error message is what we expect
            let msg = res.value(py).repr().unwrap().to_string();
            assert_eq!(msg, "TypeError('expected a datetime with non-None tzinfo')");

            // Now test that converting a PyDateTime with tzinfo to a NaiveDateTime fails
            let res: PyResult<DateTime<FixedOffset>> = py_datetime.extract();
            assert!(res.is_err());
            let res = res.err().unwrap();
            // Also check the error message is what we expect
            let msg = res.value(py).repr().unwrap().to_string();
            assert_eq!(msg, "TypeError('expected a datetime with non-None tzinfo')");
        });
    }

    #[test]
    fn test_invalid_types_fail() {
        // Test that if a user tries to convert a python's timezone aware datetime into a naive
        // one, the conversion fails.
        Python::with_gil(|py| {
            let none = py.None().into_ref(py);
            assert_eq!(
                none.extract::<Duration>().unwrap_err().to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyDelta'"
            );
            assert_eq!(
                none.extract::<FixedOffset>().unwrap_err().to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyTzInfo'"
            );
            assert_eq!(
                none.extract::<Utc>().unwrap_err().to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyTzInfo'"
            );
            assert_eq!(
                none.extract::<NaiveTime>().unwrap_err().to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyTime'"
            );
            assert_eq!(
                none.extract::<NaiveDate>().unwrap_err().to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyDate'"
            );
            assert_eq!(
                none.extract::<NaiveDateTime>().unwrap_err().to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyDateTime'"
            );
            assert_eq!(
                none.extract::<DateTime<Utc>>().unwrap_err().to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyDateTime'"
            );
            assert_eq!(
                none.extract::<DateTime<FixedOffset>>()
                    .unwrap_err()
                    .to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyDateTime'"
            );
        });
    }

    #[test]
    fn test_pyo3_timedelta_topyobject() {
        // Utility function used to check different durations.
        // The `name` parameter is used to identify the check in case of a failure.
        let check = |name: &'static str, delta: Duration, py_days, py_seconds, py_ms| {
            Python::with_gil(|py| {
                let delta = delta.to_object(py);
                let delta: &PyDelta = delta.extract(py).unwrap();
                let py_delta = PyDelta::new(py, py_days, py_seconds, py_ms, true).unwrap();
                assert!(
                    delta.eq(py_delta).unwrap(),
                    "{}: {} != {}",
                    name,
                    delta,
                    py_delta
                );
            });
        };

        let delta = Duration::days(-1) + Duration::seconds(1) + Duration::microseconds(-10);
        check("delta normalization", delta, -1, 1, -10);

        // Check the minimum value allowed by PyDelta, which is different
        // from the minimum value allowed in Duration. This should pass.
        let delta = Duration::seconds(-86399999913600); // min
        check("delta min value", delta, -999999999, 0, 0);

        // Same, for max value
        let delta = Duration::seconds(86399999999999) + Duration::nanoseconds(999999000); // max
        check("delta max value", delta, 999999999, 86399, 999999);

        // Also check that trying to convert an out of bound value panics.
        Python::with_gil(|py| {
            assert!(panic::catch_unwind(|| Duration::min_value().to_object(py)).is_err());
            assert!(panic::catch_unwind(|| Duration::max_value().to_object(py)).is_err());
        });
    }

    #[test]
    fn test_pyo3_timedelta_frompyobject() {
        // Utility function used to check different durations.
        // The `name` parameter is used to identify the check in case of a failure.
        let check = |name: &'static str, delta: Duration, py_days, py_seconds, py_ms| {
            Python::with_gil(|py| {
                let py_delta = PyDelta::new(py, py_days, py_seconds, py_ms, true).unwrap();
                let py_delta: Duration = py_delta.extract().unwrap();
                assert_eq!(py_delta, delta, "{}: {} != {}", name, py_delta, delta);
            })
        };

        // Check the minimum value allowed by PyDelta, which is different
        // from the minimum value allowed in Duration. This should pass.
        check(
            "min py_delta value",
            Duration::seconds(-86399999913600),
            -999999999,
            0,
            0,
        );
        // Same, for max value
        check(
            "max py_delta value",
            Duration::seconds(86399999999999) + Duration::microseconds(999999),
            999999999,
            86399,
            999999,
        );

        // This check is to assert that we can't construct every possible Duration from a PyDelta
        // since they have different bounds.
        Python::with_gil(|py| {
            let low_days: i32 = -1000000000;
            // This is possible
            assert!(panic::catch_unwind(|| Duration::days(low_days as i64)).is_ok());
            // This panics on PyDelta::new
            assert!(panic::catch_unwind(|| {
                let py_delta = PyDelta::new(py, low_days, 0, 0, true).unwrap();
                if let Ok(_duration) = py_delta.extract::<Duration>() {
                    // So we should never get here
                }
            })
            .is_err());

            let high_days: i32 = 1000000000;
            // This is possible
            assert!(panic::catch_unwind(|| Duration::days(high_days as i64)).is_ok());
            // This panics on PyDelta::new
            assert!(panic::catch_unwind(|| {
                let py_delta = PyDelta::new(py, high_days, 0, 0, true).unwrap();
                if let Ok(_duration) = py_delta.extract::<Duration>() {
                    // So we should never get here
                }
            })
            .is_err());
        });
    }

    #[test]
    fn test_pyo3_date_topyobject() {
        let eq_ymd = |name: &'static str, year, month, day| {
            Python::with_gil(|py| {
                let date = NaiveDate::from_ymd_opt(year, month, day)
                    .unwrap()
                    .to_object(py);
                let date: &PyDate = date.extract(py).unwrap();
                let py_date = PyDate::new(py, year, month as u8, day as u8).unwrap();
                assert_eq!(
                    date.compare(py_date).unwrap(),
                    Ordering::Equal,
                    "{}: {} != {}",
                    name,
                    date,
                    py_date
                );
            })
        };

        eq_ymd("past date", 2012, 2, 29);
        eq_ymd("min date", 1, 1, 1);
        eq_ymd("future date", 3000, 6, 5);
        eq_ymd("max date", 9999, 12, 31);
    }

    #[test]
    fn test_pyo3_date_frompyobject() {
        let eq_ymd = |name: &'static str, year, month, day| {
            Python::with_gil(|py| {
                let py_date = PyDate::new(py, year, month as u8, day as u8).unwrap();
                let py_date: NaiveDate = py_date.extract().unwrap();
                let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
                assert_eq!(py_date, date, "{}: {} != {}", name, date, py_date);
            })
        };

        eq_ymd("past date", 2012, 2, 29);
        eq_ymd("min date", 1, 1, 1);
        eq_ymd("future date", 3000, 6, 5);
        eq_ymd("max date", 9999, 12, 31);
    }

    #[test]
    fn test_pyo3_datetime_topyobject_utc() {
        Python::with_gil(|py| {
            let check_utc =
                |name: &'static str, year, month, day, hour, minute, second, ms, py_ms| {
                    let datetime = NaiveDate::from_ymd_opt(year, month, day)
                        .unwrap()
                        .and_hms_micro_opt(hour, minute, second, ms)
                        .unwrap()
                        .and_utc();
                    let datetime = datetime.to_object(py);
                    let datetime: &PyDateTime = datetime.extract(py).unwrap();
                    let py_tz = timezone_utc(py);
                    let py_datetime = PyDateTime::new(
                        py,
                        year,
                        month as u8,
                        day as u8,
                        hour as u8,
                        minute as u8,
                        second as u8,
                        py_ms,
                        Some(py_tz),
                    )
                    .unwrap();
                    assert_eq!(
                        datetime.compare(py_datetime).unwrap(),
                        Ordering::Equal,
                        "{}: {} != {}",
                        name,
                        datetime,
                        py_datetime
                    );
                };

            check_utc("regular", 2014, 5, 6, 7, 8, 9, 999_999, 999_999);

            assert_warnings!(
                py,
                check_utc("leap second", 2014, 5, 6, 7, 8, 59, 1_999_999, 999_999),
                [(
                    PyUserWarning,
                    "ignored leap-second, `datetime` does not support leap-seconds"
                )]
            );
        })
    }

    #[test]
    fn test_pyo3_datetime_topyobject_fixed_offset() {
        Python::with_gil(|py| {
            let check_fixed_offset =
                |name: &'static str, year, month, day, hour, minute, second, ms, py_ms| {
                    let offset = FixedOffset::east_opt(3600).unwrap();
                    let datetime = NaiveDate::from_ymd_opt(year, month, day)
                        .unwrap()
                        .and_hms_micro_opt(hour, minute, second, ms)
                        .unwrap()
                        .and_local_timezone(offset)
                        .unwrap();
                    let datetime = datetime.to_object(py);
                    let datetime: &PyDateTime = datetime.extract(py).unwrap();
                    let py_tz = offset.to_object(py);
                    let py_tz = py_tz.downcast(py).unwrap();
                    let py_datetime = PyDateTime::new(
                        py,
                        year,
                        month as u8,
                        day as u8,
                        hour as u8,
                        minute as u8,
                        second as u8,
                        py_ms,
                        Some(py_tz),
                    )
                    .unwrap();
                    assert_eq!(
                        datetime.compare(py_datetime).unwrap(),
                        Ordering::Equal,
                        "{}: {} != {}",
                        name,
                        datetime,
                        py_datetime
                    );
                };

            check_fixed_offset("regular", 2014, 5, 6, 7, 8, 9, 999_999, 999_999);

            assert_warnings!(
                py,
                check_fixed_offset("leap second", 2014, 5, 6, 7, 8, 59, 1_999_999, 999_999),
                [(
                    PyUserWarning,
                    "ignored leap-second, `datetime` does not support leap-seconds"
                )]
            );
        })
    }

    #[test]
    fn test_pyo3_datetime_frompyobject_utc() {
        Python::with_gil(|py| {
            let year = 2014;
            let month = 5;
            let day = 6;
            let hour = 7;
            let minute = 8;
            let second = 9;
            let micro = 999_999;
            let py_tz = timezone_utc(py);
            let py_datetime = PyDateTime::new(
                py,
                year,
                month,
                day,
                hour,
                minute,
                second,
                micro,
                Some(py_tz),
            )
            .unwrap();
            let py_datetime: DateTime<Utc> = py_datetime.extract().unwrap();
            let datetime = NaiveDate::from_ymd_opt(year, month.into(), day.into())
                .unwrap()
                .and_hms_micro_opt(hour.into(), minute.into(), second.into(), micro)
                .unwrap()
                .and_utc();
            assert_eq!(py_datetime, datetime,);
        })
    }

    #[test]
    fn test_pyo3_datetime_frompyobject_fixed_offset() {
        Python::with_gil(|py| {
            let year = 2014;
            let month = 5;
            let day = 6;
            let hour = 7;
            let minute = 8;
            let second = 9;
            let micro = 999_999;
            let offset = FixedOffset::east_opt(3600).unwrap();
            let py_tz = offset.to_object(py);
            let py_tz = py_tz.downcast(py).unwrap();
            let py_datetime = PyDateTime::new(
                py,
                year,
                month,
                day,
                hour,
                minute,
                second,
                micro,
                Some(py_tz),
            )
            .unwrap();
            let datetime_from_py: DateTime<FixedOffset> = py_datetime.extract().unwrap();
            let datetime = NaiveDate::from_ymd_opt(year, month.into(), day.into())
                .unwrap()
                .and_hms_micro_opt(hour.into(), minute.into(), second.into(), micro)
                .unwrap();
            let datetime = datetime.and_local_timezone(offset).unwrap();

            assert_eq!(datetime_from_py, datetime);
            assert!(
                py_datetime.extract::<DateTime<Utc>>().is_err(),
                "Extracting Utc from nonzero FixedOffset timezone will fail"
            );

            let utc = timezone_utc(py);
            let py_datetime_utc =
                PyDateTime::new(py, year, month, day, hour, minute, second, micro, Some(utc))
                    .unwrap();
            assert!(
                py_datetime_utc.extract::<DateTime<FixedOffset>>().is_ok(),
                "Extracting FixedOffset from Utc timezone will succeed"
            );
        })
    }

    #[test]
    fn test_pyo3_offset_fixed_topyobject() {
        Python::with_gil(|py| {
            // Chrono offset
            let offset = FixedOffset::east_opt(3600).unwrap().to_object(py);
            // Python timezone from timedelta
            let td = PyDelta::new(py, 0, 3600, 0, true).unwrap();
            let py_timedelta = py_timezone_from_offset(&py, td);
            // Should be equal
            assert!(offset.as_ref(py).eq(py_timedelta).unwrap());

            // Same but with negative values
            let offset = FixedOffset::east_opt(-3600).unwrap().to_object(py);
            let td = PyDelta::new(py, 0, -3600, 0, true).unwrap();
            let py_timedelta = py_timezone_from_offset(&py, td);
            assert!(offset.as_ref(py).eq(py_timedelta).unwrap());
        })
    }

    #[test]
    fn test_pyo3_offset_fixed_frompyobject() {
        Python::with_gil(|py| {
            let py_timedelta = PyDelta::new(py, 0, 3600, 0, true).unwrap();
            let py_tzinfo = py_timezone_from_offset(&py, py_timedelta);
            let offset: FixedOffset = py_tzinfo.extract().unwrap();
            assert_eq!(FixedOffset::east_opt(3600).unwrap(), offset);
        })
    }

    #[test]
    fn test_pyo3_offset_utc_topyobject() {
        Python::with_gil(|py| {
            let utc = Utc.to_object(py);
            let py_utc = timezone_utc(py);
            assert!(utc.as_ref(py).is(py_utc));
        })
    }

    #[test]
    fn test_pyo3_offset_utc_frompyobject() {
        Python::with_gil(|py| {
            let py_utc = timezone_utc(py);
            let py_utc: Utc = py_utc.extract().unwrap();
            assert_eq!(Utc, py_utc);

            let py_timedelta = PyDelta::new(py, 0, 0, 0, true).unwrap();
            let py_timezone_utc = py_timezone_from_offset(&py, py_timedelta);
            let py_timezone_utc: Utc = py_timezone_utc.extract().unwrap();
            assert_eq!(Utc, py_timezone_utc);

            let py_timedelta = PyDelta::new(py, 0, 3600, 0, true).unwrap();
            let py_timezone = py_timezone_from_offset(&py, py_timedelta);
            assert!(py_timezone.extract::<Utc>().is_err());
        })
    }

    #[test]
    fn test_pyo3_time_topyobject() {
        Python::with_gil(|py| {
            let check_time = |name: &'static str, hour, minute, second, ms, py_ms| {
                let time = NaiveTime::from_hms_micro_opt(hour, minute, second, ms)
                    .unwrap()
                    .to_object(py);
                let time: &PyTime = time.extract(py).unwrap();
                let py_time =
                    PyTime::new(py, hour as u8, minute as u8, second as u8, py_ms, None).unwrap();
                assert!(
                    time.eq(py_time).unwrap(),
                    "{}: {} != {}",
                    name,
                    time,
                    py_time
                );
            };

            check_time("regular", 3, 5, 7, 999_999, 999_999);

            assert_warnings!(
                py,
                check_time("leap second", 3, 5, 59, 1_999_999, 999_999),
                [(
                    PyUserWarning,
                    "ignored leap-second, `datetime` does not support leap-seconds"
                )]
            );
        })
    }

    #[test]
    fn test_pyo3_time_frompyobject() {
        let hour = 3;
        let minute = 5;
        let second = 7;
        let micro = 999_999;
        Python::with_gil(|py| {
            let py_time =
                PyTime::new(py, hour as u8, minute as u8, second as u8, micro, None).unwrap();
            let py_time: NaiveTime = py_time.extract().unwrap();
            let time = NaiveTime::from_hms_micro_opt(hour, minute, second, micro).unwrap();
            assert_eq!(py_time, time);
        })
    }

    #[cfg(all(test, not(target_arch = "wasm32")))]
    mod proptests {
        use super::*;
        use crate::types::IntoPyDict;

        use proptest::prelude::*;

        proptest! {

            // Range is limited to 1970 to 2038 due to windows limitations
            #[test]
            fn test_pyo3_offset_fixed_frompyobject_created_in_python(timestamp in 0..(i32::MAX as i64), timedelta in -86399i32..=86399i32) {
                Python::with_gil(|py| {

                    let globals = [("datetime", py.import("datetime").unwrap())].into_py_dict(py);
                    let code = format!("datetime.datetime.fromtimestamp({}).replace(tzinfo=datetime.timezone(datetime.timedelta(seconds={})))", timestamp, timedelta);
                    let t = py.eval(&code, Some(globals), None).unwrap();

                    // Get ISO 8601 string from python
                    let py_iso_str = t.call_method0("isoformat").unwrap();

                    // Get ISO 8601 string from rust
                    let t = t.extract::<DateTime<FixedOffset>>().unwrap();
                    // Python doesn't print the seconds of the offset if they are 0
                    let rust_iso_str = if timedelta % 60 == 0 {
                        t.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
                    } else {
                        t.format("%Y-%m-%dT%H:%M:%S%::z").to_string()
                    };

                    // They should be equal
                    assert_eq!(py_iso_str.to_string(), rust_iso_str);
                })
            }

            #[test]
            fn test_duration_roundtrip(days in -999999999i64..=999999999i64) {
                // Test roundtrip conversion rust->python->rust for all allowed
                // python values of durations (from -999999999 to 999999999 days),
                Python::with_gil(|py| {
                    let dur = Duration::days(days);
                    let py_delta = dur.into_py(py);
                    let roundtripped: Duration = py_delta.extract(py).expect("Round trip");
                    assert_eq!(dur, roundtripped);
                })
            }

            #[test]
            fn test_fixed_offset_roundtrip(secs in -86399i32..=86399i32) {
                Python::with_gil(|py| {
                    let offset = FixedOffset::east_opt(secs).unwrap();
                    let py_offset = offset.into_py(py);
                    let roundtripped: FixedOffset = py_offset.extract(py).expect("Round trip");
                    assert_eq!(offset, roundtripped);
                })
            }

            #[test]
            fn test_naive_date_roundtrip(
                year in 1i32..=9999i32,
                month in 1u32..=12u32,
                day in 1u32..=31u32
            ) {
                // Test roundtrip conversion rust->python->rust for all allowed
                // python dates (from year 1 to year 9999)
                Python::with_gil(|py| {
                    // We use to `from_ymd_opt` constructor so that we only test valid `NaiveDate`s.
                    // This is to skip the test if we are creating an invalid date, like February 31.
                    if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                        let py_date = date.to_object(py);
                        let roundtripped: NaiveDate = py_date.extract(py).expect("Round trip");
                        assert_eq!(date, roundtripped);
                    }
                })
            }

            #[test]
            fn test_naive_time_roundtrip(
                hour in 0u32..=23u32,
                min in 0u32..=59u32,
                sec in 0u32..=59u32,
                micro in 0u32..=1_999_999u32
            ) {
                // Test roundtrip conversion rust->python->rust for naive times.
                // Python time has a resolution of microseconds, so we only test
                // NaiveTimes with microseconds resolution, even if NaiveTime has nanosecond
                // resolution.
                Python::with_gil(|py| {
                    if let Some(time) = NaiveTime::from_hms_micro_opt(hour, min, sec, micro) {
                        // Wrap in CatchWarnings to avoid to_object firing warning for truncated leap second
                        let py_time = CatchWarnings::enter(py, |_| Ok(time.to_object(py))).unwrap();
                        let roundtripped: NaiveTime = py_time.extract(py).expect("Round trip");
                        // Leap seconds are not roundtripped
                        let expected_roundtrip_time = micro.checked_sub(1_000_000).map(|micro| NaiveTime::from_hms_micro_opt(hour, min, sec, micro).unwrap()).unwrap_or(time);
                        assert_eq!(expected_roundtrip_time, roundtripped);
                    }
                })
            }

            #[test]
            fn test_naive_datetime_roundtrip(
                year in 1i32..=9999i32,
                month in 1u32..=12u32,
                day in 1u32..=31u32,
                hour in 0u32..=24u32,
                min in 0u32..=60u32,
                sec in 0u32..=60u32,
                micro in 0u32..=999_999u32
            ) {
                Python::with_gil(|py| {
                    let date_opt = NaiveDate::from_ymd_opt(year, month, day);
                    let time_opt = NaiveTime::from_hms_micro_opt(hour, min, sec, micro);
                    if let (Some(date), Some(time)) = (date_opt, time_opt) {
                        let dt = NaiveDateTime::new(date, time);
                        let pydt = dt.to_object(py);
                        let roundtripped: NaiveDateTime = pydt.extract(py).expect("Round trip");
                        assert_eq!(dt, roundtripped);
                    }
                })
            }

            #[test]
            fn test_utc_datetime_roundtrip(
                year in 1i32..=9999i32,
                month in 1u32..=12u32,
                day in 1u32..=31u32,
                hour in 0u32..=23u32,
                min in 0u32..=59u32,
                sec in 0u32..=59u32,
                micro in 0u32..=1_999_999u32
            ) {
                Python::with_gil(|py| {
                    let date_opt = NaiveDate::from_ymd_opt(year, month, day);
                    let time_opt = NaiveTime::from_hms_micro_opt(hour, min, sec, micro);
                    if let (Some(date), Some(time)) = (date_opt, time_opt) {
                        let dt: DateTime<Utc> = NaiveDateTime::new(date, time).and_utc();
                        // Wrap in CatchWarnings to avoid into_py firing warning for truncated leap second
                        let py_dt = CatchWarnings::enter(py, |_| Ok(dt.into_py(py))).unwrap();
                        let roundtripped: DateTime<Utc> = py_dt.extract(py).expect("Round trip");
                        // Leap seconds are not roundtripped
                        let expected_roundtrip_time = micro.checked_sub(1_000_000).map(|micro| NaiveTime::from_hms_micro_opt(hour, min, sec, micro).unwrap()).unwrap_or(time);
                        let expected_roundtrip_dt: DateTime<Utc> = NaiveDateTime::new(date, expected_roundtrip_time).and_utc();
                        assert_eq!(expected_roundtrip_dt, roundtripped);
                    }
                })
            }

            #[test]
            fn test_fixed_offset_datetime_roundtrip(
                year in 1i32..=9999i32,
                month in 1u32..=12u32,
                day in 1u32..=31u32,
                hour in 0u32..=23u32,
                min in 0u32..=59u32,
                sec in 0u32..=59u32,
                micro in 0u32..=1_999_999u32,
                offset_secs in -86399i32..=86399i32
            ) {
                Python::with_gil(|py| {
                    let date_opt = NaiveDate::from_ymd_opt(year, month, day);
                    let time_opt = NaiveTime::from_hms_micro_opt(hour, min, sec, micro);
                    let offset = FixedOffset::east_opt(offset_secs).unwrap();
                    if let (Some(date), Some(time)) = (date_opt, time_opt) {
                        let dt: DateTime<FixedOffset> = NaiveDateTime::new(date, time).and_local_timezone(offset).unwrap();
                        // Wrap in CatchWarnings to avoid into_py firing warning for truncated leap second
                        let py_dt = CatchWarnings::enter(py, |_| Ok(dt.into_py(py))).unwrap();
                        let roundtripped: DateTime<FixedOffset> = py_dt.extract(py).expect("Round trip");
                        // Leap seconds are not roundtripped
                        let expected_roundtrip_time = micro.checked_sub(1_000_000).map(|micro| NaiveTime::from_hms_micro_opt(hour, min, sec, micro).unwrap()).unwrap_or(time);
                        let expected_roundtrip_dt: DateTime<FixedOffset> = NaiveDateTime::new(date, expected_roundtrip_time).and_local_timezone(offset).unwrap();
                        assert_eq!(expected_roundtrip_dt, roundtripped);
                    }
                })
            }
        }
    }
}
