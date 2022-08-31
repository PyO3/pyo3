#![cfg(feature = "chrono")]

//! Conversions to and from [chrono](https://docs.rs/chrono/)’s `Duration`,
//! `NaiveDate`, `NaiveTime`, `DateTime<Tz>`, `FixedOffset`, and `Utc`.
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! # change * to the latest versions
//! hashbrown = "*"
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
    div_mod_floor_64, DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Offset,
    TimeZone, Timelike, NANOS_PER_MICRO, SECS_PER_DAY,
};
use std::convert::TryInto;

impl ToPyObject for Duration {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let micros = self.nanos / NANOS_PER_MICRO;
        let (days, secs) = div_mod_floor_64(self.secs, SECS_PER_DAY);
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
        let secs = delta.get_days() as i64 * SECS_PER_DAY + delta.get_seconds() as i64;
        let nanos = delta.get_microseconds() * NANOS_PER_MICRO;

        Ok(Duration { secs, nanos })
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
    use crate::types::*;
    use crate::{IntoPy, PyObject, PyTryFrom, Python, ToPyObject};

    // TODO
}
