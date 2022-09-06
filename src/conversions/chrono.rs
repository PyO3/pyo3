#![cfg(feature = "chrono")]

//! Conversions to and from [chrono](https://docs.rs/chrono/)’s `TimeDelta`,
//! `NaiveDate`, `NaiveTime`, `DateTime<Tz>`, `FixedOffset`, and `Utc`.
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! # change * to the latest versions
//! pyo3 = "*"
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
use crate::exceptions::PyTypeError;
use crate::types::{
    timezone_utc, PyDate, PyDateAccess, PyDateTime, PyDelta, PyDeltaAccess, PyTime, PyTimeAccess,
    PyTzInfo, PyTzInfoAccess,
};
use crate::{FromPyObject, IntoPy, PyAny, PyObject, PyResult, PyTryFrom, Python, ToPyObject};
use chrono::offset::{FixedOffset, Utc};
use chrono::{
    DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeDelta, TimeZone, Timelike,
};
use std::convert::TryInto;

const SECS_PER_DAY: i64 = 86_400;
const NANOS_PER_MICRO: i32 = 1000;

impl ToPyObject for TimeDelta {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let days = self.secs / SECS_PER_DAY;
        let secs = self.secs - days * SECS_PER_DAY;
        let micros = self.nanos / NANOS_PER_MICRO;
        // Python will check overflow so even if we reduce the size
        // it will still overflow.
        let days = days.try_into().unwrap_or(i32::MAX);
        let secs = secs.try_into().unwrap_or(i32::MAX);

        // We do not need to check i64 to i32 cast from rust because
        // python will panic with OverflowError.
        // Not sure if we need normalize here.
        let delta = PyDelta::new(py, days, secs, micros, false).expect("Failed to construct delta");
        delta.into()
    }
}

impl IntoPy<PyObject> for TimeDelta {
    fn into_py(self, py: Python<'_>) -> PyObject {
        ToPyObject::to_object(&self, py)
    }
}

impl FromPyObject<'_> for TimeDelta {
    fn extract(ob: &PyAny) -> PyResult<TimeDelta> {
        let delta = <PyDelta as PyTryFrom>::try_from(ob)?;
        // Python size are much lower than rust size so we do not need bound checks.
        // 0 <= microseconds < 1000000
        // 0 <= seconds < 3600*24
        // -999999999 <= days <= 999999999
        let secs = delta.get_days() as i64 * SECS_PER_DAY + delta.get_seconds() as i64;
        let nanos = delta.get_microseconds() * NANOS_PER_MICRO;

        Ok(TimeDelta { secs, nanos })
    }
}

impl ToPyObject for NaiveDate {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let mdf = self.mdf();
        let date = PyDate::new(py, self.year(), mdf.month() as u8, mdf.day() as u8)
            .expect("Failed to construct date");
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
        let (h, m, s) = self.hms();
        let ns = self.nanosecond();
        let (ms, fold) = match ns.checked_sub(1_000_000_000) {
            Some(ns) => (ns / 1000, true),
            None => (ns / 1000, false),
        };
        let time = PyTime::new_with_fold(py, h as u8, m as u8, s as u8, ms, None, fold)
            .expect("Failed to construct time");
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
        let (h, m, s) = (time.get_hour(), time.get_minute(), time.get_second());
        Ok(NaiveTime::from_hms_micro(h as u32, m as u32, s as u32, ms))
    }
}

impl<Tz: TimeZone> ToPyObject for DateTime<Tz> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let (date, time) = (self.naive_utc().date(), self.naive_utc().time());
        let mdf = date.mdf();
        let (yy, mm, dd) = (date.year(), mdf.month(), mdf.day());
        let (h, m, s) = time.hms();
        let ns = time.nanosecond();
        let (ms, fold) = match ns.checked_sub(1_000_000_000) {
            Some(ns) => (ns / 1000, true),
            None => (ns / 1000, false),
        };
        let tz = self.offset().fix().to_object(py);
        let tz = tz.cast_as(py).unwrap();
        let datetime = PyDateTime::new_with_fold(
            py,
            yy,
            mm as u8,
            dd as u8,
            h as u8,
            m as u8,
            s as u8,
            ms,
            Some(&tz),
            fold,
        )
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
        let (h, m, s) = (dt.get_hour(), dt.get_minute(), dt.get_second());
        let tz = if let Some(tzinfo) = dt.get_tzinfo() {
            tzinfo.extract()?
        } else {
            return Err(PyTypeError::new_err("Not datetime.timezone.tzinfo"));
        };
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd(dt.get_year(), dt.get_month() as u32, dt.get_day() as u32),
            NaiveTime::from_hms_micro(h as u32, m as u32, s as u32, ms),
        );
        Ok(DateTime::from_utc(dt, tz))
    }
}

impl FromPyObject<'_> for DateTime<Utc> {
    fn extract(ob: &PyAny) -> PyResult<DateTime<Utc>> {
        let dt = <PyDateTime as PyTryFrom>::try_from(ob)?;
        let ms = dt.get_fold() as u32 * 1_000_000 + dt.get_microsecond();
        let (h, m, s) = (dt.get_hour(), dt.get_minute(), dt.get_second());
        let tz = if let Some(tzinfo) = dt.get_tzinfo() {
            tzinfo.extract()?
        } else {
            return Err(PyTypeError::new_err("Not datetime.timezone.utc"));
        };
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd(dt.get_year(), dt.get_month() as u32, dt.get_day() as u32),
            NaiveTime::from_hms_micro(h as u32, m as u32, s as u32, ms),
        );
        Ok(DateTime::from_utc(dt, tz))
    }
}

impl ToPyObject for FixedOffset {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let dt_module = py.import("datetime").expect("Failed to import datetime");
        let dt_timezone = dt_module
            .getattr("timezone")
            .expect("Failed to getattr timezone");
        let seconds_offset = self.local_minus_utc();
        let td =
            PyDelta::new(py, 0, seconds_offset, 0, true).expect("Failed to contruct timedelta");
        let offset = dt_timezone
            .call1((td,))
            .expect("Failed to call timezone with timedelta");
        offset.into()
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
mod test_chrono {
    // TODO: change Python::acquire_gil to Python::with_gil
    #![allow(deprecated)]
    use crate::types::*;
    use crate::{Python, ToPyObject};
    use chrono::offset::{FixedOffset, Utc};
    use chrono::{DateTime, NaiveDate, NaiveTime, TimeDelta};
    use std::cmp::Ordering;

    #[test]
    fn test_pyo3_timedelta_topyobject() {
        use std::panic;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let check = |s, ns, py_d, py_s, py_ms| {
            let delta = TimeDelta { secs: s, nanos: ns }.to_object(py);
            let delta: &PyDelta = delta.extract(py).unwrap();
            let py_delta = PyDelta::new(py, py_d, py_s, py_ms, true).unwrap();
            assert!(delta.eq(py_delta).unwrap());
        };
        let check_panic = |duration: TimeDelta| {
            assert!(panic::catch_unwind(|| {
                let gil = Python::acquire_gil();
                let py = gil.python();
                duration.to_object(py);
            })
            .is_err())
        };

        check(-86399999913600, 0, -999999999, 0, 0); // min
        check(86399999999999, 999999000, 999999999, 86399, 999999); // max

        check_panic(TimeDelta::MIN);
        check_panic(TimeDelta::MAX);
        // TODO: check timedelta underflow
    }

    #[test]
    fn test_pyo3_timedelta_frompyobject() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let check = |s, ns, py_d, py_s, py_ms| {
            let py_delta = PyDelta::new(py, py_d, py_s, py_ms, false).unwrap();
            let py_delta: TimeDelta = py_delta.extract().unwrap();
            let delta = TimeDelta { secs: s, nanos: ns };
            assert_eq!(py_delta, delta);
        };

        check(-86399999913600, 0, -999999999, 0, 0); // min
        check(86399999999999, 999999000, 999999999, 86399, 999999); // max
    }

    #[test]
    fn test_pyo3_date_topyobject() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let eq_ymd = |y, m, d| {
            let date = NaiveDate::from_ymd(y, m, d).to_object(py);
            let date: &PyDate = date.extract(py).unwrap();
            let py_date = PyDate::new(py, y, m as u8, d as u8).unwrap();
            assert_eq!(date.compare(py_date).unwrap(), Ordering::Equal);
        };

        eq_ymd(2012, 2, 29);
        eq_ymd(1, 1, 1); // min
        eq_ymd(3000, 6, 5); // future
        eq_ymd(9999, 12, 31); // max
    }

    #[test]
    fn test_pyo3_date_frompyobject() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let eq_ymd = |y, m, d| {
            let py_date = PyDate::new(py, y, m as u8, d as u8).unwrap();
            let py_date: NaiveDate = py_date.extract().unwrap();
            let date = NaiveDate::from_ymd(y, m, d);
            assert_eq!(py_date, date);
        };

        eq_ymd(2012, 2, 29);
        eq_ymd(1, 1, 1); // min
        eq_ymd(3000, 6, 5); // future
        eq_ymd(9999, 12, 31); // max
    }

    #[test]
    fn test_pyo3_datetime_topyobject() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let check = |y, mo, d, h, m, s, ms, py_ms, f| {
            let datetime = NaiveDate::from_ymd(y, mo, d).and_hms_micro(h, m, s, ms);
            let datetime = DateTime::<Utc>::from_utc(datetime, Utc).to_object(py);
            let datetime: &PyDateTime = datetime.extract(py).unwrap();
            let py_tz = Utc.to_object(py);
            let py_tz = py_tz.cast_as(py).unwrap();
            let py_datetime = PyDateTime::new_with_fold(
                py,
                y,
                mo as u8,
                d as u8,
                h as u8,
                m as u8,
                s as u8,
                py_ms,
                Some(&py_tz),
                f,
            )
            .unwrap();
            assert_eq!(datetime.compare(py_datetime).unwrap(), Ordering::Equal);
        };

        check(2014, 5, 6, 7, 8, 9, 1_999_999, 999_999, true);
        check(2014, 5, 6, 7, 8, 9, 999_999, 999_999, false);

        let check = |y, mo, d, h, m, s, ms, py_ms, f| {
            let offset = FixedOffset::east(3600);
            let datetime = NaiveDate::from_ymd(y, mo, d).and_hms_micro(h, m, s, ms);
            let datetime = DateTime::<FixedOffset>::from_utc(datetime, offset).to_object(py);
            let datetime: &PyDateTime = datetime.extract(py).unwrap();
            let py_tz = offset.to_object(py);
            let py_tz = py_tz.cast_as(py).unwrap();
            let py_datetime = PyDateTime::new_with_fold(
                py,
                y,
                mo as u8,
                d as u8,
                h as u8,
                m as u8,
                s as u8,
                py_ms,
                Some(&py_tz),
                f,
            )
            .unwrap();
            assert_eq!(datetime.compare(py_datetime).unwrap(), Ordering::Equal);
        };

        check(2014, 5, 6, 7, 8, 9, 1_999_999, 999_999, true);
        check(2014, 5, 6, 7, 8, 9, 999_999, 999_999, false);
    }

    #[test]
    fn test_pyo3_datetime_frompyobject() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let check = |y, mo, d, h, m, s, ms, py_ms, f| {
            let py_tz = Utc.to_object(py);
            let py_tz = py_tz.cast_as(py).unwrap();
            let py_datetime = PyDateTime::new_with_fold(
                py,
                y as i32,
                mo as u8,
                d as u8,
                h as u8,
                m as u8,
                s as u8,
                py_ms,
                Some(&py_tz),
                f,
            )
            .unwrap();
            let py_datetime: DateTime<Utc> = py_datetime.extract().unwrap();
            let datetime = NaiveDate::from_ymd(y, mo, d).and_hms_micro(h, m, s, ms);
            let datetime = DateTime::<Utc>::from_utc(datetime, Utc);
            assert_eq!(py_datetime, datetime);
        };

        check(2014, 5, 6, 7, 8, 9, 1_999_999, 999_999, true);
        check(2014, 5, 6, 7, 8, 9, 999_999, 999_999, false);

        let check = |y, mo, d, h, m, s, ms, py_ms, f| {
            let offset = FixedOffset::east(3600);
            let py_tz = offset.to_object(py);
            let py_tz = py_tz.cast_as(py).unwrap();
            let py_datetime = PyDateTime::new_with_fold(
                py,
                y as i32,
                mo as u8,
                d as u8,
                h as u8,
                m as u8,
                s as u8,
                py_ms,
                Some(&py_tz),
                f,
            )
            .unwrap();
            let py_datetime: DateTime<FixedOffset> = py_datetime.extract().unwrap();
            let datetime = NaiveDate::from_ymd(y, mo, d).and_hms_micro(h, m, s, ms);
            let datetime = DateTime::<FixedOffset>::from_utc(datetime, offset);
            assert_eq!(py_datetime, datetime);
        };

        check(2014, 5, 6, 7, 8, 9, 1_999_999, 999_999, true);
        check(2014, 5, 6, 7, 8, 9, 999_999, 999_999, false);

        // extract utc with fixedoffset should fail
        // but fixedoffset from utc seemed to work, maybe because it is also considered fixedoffset?
        let py_tz = Utc.to_object(py);
        let py_tz = py_tz.cast_as(py).unwrap();
        let py_datetime =
            PyDateTime::new_with_fold(py, 2014, 5, 6, 7, 8, 9, 999_999, Some(&py_tz), false)
                .unwrap();
        assert!(py_datetime.extract::<DateTime<FixedOffset>>().is_ok());
        let offset = FixedOffset::east(3600);
        let py_tz = offset.to_object(py);
        let py_tz = py_tz.cast_as(py).unwrap();
        let py_datetime =
            PyDateTime::new_with_fold(py, 2014, 5, 6, 7, 8, 9, 999_999, Some(&py_tz), false)
                .unwrap();
        assert!(py_datetime.extract::<DateTime<Utc>>().is_err());
    }

    #[test]
    fn test_pyo3_offset_fixed_topyobject() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_module = py.import("datetime").unwrap();
        let py_timezone = py_module.getattr("timezone").unwrap();
        let offset = FixedOffset::east(3600).to_object(py);
        let py_timedelta = PyDelta::new(py, 0, 3600, 0, true).unwrap();
        let py_timedelta = py_timezone.call1((py_timedelta,)).unwrap();
        assert!(offset.as_ref(py).eq(py_timedelta).unwrap());
        let offset = FixedOffset::east(-3600).to_object(py);
        let py_timedelta = PyDelta::new(py, 0, -3600, 0, true).unwrap();
        let py_timedelta = py_timezone.call1((py_timedelta,)).unwrap();
        assert!(offset.as_ref(py).eq(py_timedelta).unwrap());
    }

    #[test]
    fn test_pyo3_offset_fixed_frompyobject() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_module = py.import("datetime").unwrap();
        let py_timezone = py_module.getattr("timezone").unwrap();
        let py_timedelta = PyDelta::new(py, 0, 3600, 0, true).unwrap();
        let py_tzinfo = py_timezone.call1((py_timedelta,)).unwrap();
        let offset: FixedOffset = py_tzinfo.extract().unwrap();
        assert_eq!(FixedOffset::east(3600), offset);
    }

    #[test]
    fn test_pyo3_offset_utc_topyobject() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let utc = Utc.to_object(py);
        let py_module = py.import("datetime").unwrap();
        let py_timezone = py_module.getattr("timezone").unwrap();
        let py_utc = py_timezone.getattr("utc").unwrap();
        assert!(utc.as_ref(py).is(py_utc));
    }

    #[test]
    fn test_pyo3_offset_utc_frompyobject() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_module = py.import("datetime").unwrap();
        let py_timezone = py_module.getattr("timezone").unwrap();
        let py_utc = py_timezone.getattr("utc").unwrap();
        let py_utc: Utc = py_utc.extract().unwrap();
        assert_eq!(Utc, py_utc);
        let py_timedelta = PyDelta::new(py, 0, 0, 0, false).unwrap();
        let py_timezone_utc = py_timezone.call1((py_timedelta,)).unwrap();
        let py_timezone_utc: Utc = py_timezone_utc.extract().unwrap();
        assert_eq!(Utc, py_timezone_utc);
        let py_timedelta = PyDelta::new(py, 0, 3600, 0, false).unwrap();
        let py_timezone = py_timezone.call1((py_timedelta,)).unwrap();
        assert!(py_timezone.extract::<Utc>().is_err());
    }

    #[test]
    fn test_pyo3_time_topyobject() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let hmsm = |h, m, s, ms, py_ms, f| {
            let time = NaiveTime::from_hms_micro(h, m, s, ms).to_object(py);
            let time: &PyTime = time.extract(py).unwrap();
            let py_time =
                PyTime::new_with_fold(py, h as u8, m as u8, s as u8, py_ms, None, f).unwrap();
            assert_eq!(time.compare(py_time).unwrap(), Ordering::Equal);
        };

        hmsm(3, 5, 7, 1_999_999, 999_999, true);
        hmsm(3, 5, 7, 999_999, 999_999, false);
    }

    #[test]
    fn test_pyo3_time_frompyobject() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let hmsm = |h, m, s, ms, py_ms, f| {
            let py_time =
                PyTime::new_with_fold(py, h as u8, m as u8, s as u8, py_ms, None, f).unwrap();
            let py_time: NaiveTime = py_time.extract().unwrap();
            let time = NaiveTime::from_hms_micro(h, m, s, ms);
            assert_eq!(py_time, time);
        };

        hmsm(3, 5, 7, 1_999_999, 999_999, true);
        hmsm(3, 5, 7, 999_999, 999_999, false);
    }
}
