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
    DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Timelike,
};
use std::convert::TryInto;

const SECS_PER_DAY: i64 = 86_400;
const MICROS_PER_SEC: i64 = 1_000_000;

impl ToPyObject for Duration {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        // This is the total number of microseconds in the duration,
        // unlike std::time::Duration we can't have only the fractional part,
        // so we need to calculate it ourselves.
        let total_micros = self
            .num_microseconds()
            // num_microseconds returns None on overflow
            .unwrap_or(i64::MAX);
        let total_secs = total_micros / MICROS_PER_SEC;
        let micros = total_micros - total_secs * MICROS_PER_SEC;
        let days = self.num_days();
        let secs = total_secs - days * SECS_PER_DAY;

        // We do not need to check i64 to i32 cast from rust because
        // python will panic with OverflowError.
        // Not sure if we need normalize here.
        let delta = PyDelta::new(
            py,
            days.try_into().unwrap_or(i32::MAX),
            secs.try_into().unwrap_or(i32::MAX),
            micros.try_into().unwrap_or(i32::MAX),
            false,
        )
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
        let secs = delta.get_days() as i64 * SECS_PER_DAY + delta.get_seconds() as i64;
        let micros = delta.get_microseconds() as i64;

        Ok(Duration::microseconds(secs * MICROS_PER_SEC + micros))
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
        let datetime = PyDateTime::new_with_fold(py, yy, mm, dd, h, m, s, ms, Some(&tz), fold)
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
    // use crate::types::*;
    // use crate::{IntoPy, PyObject, PyTryFrom, Python, ToPyObject};

    // TODO
}
