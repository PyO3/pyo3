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
// workaround for `extended_key_value_attributes`: https://github.com/rust-lang/rust/issues/82768#issuecomment-803935643
#![cfg_attr(docsrs, cfg_attr(docsrs, doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"chrono\"] }")))]
#![cfg_attr(
    not(docsrs),
    doc = "pyo3 = { version = \"*\", features = [\"chrono\"] }"
)]
//! ```
//!
//! Note that you must use compatible versions of chrono and PyO3.
//! The required chrono version may vary based on the version of PyO3.
//!
//! # Example: Convert a PyDateTime to chrono's DateTime<Utc>
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
//!         let py_tz = py_tz.cast_as(py).unwrap();
//!         let pydatetime = PyDateTime::new(py, 2022, 1, 1, 12, 0, 0, 0, Some(py_tz)).unwrap();
//!         println!("PyDateTime: {}", pydatetime);
//!         // Now convert it to chrono's DateTime<Utc>
//!         let chrono_datetime: DateTime<Utc> = pydatetime.extract().unwrap();
//!         println!("DateTime<Utc>: {}", chrono_datetime);
//!     });
//! }
//! ```
use crate::exceptions::PyTypeError;
use crate::types::{
    timezone_utc, PyDate, PyDateAccess, PyDateTime, PyDelta, PyDeltaAccess, PyTime, PyTimeAccess,
    PyTzInfo, PyTzInfoAccess,
};
use crate::{
    AsPyPointer, FromPyObject, IntoPy, PyAny, PyObject, PyResult, PyTryFrom, Python, ToPyObject,
};
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
            .expect("Failed to construct delta");
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
        let delta = <PyDelta as PyTryFrom>::try_from(ob)?;
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
        // This cast should be safe, right?
        let month = self.month() as u8;
        let day = self.day() as u8;
        let date = PyDate::new(py, self.year(), month, day).expect("Failed to construct date");
        date.into()
    }
}

impl IntoPy<PyObject> for NaiveDate {
    fn into_py(self, py: Python<'_>) -> PyObject {
        ToPyObject::to_object(&self, py)
    }
}

impl FromPyObject<'_> for NaiveDate {
    fn extract(ob: &PyAny) -> PyResult<NaiveDate> {
        let date = <PyDate as PyTryFrom>::try_from(ob)?;
        Ok(NaiveDate::from_ymd(
            date.get_year(),
            date.get_month() as u32,
            date.get_day() as u32,
        ))
    }
}

impl ToPyObject for NaiveTime {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let h = self.hour() as u8;
        let m = self.minute() as u8;
        let s = self.second() as u8;
        let ns = self.nanosecond();
        let (ms, fold) = match ns.checked_sub(1_000_000_000) {
            Some(ns) => (ns / 1000, true),
            None => (ns / 1000, false),
        };
        let time =
            PyTime::new_with_fold(py, h, m, s, ms, None, fold).expect("Failed to construct time");
        time.into()
    }
}

impl IntoPy<PyObject> for NaiveTime {
    fn into_py(self, py: Python<'_>) -> PyObject {
        ToPyObject::to_object(&self, py)
    }
}

impl FromPyObject<'_> for NaiveTime {
    fn extract(ob: &PyAny) -> PyResult<NaiveTime> {
        let time = <PyTime as PyTryFrom>::try_from(ob)?;
        let ms = time.get_fold() as u32 * 1_000_000 + time.get_microsecond();
        let h = time.get_hour() as u32;
        let m = time.get_minute() as u32;
        let s = time.get_second() as u32;
        Ok(NaiveTime::from_hms_micro(h, m, s, ms))
    }
}

impl<Tz: TimeZone> ToPyObject for DateTime<Tz> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let date = self.naive_utc().date();
        let time = self.naive_utc().time();
        let yy = date.year();
        let mm = date.month() as u8;
        let dd = date.day() as u8;
        let h = time.hour() as u8;
        let m = time.minute() as u8;
        let s = time.second() as u8;
        let ns = time.nanosecond();
        let (ms, fold) = match ns.checked_sub(1_000_000_000) {
            Some(ns) => (ns / 1000, true),
            None => (ns / 1000, false),
        };
        let tz = self.offset().fix().to_object(py);
        let tz = tz.cast_as(py).unwrap();
        let datetime = PyDateTime::new_with_fold(py, yy, mm, dd, h, m, s, ms, Some(tz), fold)
            .expect("Failed to construct datetime");
        datetime.into()
    }
}

impl<Tz: TimeZone> IntoPy<PyObject> for DateTime<Tz> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        ToPyObject::to_object(&self, py)
    }
}

impl FromPyObject<'_> for DateTime<FixedOffset> {
    fn extract(ob: &PyAny) -> PyResult<DateTime<FixedOffset>> {
        let dt = <PyDateTime as PyTryFrom>::try_from(ob)?;
        let ms = dt.get_fold() as u32 * 1_000_000 + dt.get_microsecond();
        let h = dt.get_hour().into();
        let m = dt.get_minute().into();
        let s = dt.get_second().into();
        let tz = if let Some(tzinfo) = dt.get_tzinfo() {
            tzinfo.extract()?
        } else {
            return Err(PyTypeError::new_err("Not datetime.timezone.tzinfo"));
        };
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd(dt.get_year(), dt.get_month().into(), dt.get_day().into()),
            NaiveTime::from_hms_micro(h, m, s, ms),
        );
        Ok(DateTime::from_utc(dt, tz))
    }
}

impl FromPyObject<'_> for DateTime<Utc> {
    fn extract(ob: &PyAny) -> PyResult<DateTime<Utc>> {
        let dt = <PyDateTime as PyTryFrom>::try_from(ob)?;
        let ms = dt.get_fold() as u32 * 1_000_000 + dt.get_microsecond();
        let h = dt.get_hour().into();
        let m = dt.get_minute().into();
        let s = dt.get_second().into();
        let tz = if let Some(tzinfo) = dt.get_tzinfo() {
            tzinfo.extract()?
        } else {
            return Err(PyTypeError::new_err("Not datetime.timezone.utc"));
        };
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd(dt.get_year(), dt.get_month().into(), dt.get_day().into()),
            NaiveTime::from_hms_micro(h, m, s, ms),
        );
        Ok(DateTime::from_utc(dt, tz))
    }
}

// Utiliy function used to convert PyDelta to timezone
fn pytimezone_fromoffset<'a>(py: &Python<'a>, td: &PyDelta) -> &'a PyAny {
    // Safety: py.from_borrowed_ptr needs the cast to be valid.
    // Since we are forcing a &PyDelta as input, the cast should always be valid.
    unsafe {
        PyDateTime_IMPORT();
        py.from_borrowed_ptr(PyTimeZone_FromOffset(td.as_ptr()))
    }
}

impl ToPyObject for FixedOffset {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let seconds_offset = self.local_minus_utc();
        // XXX: Here we don't normalize, otherwise the meaning
        // of the offset changes. Normalizing a negative PyDelta and converting
        // back to rust would transform a `-00:00:01` offset into `23:59:59`.
        let td =
            PyDelta::new(py, 0, seconds_offset, 0, false).expect("Failed to contruct timedelta");
        pytimezone_fromoffset(&py, td).into()
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
        let py_tzinfo = <PyTzInfo as PyTryFrom>::try_from(ob)?;
        let py_timedelta = py_tzinfo.call_method1("utcoffset", (ob.py().None(),))?;
        let py_timedelta = <PyDelta as PyTryFrom>::try_from(py_timedelta)?;
        Ok(FixedOffset::east(py_timedelta.get_seconds()))
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
        let py_tzinfo = <PyTzInfo as PyTryFrom>::try_from(ob)?;
        let py_utc = timezone_utc(ob.py());
        if py_tzinfo.eq(py_utc)? {
            Ok(Utc)
        } else {
            Err(PyTypeError::new_err("Not datetime.timezone.utc"))
        }
    }
}

#[cfg(test)]
mod proptests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    use proptest::prelude::*;

    #[cfg(not(target_arch = "wasm32"))]
    proptest! {
        #[test]
        fn test_duration_roundtrip(days in -999999999i64..=999999999i64) {
            // Test roundtrip convertion rust->python->rust for all allowed
            // python values of durations (from -999999999 to 999999999 days),
            Python::with_gil(|py| {
                let dur = Duration::days(days);
                let pydelta = dur.into_py(py);
                let roundtripped: Duration = pydelta.extract(py).expect("Round trip");
                assert_eq!(dur, roundtripped);
            })
        }

        #[test]
        fn test_fixedoffset_roundtrip(secs in -86_400i32..=86_400i32) {
            Python::with_gil(|py| {
                let offset = FixedOffset::east(secs);
                let pyoffset = offset.into_py(py);
                let roundtripped: FixedOffset = pyoffset.extract(py).expect("Round trip");
                assert_eq!(offset, roundtripped);
            })
        }

        #[test]
        fn test_naivedate_roundtrip(
            year in 1i32..=9999i32,
            month in 1u32..=12u32,
            day in 1u32..=31u32
        ) {
            // Test roundtrip convertion rust->python->rust for all allowed
            // python dates (from year 1 to year 9999)
            Python::with_gil(|py| {
                // We use to `from_ymd_opt` constructor so that we only test valid `NaiveDate`s.
                // This is to skip the test if we are creating an invalid date, like February 31.
                if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                    let pydate = date.into_py(py);
                    let roundtripped: NaiveDate = pydate.extract(py).expect("Round trip");
                    assert_eq!(date, roundtripped);
                }
            })
        }

        #[test]
        fn test_naivetime_roundtrip(
            hour in 0u32..=24u32,
            min in 0u32..=60u32,
            sec in 0u32..=60u32,
            micro in 0u32..=2_000_000u32
        ) {
            // Test roundtrip convertion rust->python->rust for naive times.
            // Python time has a resolution of microseconds, so we only test
            // NaiveTimes with microseconds resolution, even if NaiveTime has nanosecond
            // resolution.
            Python::with_gil(|py| {
                // We use to `from_hms_micro_opt` constructor so that we only test valid `NaiveTime`s.
                // This is to skip the test if we are creating an invalid time
                if let Some(time) = NaiveTime::from_hms_micro_opt(hour, min, sec, micro) {
                    let pytime = time.into_py(py);
                    let roundtripped: NaiveTime = pytime.extract(py).expect("Round trip");
                    assert_eq!(time, roundtripped);
                }
            })
        }

        #[test]
        fn test_utc_datetime_roundtrip(
            year in 1i32..=9999i32,
            month in 1u32..=12u32,
            day in 1u32..=31u32,
            hour in 0u32..=24u32,
            min in 0u32..=60u32,
            sec in 0u32..=60u32,
            micro in 0u32..=2_000_000u32
        ) {
            Python::with_gil(|py| {
                let date_opt = NaiveDate::from_ymd_opt(year, month, day);
                let time_opt = NaiveTime::from_hms_micro_opt(hour, min, sec, micro);
                if let (Some(date), Some(time)) = (date_opt, time_opt) {
                    let dt: DateTime<Utc> = DateTime::from_utc(NaiveDateTime::new(date, time), Utc);
                    let pydt = dt.into_py(py);
                    let roundtripped: DateTime<Utc> = pydt.extract(py).expect("Round trip");
                    assert_eq!(dt, roundtripped);
                }
            })
        }

        #[test]
        fn test_fixedoffset_datetime_roundtrip(
            year in 1i32..=9999i32,
            month in 1u32..=12u32,
            day in 1u32..=31u32,
            hour in 0u32..=24u32,
            min in 0u32..=60u32,
            sec in 0u32..=60u32,
            micro in 0u32..=2_000_000u32,
            offset_secs in -86_400i32..=86_400i32
        ) {
            Python::with_gil(|py| {
                let date_opt = NaiveDate::from_ymd_opt(year, month, day);
                let time_opt = NaiveTime::from_hms_micro_opt(hour, min, sec, micro);
                let offset = FixedOffset::east(offset_secs);
                if let (Some(date), Some(time)) = (date_opt, time_opt) {
                    let dt: DateTime<FixedOffset> = DateTime::from_utc(NaiveDateTime::new(date, time), offset);
                    let pydt = dt.into_py(py);
                    let roundtripped: DateTime<FixedOffset> = pydt.extract(py).expect("Round trip");
                    assert_eq!(dt, roundtripped);
                }
            })
        }
    }
}

#[cfg(test)]
mod test_chrono {
    use crate::chrono::pytimezone_fromoffset;
    use crate::types::*;
    use crate::{Python, ToPyObject};
    use chrono::offset::{FixedOffset, Utc};
    use chrono::{DateTime, Duration, NaiveDate, NaiveTime};
    use std::cmp::Ordering;
    use std::panic;

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
                let py_delta = PyDelta::new(py, py_days, py_seconds, py_ms, false).unwrap();
                let py_delta: Duration = py_delta.extract().unwrap();
                assert_eq!(py_delta, delta, "{}: {} != {}", name, py_delta, delta);
            })
        };

        // Check the minimum value allowed by PyDelta, which is different
        // from the minimum value allowed in Duration. This should pass.
        check(
            "min pydelta value",
            Duration::seconds(-86399999913600),
            -999999999,
            0,
            0,
        );
        // Same, for max value
        check(
            "max pydelta value",
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
                let pydelta = PyDelta::new(py, low_days, 0, 0, true).unwrap();
                if let Ok(_duration) = pydelta.extract::<Duration>() {
                    // So we should never get here
                }
            })
            .is_err());

            let high_days: i32 = 1000000000;
            // This is possible
            assert!(panic::catch_unwind(|| Duration::days(high_days as i64)).is_ok());
            // This panics on PyDelta::new
            assert!(panic::catch_unwind(|| {
                let pydelta = PyDelta::new(py, high_days, 0, 0, true).unwrap();
                if let Ok(_duration) = pydelta.extract::<Duration>() {
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
                let date = NaiveDate::from_ymd(year, month, day).to_object(py);
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
                let date = NaiveDate::from_ymd(year, month, day);
                assert_eq!(py_date, date, "{}: {} != {}", name, date, py_date);
            })
        };

        eq_ymd("past date", 2012, 2, 29);
        eq_ymd("min date", 1, 1, 1);
        eq_ymd("future date", 3000, 6, 5);
        eq_ymd("max date", 9999, 12, 31);
    }

    #[test]
    fn test_pyo3_datetime_topyobject() {
        let check_utc =
            |name: &'static str, year, month, day, hour, minute, second, ms, py_ms, fold| {
                Python::with_gil(|py| {
                    let datetime = NaiveDate::from_ymd(year, month, day)
                        .and_hms_micro(hour, minute, second, ms);
                    let datetime = DateTime::<Utc>::from_utc(datetime, Utc).to_object(py);
                    let datetime: &PyDateTime = datetime.extract(py).unwrap();
                    let py_tz = Utc.to_object(py);
                    let py_tz = py_tz.cast_as(py).unwrap();
                    let py_datetime = PyDateTime::new_with_fold(
                        py,
                        year,
                        month as u8,
                        day as u8,
                        hour as u8,
                        minute as u8,
                        second as u8,
                        py_ms,
                        Some(py_tz),
                        fold,
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
                })
            };

        check_utc("fold", 2014, 5, 6, 7, 8, 9, 1_999_999, 999_999, true);
        check_utc("non fold", 2014, 5, 6, 7, 8, 9, 999_999, 999_999, false);

        let check_fixed_offset =
            |name: &'static str, year, month, day, hour, minute, ssecond, ms, py_ms, fold| {
                Python::with_gil(|py| {
                    let offset = FixedOffset::east(3600);
                    let datetime = NaiveDate::from_ymd(year, month, day)
                        .and_hms_micro(hour, minute, ssecond, ms);
                    let datetime =
                        DateTime::<FixedOffset>::from_utc(datetime, offset).to_object(py);
                    let datetime: &PyDateTime = datetime.extract(py).unwrap();
                    let py_tz = offset.to_object(py);
                    let py_tz = py_tz.cast_as(py).unwrap();
                    let py_datetime = PyDateTime::new_with_fold(
                        py,
                        year,
                        month as u8,
                        day as u8,
                        hour as u8,
                        minute as u8,
                        ssecond as u8,
                        py_ms,
                        Some(py_tz),
                        fold,
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
                })
            };

        check_fixed_offset("fold", 2014, 5, 6, 7, 8, 9, 1_999_999, 999_999, true);
        check_fixed_offset("non fold", 2014, 5, 6, 7, 8, 9, 999_999, 999_999, false);
    }

    #[test]
    fn test_pyo3_datetime_frompyobject() {
        let check_utc =
            |name: &'static str, year, month, day, hour, minute, second, ms, py_ms, fold| {
                Python::with_gil(|py| {
                    let py_tz = Utc.to_object(py);
                    let py_tz = py_tz.cast_as(py).unwrap();
                    let py_datetime = PyDateTime::new_with_fold(
                        py,
                        year as i32,
                        month as u8,
                        day as u8,
                        hour as u8,
                        minute as u8,
                        second as u8,
                        py_ms,
                        Some(py_tz),
                        fold,
                    )
                    .unwrap();
                    let py_datetime: DateTime<Utc> = py_datetime.extract().unwrap();
                    let datetime = NaiveDate::from_ymd(year, month, day)
                        .and_hms_micro(hour, minute, second, ms);
                    let datetime = DateTime::<Utc>::from_utc(datetime, Utc);
                    assert_eq!(
                        py_datetime, datetime,
                        "{}: {} != {}",
                        name, datetime, py_datetime
                    );
                })
            };

        check_utc("fold", 2014, 5, 6, 7, 8, 9, 1_999_999, 999_999, true);
        check_utc("non fold", 2014, 5, 6, 7, 8, 9, 999_999, 999_999, false);

        let check_fixed_offset =
            |name: &'static str, year, month, day, hour, minute, second, ms, py_ms, fold| {
                Python::with_gil(|py| {
                    let offset = FixedOffset::east(3600);
                    let py_tz = offset.to_object(py);
                    let py_tz = py_tz.cast_as(py).unwrap();
                    let py_datetime = PyDateTime::new_with_fold(
                        py,
                        year as i32,
                        month as u8,
                        day as u8,
                        hour as u8,
                        minute as u8,
                        second as u8,
                        py_ms,
                        Some(py_tz),
                        fold,
                    )
                    .unwrap();
                    let py_datetime: DateTime<FixedOffset> = py_datetime.extract().unwrap();
                    let datetime = NaiveDate::from_ymd(year, month, day)
                        .and_hms_micro(hour, minute, second, ms);
                    let datetime = DateTime::<FixedOffset>::from_utc(datetime, offset);
                    assert_eq!(
                        py_datetime, datetime,
                        "{}: {} != {}",
                        name, datetime, py_datetime
                    );
                })
            };

        check_fixed_offset("fold", 2014, 5, 6, 7, 8, 9, 1_999_999, 999_999, true);
        check_fixed_offset("non fold", 2014, 5, 6, 7, 8, 9, 999_999, 999_999, false);

        Python::with_gil(|py| {
            // extract utc with fixedoffset should fail
            // but fixedoffset from utc seemed to work, maybe because it is also considered fixedoffset?
            let py_tz = Utc.to_object(py);
            let py_tz = py_tz.cast_as(py).unwrap();
            let py_datetime =
                PyDateTime::new_with_fold(py, 2014, 5, 6, 7, 8, 9, 999_999, Some(py_tz), false)
                    .unwrap();
            assert!(py_datetime.extract::<DateTime<FixedOffset>>().is_ok());
            let offset = FixedOffset::east(3600);
            let py_tz = offset.to_object(py);
            let py_tz = py_tz.cast_as(py).unwrap();
            let py_datetime =
                PyDateTime::new_with_fold(py, 2014, 5, 6, 7, 8, 9, 999_999, Some(py_tz), false)
                    .unwrap();
            assert!(py_datetime.extract::<DateTime<Utc>>().is_err());
        })
    }

    #[test]
    fn test_pyo3_offset_fixed_topyobject() {
        Python::with_gil(|py| {
            // Chrono offset
            let offset = FixedOffset::east(3600).to_object(py);
            // Python timezone from timedelta
            let td = PyDelta::new(py, 0, 3600, 0, true).unwrap();
            let py_timedelta = pytimezone_fromoffset(&py, td);
            // Should be equal
            assert!(offset.as_ref(py).eq(py_timedelta).unwrap());

            // Same but with negative values
            let offset = FixedOffset::east(-3600).to_object(py);
            let td = PyDelta::new(py, 0, -3600, 0, false).unwrap();
            let py_timedelta = pytimezone_fromoffset(&py, td);
            assert!(offset.as_ref(py).eq(py_timedelta).unwrap());
        })
    }

    #[test]
    fn test_pyo3_offset_fixed_frompyobject() {
        Python::with_gil(|py| {
            let py_timedelta = PyDelta::new(py, 0, 3600, 0, true).unwrap();
            let py_tzinfo = pytimezone_fromoffset(&py, py_timedelta);
            let offset: FixedOffset = py_tzinfo.extract().unwrap();
            assert_eq!(FixedOffset::east(3600), offset);
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

            let py_timedelta = PyDelta::new(py, 0, 0, 0, false).unwrap();
            let py_timezone_utc = pytimezone_fromoffset(&py, py_timedelta);
            let py_timezone_utc: Utc = py_timezone_utc.extract().unwrap();
            assert_eq!(Utc, py_timezone_utc);

            let py_timedelta = PyDelta::new(py, 0, 3600, 0, false).unwrap();
            let py_timezone = pytimezone_fromoffset(&py, py_timedelta);
            assert!(py_timezone.extract::<Utc>().is_err());
        })
    }

    #[test]
    fn test_pyo3_time_topyobject() {
        let check_time = |name: &'static str, hour, minute, second, ms, py_ms, fold| {
            Python::with_gil(|py| {
                let time = NaiveTime::from_hms_micro(hour, minute, second, ms).to_object(py);
                let time: &PyTime = time.extract(py).unwrap();
                let py_time = PyTime::new_with_fold(
                    py,
                    hour as u8,
                    minute as u8,
                    second as u8,
                    py_ms,
                    None,
                    fold,
                )
                .unwrap();
                assert_eq!(
                    time.compare(py_time).unwrap(),
                    Ordering::Equal,
                    "{}: {} != {}",
                    name,
                    time,
                    py_time
                );
            })
        };

        check_time("fold", 3, 5, 7, 1_999_999, 999_999, true);
        check_time("non fold", 3, 5, 7, 999_999, 999_999, false);
    }

    #[test]
    fn test_pyo3_time_frompyobject() {
        let check_time = |name: &'static str, hour, minute, second, ms, py_ms, fold| {
            Python::with_gil(|py| {
                let py_time = PyTime::new_with_fold(
                    py,
                    hour as u8,
                    minute as u8,
                    second as u8,
                    py_ms,
                    None,
                    fold,
                )
                .unwrap();
                let py_time: NaiveTime = py_time.extract().unwrap();
                let time = NaiveTime::from_hms_micro(hour, minute, second, ms);
                assert_eq!(py_time, time, "{}: {} != {}", name, py_time, time);
            })
        };

        check_time("fold", 3, 5, 7, 1_999_999, 999_999, true);
        check_time("non fold", 3, 5, 7, 999_999, 999_999, false);
    }
}
