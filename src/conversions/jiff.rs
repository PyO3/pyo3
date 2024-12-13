#![cfg(feature = "jiff-01")]

//! Conversions to and from [jiff](https://docs.rs/jiff/)’s `Span`, `SignedDuration`, `TimeZone`,
//! `Offset`, `Date`, `Time`, `DateTime`, `Zoned`, and `Timestamp`.
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! jiff = "0.1"
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"jiff-01\"] }")]
//! ```
//!
//! Note that you must use compatible versions of jiff and PyO3.
//! The required jiff version may vary based on the version of PyO3. Jiff also requires a MSRV
//! of 1.70.
//!
//! # Example: Convert a `datetime.datetime` to jiff `Zoned`
//!
//! ```rust
//! # #![cfg_attr(windows, allow(unused_imports))]
//! # use jiff_01 as jiff;
//! use jiff::{Zoned, ToSpan};
//! use pyo3::{Python, PyResult, IntoPyObject, types::PyAnyMethods};
//!
//! # #[cfg(windows)]
//! # fn main() -> () {}
//! # #[cfg(not(windows))]
//! fn main() -> PyResult<()> {
//!     pyo3::prepare_freethreaded_python();
//!     Python::with_gil(|py| {
//!         // Build some jiff values
//!         let jiff_zoned = Zoned::now();
//!         let jiff_span = 1.second();
//!         // Convert them to Python
//!         let py_datetime = jiff_zoned.into_pyobject(py)?;
//!         let py_timedelta = jiff_span.into_pyobject(py)?;
//!         // Do an operation in Python
//!         let py_sum = py_datetime.call_method1("__add__", (py_timedelta,))?;
//!         // Convert back to Rust
//!         let jiff_sum: Zoned = py_sum.extract()?;
//!         println!("Zoned: {}", jiff_sum);
//!         Ok(())
//!     })
//! }
//! ```
use crate::exceptions::{PyTypeError, PyValueError};
use crate::pybacked::PyBackedStr;
use crate::sync::GILOnceCell;
#[cfg(not(Py_LIMITED_API))]
use crate::types::datetime::timezone_from_offset;
#[cfg(Py_LIMITED_API)]
use crate::types::datetime_abi3::{check_type, timezone_utc, DatetimeTypes};
#[cfg(Py_LIMITED_API)]
use crate::types::IntoPyDict;
#[cfg(not(Py_LIMITED_API))]
use crate::types::{
    timezone_utc, PyDate, PyDateAccess, PyDateTime, PyDelta, PyDeltaAccess, PyTime, PyTimeAccess,
    PyTzInfo, PyTzInfoAccess,
};
use crate::types::{PyAnyMethods, PyNone, PyType};
use crate::{intern, Bound, FromPyObject, IntoPyObject, Py, PyAny, PyErr, PyResult, Python};
use jiff::civil::{Date, DateTime, Time};
use jiff::tz::{Offset, TimeZone};
use jiff::{SignedDuration, Span, Timestamp, Zoned};
#[cfg(feature = "jiff-01")]
use jiff_01 as jiff;

#[cfg(not(Py_LIMITED_API))]
fn datetime_to_pydatetime<'py>(
    py: Python<'py>,
    datetime: &DateTime,
    fold: bool,
    timezone: Option<&TimeZone>,
) -> PyResult<Bound<'py, PyDateTime>> {
    PyDateTime::new_with_fold(
        py,
        datetime.year().into(),
        datetime.month().try_into()?,
        datetime.day().try_into()?,
        datetime.hour().try_into()?,
        datetime.minute().try_into()?,
        datetime.second().try_into()?,
        (datetime.subsec_nanosecond() / 1000).try_into()?,
        timezone
            .map(|tz| tz.into_pyobject(py))
            .transpose()?
            .as_ref(),
        fold,
    )
}

#[cfg(Py_LIMITED_API)]
fn datetime_to_pydatetime<'py>(
    py: Python<'py>,
    datetime: &DateTime,
    fold: bool,
    timezone: Option<&TimeZone>,
) -> PyResult<Bound<'py, PyAny>> {
    DatetimeTypes::try_get(py)?.datetime.bind(py).call(
        (
            datetime.year(),
            datetime.month(),
            datetime.day(),
            datetime.hour(),
            datetime.minute(),
            datetime.second(),
            datetime.subsec_nanosecond() / 1000,
            timezone,
        ),
        Some(&[("fold", fold as u8)].into_py_dict(py)?),
    )
}

#[cfg(not(Py_LIMITED_API))]
fn pytime_to_time(time: &impl PyTimeAccess) -> PyResult<Time> {
    Ok(Time::new(
        time.get_hour().try_into()?,
        time.get_minute().try_into()?,
        time.get_second().try_into()?,
        (time.get_microsecond() * 1000).try_into()?,
    )?)
}

#[cfg(Py_LIMITED_API)]
fn pytime_to_time(time: &Bound<'_, PyAny>) -> PyResult<Time> {
    let py = time.py();
    Ok(Time::new(
        time.getattr(intern!(py, "hour"))?.extract()?,
        time.getattr(intern!(py, "minute"))?.extract()?,
        time.getattr(intern!(py, "second"))?.extract()?,
        time.getattr(intern!(py, "microsecond"))?.extract::<i32>()? * 1000,
    )?)
}

impl<'py> IntoPyObject<'py> for Timestamp {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Timestamp {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.to_zoned(TimeZone::UTC).into_pyobject(py)
    }
}

impl<'py> FromPyObject<'py> for Timestamp {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let zoned = ob.extract::<Zoned>()?;
        Ok(zoned.timestamp())
    }
}

impl<'py> IntoPyObject<'py> for Date {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDate;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Date {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDate;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        #[cfg(not(Py_LIMITED_API))]
        {
            PyDate::new(
                py,
                self.year().into(),
                self.month().try_into()?,
                self.day().try_into()?,
            )
        }

        #[cfg(Py_LIMITED_API)]
        {
            DatetimeTypes::try_get(py)?
                .date
                .bind(py)
                .call1((self.year(), self.month(), self.day()))
        }
    }
}

impl<'py> FromPyObject<'py> for Date {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        {
            let date = ob.downcast::<PyDate>()?;
            Ok(Date::new(
                date.get_year().try_into()?,
                date.get_month().try_into()?,
                date.get_day().try_into()?,
            )?)
        }

        #[cfg(Py_LIMITED_API)]
        {
            check_type(ob, &DatetimeTypes::get(ob.py()).date, "PyDate")?;
            Ok(Date::new(
                ob.getattr(intern!(ob.py(), "year"))?.extract()?,
                ob.getattr(intern!(ob.py(), "month"))?.extract()?,
                ob.getattr(intern!(ob.py(), "day"))?.extract()?,
            )?)
        }
    }
}

impl<'py> IntoPyObject<'py> for Time {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyTime;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Time {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyTime;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        #[cfg(not(Py_LIMITED_API))]
        {
            PyTime::new(
                py,
                self.hour().try_into()?,
                self.minute().try_into()?,
                self.second().try_into()?,
                (self.subsec_nanosecond() / 1000).try_into()?,
                None,
            )
        }

        #[cfg(Py_LIMITED_API)]
        {
            DatetimeTypes::try_get(py)?.time.bind(py).call1((
                self.hour(),
                self.minute(),
                self.second(),
                self.subsec_nanosecond() / 1000,
            ))
        }
    }
}

impl<'py> FromPyObject<'py> for Time {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        let ob = ob.downcast::<PyTime>()?;

        #[cfg(Py_LIMITED_API)]
        check_type(ob, &DatetimeTypes::get(ob.py()).time, "PyTime")?;

        pytime_to_time(ob)
    }
}

impl<'py> IntoPyObject<'py> for DateTime {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &DateTime {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        datetime_to_pydatetime(py, self, false, None)
    }
}

impl<'py> FromPyObject<'py> for DateTime {
    fn extract_bound(dt: &Bound<'py, PyAny>) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        let dt = dt.downcast::<PyDateTime>()?;

        #[cfg(Py_LIMITED_API)]
        check_type(dt, &DatetimeTypes::get(dt.py()).datetime, "PyDateTime")?;

        #[cfg(not(Py_LIMITED_API))]
        let has_tzinfo = dt.get_tzinfo().is_some();

        #[cfg(Py_LIMITED_API)]
        let has_tzinfo = !dt.getattr(intern!(dt.py(), "tzinfo"))?.is_none();

        if has_tzinfo {
            return Err(PyTypeError::new_err("expected a datetime without tzinfo"));
        }

        Ok(DateTime::from_parts(dt.extract()?, pytime_to_time(dt)?))
    }
}

impl<'py> IntoPyObject<'py> for Zoned {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Zoned {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        fn fold(zoned: &Zoned) -> Option<bool> {
            let prev = zoned.time_zone().preceding(zoned.timestamp()).next()?;
            let next = zoned.time_zone().following(prev.timestamp()).next()?;
            let start_of_current_offset = if next.timestamp() == zoned.timestamp() {
                next.timestamp()
            } else {
                prev.timestamp()
            };
            Some(zoned.timestamp() + (zoned.offset() - prev.offset()) <= start_of_current_offset)
        }
        datetime_to_pydatetime(
            py,
            &self.datetime(),
            fold(self).unwrap_or(false),
            Some(self.time_zone()),
        )
    }
}

impl<'py> FromPyObject<'py> for Zoned {
    fn extract_bound(dt: &Bound<'py, PyAny>) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        let dt = dt.downcast::<PyDateTime>()?;

        #[cfg(Py_LIMITED_API)]
        check_type(dt, &DatetimeTypes::get(dt.py()).datetime, "PyDateTime")?;

        let tz = {
            #[cfg(not(Py_LIMITED_API))]
            let tzinfo: Option<_> = dt.get_tzinfo();

            #[cfg(Py_LIMITED_API)]
            let tzinfo: Option<Bound<'_, PyAny>> =
                dt.getattr(intern!(dt.py(), "tzinfo"))?.extract()?;

            tzinfo
                .map(|tz| tz.extract::<TimeZone>())
                .unwrap_or_else(|| {
                    Err(PyTypeError::new_err(
                        "expected a datetime with non-None tzinfo",
                    ))
                })?
        };
        let datetime = DateTime::from_parts(dt.extract()?, pytime_to_time(dt)?);
        let zoned = tz.into_ambiguous_zoned(datetime);

        #[cfg(not(Py_LIMITED_API))]
        let fold = dt.get_fold();

        #[cfg(Py_LIMITED_API)]
        let fold = dt.getattr(intern!(dt.py(), "fold"))?.extract::<usize>()? > 0;

        if fold {
            Ok(zoned.later()?)
        } else {
            Ok(zoned.earlier()?)
        }
    }
}

impl<'py> IntoPyObject<'py> for TimeZone {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyTzInfo;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &TimeZone {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyTzInfo;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        if self == &TimeZone::UTC {
            Ok(timezone_utc(py))
        } else if let Some(iana_name) = self.iana_name() {
            static ZONE_INFO: GILOnceCell<Py<PyType>> = GILOnceCell::new();
            let tz = ZONE_INFO
                .import(py, "zoneinfo", "ZoneInfo")
                .and_then(|obj| obj.call1((iana_name,)))?;

            #[cfg(not(Py_LIMITED_API))]
            let tz = tz.downcast_into()?;

            Ok(tz)
        } else {
            self.to_fixed_offset()?.into_pyobject(py)
        }
    }
}

impl<'py> FromPyObject<'py> for TimeZone {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        let ob = ob.downcast::<PyTzInfo>()?;

        #[cfg(Py_LIMITED_API)]
        check_type(ob, &DatetimeTypes::get(ob.py()).tzinfo, "PyTzInfo")?;

        let attr = intern!(ob.py(), "key");
        if ob.hasattr(attr)? {
            Ok(TimeZone::get(&ob.getattr(attr)?.extract::<PyBackedStr>()?)?)
        } else {
            Ok(ob.extract::<Offset>()?.to_time_zone())
        }
    }
}

impl<'py> IntoPyObject<'py> for &Offset {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyTzInfo;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        if self == &Offset::UTC {
            return Ok(timezone_utc(py));
        }

        let delta = self.duration_since(Offset::UTC).into_pyobject(py)?;

        #[cfg(not(Py_LIMITED_API))]
        {
            timezone_from_offset(&delta)
        }

        #[cfg(Py_LIMITED_API)]
        {
            DatetimeTypes::try_get(py)?
                .timezone
                .bind(py)
                .call1((delta,))
        }
    }
}

impl<'py> IntoPyObject<'py> for Offset {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyTzInfo;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> FromPyObject<'py> for Offset {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let py = ob.py();

        #[cfg(not(Py_LIMITED_API))]
        let ob = ob.downcast::<PyTzInfo>()?;

        #[cfg(Py_LIMITED_API)]
        check_type(ob, &DatetimeTypes::get(ob.py()).tzinfo, "PyTzInfo")?;

        let py_timedelta = ob.call_method1(intern!(py, "utcoffset"), (PyNone::get(py),))?;
        if py_timedelta.is_none() {
            return Err(PyTypeError::new_err(format!(
                "{:?} is not a fixed offset timezone",
                ob
            )));
        }

        let total_seconds = py_timedelta.extract::<SignedDuration>()?.as_secs();
        debug_assert!(
            (total_seconds / 3600).abs() <= 24,
            "Offset must be between -24 hours and 24 hours but was {}h",
            total_seconds / 3600
        );
        // This cast is safe since the timedelta is limited to -24 hours and 24 hours.
        Ok(Offset::from_seconds(total_seconds as i32)?)
    }
}

impl<'py> IntoPyObject<'py> for &SignedDuration {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDelta;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let total_seconds = self.as_secs();
        let days: i32 = (total_seconds / (24 * 60 * 60)).try_into()?;
        let seconds: i32 = (total_seconds % (24 * 60 * 60)).try_into()?;
        let microseconds = self.subsec_micros();

        #[cfg(not(Py_LIMITED_API))]
        {
            PyDelta::new(py, days, seconds, microseconds, true)
        }

        #[cfg(Py_LIMITED_API)]
        {
            DatetimeTypes::try_get(py)?
                .timedelta
                .bind(py)
                .call1((days, seconds, microseconds))
        }
    }
}

impl<'py> IntoPyObject<'py> for SignedDuration {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDelta;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> FromPyObject<'py> for SignedDuration {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        let (seconds, microseconds) = {
            let delta = ob.downcast::<PyDelta>()?;
            let days = delta.get_days() as i64;
            let seconds = delta.get_seconds() as i64;
            let microseconds = delta.get_microseconds();
            (days * 24 * 60 * 60 + seconds, microseconds)
        };

        #[cfg(Py_LIMITED_API)]
        let (seconds, microseconds) = {
            check_type(ob, &DatetimeTypes::get(ob.py()).timedelta, "PyDelta")?;
            let days = ob.getattr(intern!(ob.py(), "days"))?.extract::<i64>()?;
            let seconds = ob.getattr(intern!(ob.py(), "seconds"))?.extract::<i64>()?;
            let microseconds = ob
                .getattr(intern!(ob.py(), "microseconds"))?
                .extract::<i32>()?;
            (days * 24 * 60 * 60 + seconds, microseconds)
        };

        Ok(SignedDuration::new(seconds, microseconds * 1000))
    }
}

impl<'py> IntoPyObject<'py> for Span {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDelta;
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        // This can fail if this Span contains units greater than days.
        let duration: SignedDuration = self.try_into()?;
        duration.into_pyobject(py)
    }
}

impl<'py> FromPyObject<'py> for Span {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let duration = ob.extract::<SignedDuration>()?;
        Ok(duration.try_into()?)
    }
}

impl From<jiff::Error> for PyErr {
    fn from(e: jiff::Error) -> Self {
        PyValueError::new_err(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(Py_LIMITED_API))]
    use crate::types::timezone_utc;
    use crate::{types::PyTuple, BoundObject};
    use jiff::tz::Offset;
    use std::cmp::Ordering;

    #[test]
    // Only Python>=3.9 has the zoneinfo package
    // We skip the test on windows too since we'd need to install
    // tzdata there to make this work.
    #[cfg(all(Py_3_9, not(target_os = "windows")))]
    fn test_zoneinfo_is_not_fixed_offset() {
        use crate::ffi;
        use crate::types::any::PyAnyMethods;
        use crate::types::dict::PyDictMethods;

        Python::with_gil(|py| {
            let locals = crate::types::PyDict::new(py);
            py.run(
                ffi::c_str!("import zoneinfo; zi = zoneinfo.ZoneInfo('Europe/London')"),
                None,
                Some(&locals),
            )
            .unwrap();
            let result: PyResult<Offset> = locals.get_item("zi").unwrap().unwrap().extract();
            assert!(result.is_err());
            let res = result.err().unwrap();
            // Also check the error message is what we expect
            let msg = res.value(py).repr().unwrap().to_string();
            assert_eq!(msg, "TypeError(\"zoneinfo.ZoneInfo(key='Europe/London') is not a fixed offset timezone\")");
        });
    }

    #[test]
    fn test_timezone_aware_to_naive_fails() {
        // Test that if a user tries to convert a python's timezone aware datetime into a naive
        // one, the conversion fails.
        Python::with_gil(|py| {
            let py_datetime =
                new_py_datetime_ob(py, "datetime", (2022, 1, 1, 1, 0, 0, 0, python_utc(py)));
            // Now test that converting a PyDateTime with tzinfo to a NaiveDateTime fails
            let res: PyResult<DateTime> = py_datetime.extract();
            assert_eq!(
                res.unwrap_err().value(py).repr().unwrap().to_string(),
                "TypeError('expected a datetime without tzinfo')"
            );
        });
    }

    #[test]
    fn test_naive_to_timezone_aware_fails() {
        // Test that if a user tries to convert a python's naive datetime into a timezone aware
        // one, the conversion fails.
        Python::with_gil(|py| {
            let py_datetime = new_py_datetime_ob(py, "datetime", (2022, 1, 1, 1, 0, 0, 0));
            let res: PyResult<Zoned> = py_datetime.extract();
            assert_eq!(
                res.unwrap_err().value(py).repr().unwrap().to_string(),
                "TypeError('expected a datetime with non-None tzinfo')"
            );
        });
    }

    #[test]
    fn test_invalid_types_fail() {
        Python::with_gil(|py| {
            let none = py.None().into_bound(py);
            assert_eq!(
                none.extract::<Span>().unwrap_err().to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyDelta'"
            );
            assert_eq!(
                none.extract::<Offset>().unwrap_err().to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyTzInfo'"
            );
            assert_eq!(
                none.extract::<TimeZone>().unwrap_err().to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyTzInfo'"
            );
            assert_eq!(
                none.extract::<Time>().unwrap_err().to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyTime'"
            );
            assert_eq!(
                none.extract::<Date>().unwrap_err().to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyDate'"
            );
            assert_eq!(
                none.extract::<DateTime>().unwrap_err().to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyDateTime'"
            );
            assert_eq!(
                none.extract::<Zoned>().unwrap_err().to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'PyDateTime'"
            );
        });
    }

    #[test]
    fn test_pyo3_date_into_pyobject() {
        let eq_ymd = |name: &'static str, year, month, day| {
            Python::with_gil(|py| {
                let date = Date::new(year, month, day)
                    .unwrap()
                    .into_pyobject(py)
                    .unwrap();
                let py_date = new_py_datetime_ob(py, "date", (year, month, day));
                assert_eq!(
                    date.compare(&py_date).unwrap(),
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
                let py_date = new_py_datetime_ob(py, "date", (year, month, day));
                let py_date: Date = py_date.extract().unwrap();
                let date = Date::new(year, month, day).unwrap();
                assert_eq!(py_date, date, "{}: {} != {}", name, date, py_date);
            })
        };

        eq_ymd("past date", 2012, 2, 29);
        eq_ymd("min date", 1, 1, 1);
        eq_ymd("future date", 3000, 6, 5);
        eq_ymd("max date", 9999, 12, 31);
    }

    #[test]
    fn test_pyo3_datetime_into_pyobject_utc() {
        Python::with_gil(|py| {
            let check_utc =
                |name: &'static str, year, month, day, hour, minute, second, ms, py_ms| {
                    let datetime = DateTime::new(year, month, day, hour, minute, second, ms * 1000)
                        .unwrap()
                        .to_zoned(TimeZone::UTC)
                        .unwrap();
                    let datetime = datetime.into_pyobject(py).unwrap();
                    let py_datetime = new_py_datetime_ob(
                        py,
                        "datetime",
                        (
                            year,
                            month,
                            day,
                            hour,
                            minute,
                            second,
                            py_ms,
                            python_utc(py),
                        ),
                    );
                    assert_eq!(
                        datetime.compare(&py_datetime).unwrap(),
                        Ordering::Equal,
                        "{}: {} != {}",
                        name,
                        datetime,
                        py_datetime
                    );
                };

            check_utc("regular", 2014, 5, 6, 7, 8, 9, 999_999, 999_999);
        })
    }

    #[test]
    fn test_pyo3_datetime_into_pyobject_fixed_offset() {
        Python::with_gil(|py| {
            let check_fixed_offset =
                |name: &'static str, year, month, day, hour, minute, second, ms, py_ms| {
                    let offset = Offset::from_seconds(3600).unwrap();
                    let datetime = DateTime::new(year, month, day, hour, minute, second, ms * 1000)
                        .map_err(|e| {
                            eprintln!("{}: {}", name, e);
                            e
                        })
                        .unwrap()
                        .to_zoned(offset.to_time_zone())
                        .unwrap();
                    let datetime = datetime.into_pyobject(py).unwrap();
                    let py_tz = offset.into_pyobject(py).unwrap();
                    let py_datetime = new_py_datetime_ob(
                        py,
                        "datetime",
                        (year, month, day, hour, minute, second, py_ms, py_tz),
                    );
                    assert_eq!(
                        datetime.compare(&py_datetime).unwrap(),
                        Ordering::Equal,
                        "{}: {} != {}",
                        name,
                        datetime,
                        py_datetime
                    );
                };

            check_fixed_offset("regular", 2014, 5, 6, 7, 8, 9, 999_999, 999_999);
        })
    }

    #[test]
    #[cfg(all(Py_3_9, not(windows)))]
    fn test_pyo3_datetime_into_pyobject_tz() {
        Python::with_gil(|py| {
            let datetime = DateTime::new(2024, 12, 11, 23, 3, 13, 0)
                .unwrap()
                .to_zoned(TimeZone::get("Europe/London").unwrap())
                .unwrap();
            let datetime = datetime.into_pyobject(py).unwrap();
            let py_datetime = new_py_datetime_ob(
                py,
                "datetime",
                (
                    2024,
                    12,
                    11,
                    23,
                    3,
                    13,
                    0,
                    python_zoneinfo(py, "Europe/London"),
                ),
            );
            assert_eq!(datetime.compare(&py_datetime).unwrap(), Ordering::Equal);
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
            let tz_utc = timezone_utc(py);
            let py_datetime = new_py_datetime_ob(
                py,
                "datetime",
                (year, month, day, hour, minute, second, micro, tz_utc),
            );
            let py_datetime: Zoned = py_datetime.extract().unwrap();
            let datetime = DateTime::new(year, month, day, hour, minute, second, micro * 1000)
                .unwrap()
                .to_zoned(TimeZone::UTC)
                .unwrap();
            assert_eq!(py_datetime, datetime,);
        })
    }

    #[test]
    #[cfg(all(Py_3_9, not(windows)))]
    fn test_ambiguous_datetime_to_pyobject() {
        use std::str::FromStr;
        let dates = [
            Zoned::from_str("2020-10-24 23:00:00[UTC]").unwrap(),
            Zoned::from_str("2020-10-25 00:00:00[UTC]").unwrap(),
            Zoned::from_str("2020-10-25 01:00:00[UTC]").unwrap(),
            Zoned::from_str("2020-10-25 02:00:00[UTC]").unwrap(),
        ];

        let tz = TimeZone::get("Europe/London").unwrap();
        let dates = dates.map(|dt| dt.with_time_zone(tz.clone()));

        assert_eq!(
            dates.clone().map(|ref dt| dt.to_string()),
            [
                "2020-10-25T00:00:00+01:00[Europe/London]",
                "2020-10-25T01:00:00+01:00[Europe/London]",
                "2020-10-25T01:00:00+00:00[Europe/London]",
                "2020-10-25T02:00:00+00:00[Europe/London]",
            ]
        );

        let dates = Python::with_gil(|py| {
            let pydates = dates.map(|dt| dt.into_pyobject(py).unwrap());
            assert_eq!(
                pydates
                    .clone()
                    .map(|dt| dt.getattr("hour").unwrap().extract::<usize>().unwrap()),
                [0, 1, 1, 2]
            );

            assert_eq!(
                pydates
                    .clone()
                    .map(|dt| dt.getattr("fold").unwrap().extract::<usize>().unwrap() > 0),
                [false, false, true, false]
            );

            pydates.map(|dt| dt.extract::<Zoned>().unwrap())
        });

        assert_eq!(
            dates.map(|dt| dt.to_string()),
            [
                "2020-10-25T00:00:00+01:00[Europe/London]",
                "2020-10-25T01:00:00+01:00[Europe/London]",
                "2020-10-25T01:00:00+00:00[Europe/London]",
                "2020-10-25T02:00:00+00:00[Europe/London]",
            ]
        );
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
            let offset = Offset::from_seconds(3600).unwrap();
            let py_tz = offset.into_pyobject(py).unwrap();
            let py_datetime = new_py_datetime_ob(
                py,
                "datetime",
                (year, month, day, hour, minute, second, micro, py_tz),
            );
            let datetime_from_py: Zoned = py_datetime.extract().unwrap();
            let datetime =
                DateTime::new(year, month, day, hour, minute, second, micro * 1000).unwrap();
            let datetime = datetime.to_zoned(offset.to_time_zone()).unwrap();

            assert_eq!(datetime_from_py, datetime);
        })
    }

    #[test]
    fn test_pyo3_offset_fixed_into_pyobject() {
        Python::with_gil(|py| {
            // jiff offset
            let offset = Offset::from_seconds(3600)
                .unwrap()
                .into_pyobject(py)
                .unwrap();
            // Python timezone from timedelta
            let td = new_py_datetime_ob(py, "timedelta", (0, 3600, 0));
            let py_timedelta = new_py_datetime_ob(py, "timezone", (td,));
            // Should be equal
            assert!(offset.eq(py_timedelta).unwrap());

            // Same but with negative values
            let offset = Offset::from_seconds(-3600)
                .unwrap()
                .into_pyobject(py)
                .unwrap();
            let td = new_py_datetime_ob(py, "timedelta", (0, -3600, 0));
            let py_timedelta = new_py_datetime_ob(py, "timezone", (td,));
            assert!(offset.eq(py_timedelta).unwrap());
        })
    }

    #[test]
    fn test_pyo3_offset_fixed_frompyobject() {
        Python::with_gil(|py| {
            let py_timedelta = new_py_datetime_ob(py, "timedelta", (0, 3600, 0));
            let py_tzinfo = new_py_datetime_ob(py, "timezone", (py_timedelta,));
            let offset: Offset = py_tzinfo.extract().unwrap();
            assert_eq!(Offset::from_seconds(3600).unwrap(), offset);
        })
    }

    #[test]
    fn test_pyo3_offset_utc_into_pyobject() {
        Python::with_gil(|py| {
            let utc = Offset::UTC.into_pyobject(py).unwrap();
            let py_utc = python_utc(py);
            assert!(utc.is(&py_utc));
        })
    }

    #[test]
    fn test_pyo3_offset_utc_frompyobject() {
        Python::with_gil(|py| {
            let py_utc = python_utc(py);
            let py_utc: Offset = py_utc.extract().unwrap();
            assert_eq!(Offset::UTC, py_utc);

            let py_timedelta = new_py_datetime_ob(py, "timedelta", (0, 0, 0));
            let py_timezone_utc = new_py_datetime_ob(py, "timezone", (py_timedelta,));
            let py_timezone_utc: Offset = py_timezone_utc.extract().unwrap();
            assert_eq!(Offset::UTC, py_timezone_utc);

            let py_timedelta = new_py_datetime_ob(py, "timedelta", (0, 3600, 0));
            let py_timezone = new_py_datetime_ob(py, "timezone", (py_timedelta,));
            assert_ne!(Offset::UTC, py_timezone.extract::<Offset>().unwrap());
        })
    }

    #[test]
    fn test_pyo3_time_into_pyobject() {
        Python::with_gil(|py| {
            let check_time = |name: &'static str, hour, minute, second, ms, py_ms| {
                let time = Time::new(hour, minute, second, ms * 1000)
                    .unwrap()
                    .into_pyobject(py)
                    .unwrap();
                let py_time = new_py_datetime_ob(py, "time", (hour, minute, second, py_ms));
                assert!(
                    time.eq(&py_time).unwrap(),
                    "{}: {} != {}",
                    name,
                    time,
                    py_time
                );
            };

            check_time("regular", 3, 5, 7, 999_999, 999_999);
        })
    }

    #[test]
    fn test_pyo3_time_frompyobject() {
        let hour = 3;
        let minute = 5;
        let second = 7;
        let micro = 999_999;
        Python::with_gil(|py| {
            let py_time = new_py_datetime_ob(py, "time", (hour, minute, second, micro));
            let py_time: Time = py_time.extract().unwrap();
            let time = Time::new(hour, minute, second, micro * 1000).unwrap();
            assert_eq!(py_time, time);
        })
    }

    fn new_py_datetime_ob<'py, A>(py: Python<'py>, name: &str, args: A) -> Bound<'py, PyAny>
    where
        A: IntoPyObject<'py, Target = PyTuple>,
    {
        py.import("datetime")
            .unwrap()
            .getattr(name)
            .unwrap()
            .call1(
                args.into_pyobject(py)
                    .map_err(Into::into)
                    .unwrap()
                    .into_bound(),
            )
            .unwrap()
    }

    fn python_utc(py: Python<'_>) -> Bound<'_, PyAny> {
        py.import("datetime")
            .unwrap()
            .getattr("timezone")
            .unwrap()
            .getattr("utc")
            .unwrap()
    }

    #[cfg(all(Py_3_9, not(windows)))]
    fn python_zoneinfo<'py>(py: Python<'py>, timezone: &str) -> Bound<'py, PyAny> {
        py.import("zoneinfo")
            .unwrap()
            .getattr("ZoneInfo")
            .unwrap()
            .call1((timezone,))
            .unwrap()
    }

    #[cfg(not(any(target_arch = "wasm32", Py_GIL_DISABLED)))]
    mod proptests {
        use super::*;
        use crate::types::IntoPyDict;
        use jiff::tz::TimeZoneTransition;
        use proptest::prelude::*;
        use std::ffi::CString;

        // This is to skip the test if we are creating an invalid date, like February 31.
        fn try_date(year: i32, month: u32, day: u32) -> PyResult<Date> {
            Ok(Date::new(
                year.try_into()?,
                month.try_into()?,
                day.try_into()?,
            )?)
        }

        fn try_time(hour: u32, min: u32, sec: u32, micro: u32) -> PyResult<Time> {
            Ok(Time::new(
                hour.try_into()?,
                min.try_into()?,
                sec.try_into()?,
                (micro * 1000).try_into()?,
            )?)
        }

        prop_compose! {
            fn timezone_transitions(timezone: &TimeZone)
                            (year in 1900i16..=2100i16, month in 1i8..=12i8)
                            -> TimeZoneTransition<'_> {
                let datetime = DateTime::new(year, month, 1, 0, 0, 0, 0).unwrap();
                let timestamp= timezone.to_zoned(datetime).unwrap().timestamp();
                timezone.following(timestamp).next().unwrap()
            }
        }

        proptest! {

            // Range is limited to 1970 to 2038 due to windows limitations
            #[test]
            fn test_pyo3_offset_fixed_frompyobject_created_in_python(timestamp in 0..(i32::MAX as i64), timedelta in -86399i32..=86399i32) {
                Python::with_gil(|py| {

                    let globals = [("datetime", py.import("datetime").unwrap())].into_py_dict(py).unwrap();
                    let code = format!("datetime.datetime.fromtimestamp({}).replace(tzinfo=datetime.timezone(datetime.timedelta(seconds={})))", timestamp, timedelta);
                    let t = py.eval(&CString::new(code).unwrap(), Some(&globals), None).unwrap();

                    // Get ISO 8601 string from python
                    let py_iso_str = t.call_method0("isoformat").unwrap();

                    // Get ISO 8601 string from rust
                    let rust_iso_str = t.extract::<Zoned>().unwrap().strftime("%Y-%m-%dT%H:%M:%S%:z").to_string();

                    // They should be equal
                    assert_eq!(py_iso_str.to_string(), rust_iso_str);
                })
            }

            #[test]
            fn test_duration_roundtrip(days in -999999999i64..=999999999i64) {
                // Test roundtrip conversion rust->python->rust for all allowed
                // python values of durations (from -999999999 to 999999999 days),
                Python::with_gil(|py| {
                    let dur = SignedDuration::new(days * 24 * 60 * 60, 0);
                    let py_delta = dur.into_pyobject(py).unwrap();
                    let roundtripped: SignedDuration = py_delta.extract().expect("Round trip");
                    assert_eq!(dur, roundtripped);
                })
            }

            #[test]
            fn test_span_roundtrip(days in -999999999i64..=999999999i64) {
                // Test roundtrip conversion rust->python->rust for all allowed
                // python values of durations (from -999999999 to 999999999 days),
                Python::with_gil(|py| {
                    if let Ok(span) = Span::new().try_days(days) {
                        let py_delta = span.into_pyobject(py).unwrap();
                        let roundtripped: Span = py_delta.extract().expect("Round trip");
                        assert_eq!(span.compare(roundtripped).unwrap(), Ordering::Equal);
                    }
                })
            }

            #[test]
            fn test_fixed_offset_roundtrip(secs in -86399i32..=86399i32) {
                Python::with_gil(|py| {
                    let offset = Offset::from_seconds(secs).unwrap();
                    let py_offset = offset.into_pyobject(py).unwrap();
                    let roundtripped: Offset = py_offset.extract().expect("Round trip");
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
                    if let Ok(date) = try_date(year, month, day) {
                        let py_date = date.into_pyobject(py).unwrap();
                        let roundtripped: Date = py_date.extract().expect("Round trip");
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
                Python::with_gil(|py| {
                    if let Ok(time) = try_time(hour, min, sec, micro) {
                        let py_time = time.into_pyobject(py).unwrap();
                        let roundtripped: Time = py_time.extract().expect("Round trip");
                        assert_eq!(time, roundtripped);
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
                    let date_opt = try_date(year, month, day);
                    let time_opt = try_time(hour, min, sec, micro);
                    if let (Ok(date), Ok(time)) = (date_opt, time_opt) {
                        let dt = DateTime::from_parts(date, time);
                        let pydt = dt.into_pyobject(py).unwrap();
                        let roundtripped: DateTime = pydt.extract().expect("Round trip");
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
                    let date_opt = try_date(year, month, day);
                    let time_opt = try_time(hour, min, sec, micro);
                    if let (Ok(date), Ok(time)) = (date_opt, time_opt) {
                        let dt: Zoned = DateTime::from_parts(date, time).to_zoned(TimeZone::UTC).unwrap();
                        let py_dt = (&dt).into_pyobject(py).unwrap();
                        let roundtripped: Zoned = py_dt.extract().expect("Round trip");
                        assert_eq!(dt, roundtripped);
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
                    let date_opt = try_date(year, month, day);
                    let time_opt = try_time(hour, min, sec, micro);
                    let offset = Offset::from_seconds(offset_secs).unwrap();
                    if let (Ok(date), Ok(time)) = (date_opt, time_opt) {
                        let dt: Zoned = DateTime::from_parts(date, time).to_zoned(offset.to_time_zone()).unwrap();
                        let py_dt = (&dt).into_pyobject(py).unwrap();
                        let roundtripped: Zoned = py_dt.extract().expect("Round trip");
                        assert_eq!(dt, roundtripped);
                    }
                })
            }

            #[test]
            #[cfg(all(Py_3_9, not(windows)))]
            fn test_zoned_datetime_roundtrip_around_timezone_transition(
                (timezone, transition) in prop_oneof![
                                Just(&TimeZone::get("Europe/London").unwrap()),
                                Just(&TimeZone::get("America/New_York").unwrap()),
                                Just(&TimeZone::get("Australia/Sydney").unwrap()),
                            ].prop_flat_map(|tz| (Just(tz), timezone_transitions(tz))),
                hour in -2i32..=2i32,
                min in 0u32..=59u32,
            ) {

                Python::with_gil(|py| {
                    let transition_moment = transition.timestamp();
                    let zoned = (transition_moment - Span::new().hours(hour).minutes(min))
                        .to_zoned(timezone.clone());

                    let py_dt = (&zoned).into_pyobject(py).unwrap();
                    let roundtripped: Zoned = py_dt.extract().expect("Round trip");
                    assert_eq!(zoned, roundtripped);
                })

            }
        }
    }
}
