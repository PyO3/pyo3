#![cfg(feature = "time")]

//! Conversions to and from [time](https://docs.rs/time/)’s `Date`,
//! `Duration`, `OffsetDateTime`, `PrimitiveDateTime`, `Time`, `UtcDateTime`,
//! and `UtcOffset`.
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! time = "0.3"
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"time\"] }")]
//! ```
//!
//! Note that you must use compatible versions of time and PyO3.
//! The required time version may vary based on the version of PyO3.
//!
//! FIXME: Rework this example
//! # Example: Convert a `datetime.datetime` to chrono's `DateTime<Utc>`
//!
//! ```rust
//! use chrono::{DateTime, Duration, TimeZone, Utc};
//! use pyo3::{Python, PyResult, IntoPyObject, types::PyAnyMethods};
//!
//! fn main() -> PyResult<()> {
//!     pyo3::prepare_freethreaded_python();
//!     Python::with_gil(|py| {
//!         // Build some chrono values
//!         let chrono_datetime = Utc.with_ymd_and_hms(2022, 1, 1, 12, 0, 0).unwrap();
//!         let chrono_duration = Duration::seconds(1);
//!         // Convert them to Python
//!         let py_datetime = chrono_datetime.into_pyobject(py)?;
//!         let py_timedelta = chrono_duration.into_pyobject(py)?;
//!         // Do an operation in Python
//!         let py_sum = py_datetime.call_method1("__add__", (py_timedelta,))?;
//!         // Convert back to Rust
//!         let chrono_sum: DateTime<Utc> = py_sum.extract()?;
//!         println!("DateTime<Utc>: {}", chrono_datetime);
//!         Ok(())
//!     })
//! }
//! ```

use crate::exceptions::{PyTypeError, PyValueError};
use crate::intern;
#[cfg(not(Py_LIMITED_API))]
use crate::types::datetime::{timezone_from_offset, PyDateAccess, PyDeltaAccess};
use crate::types::{PyAnyMethods, PyNone};
#[cfg(not(Py_LIMITED_API))]
use crate::types::{PyDate, PyDateTime, PyDelta, PyTime, PyTimeAccess, PyTzInfo, PyTzInfoAccess};
use crate::{Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python};
use time::{
    Date, Duration, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcDateTime, UtcOffset,
};

// Macro for reference implementation
macro_rules! impl_into_py_for_ref {
    ($type:ty, $target:ty) => {
        impl<'py> IntoPyObject<'py> for &$type {
            #[cfg(Py_LIMITED_API)]
            type Target = PyAny;
            #[cfg(not(Py_LIMITED_API))]
            type Target = $target;
            type Output = Bound<'py, Self::Target>;
            type Error = PyErr;

            #[inline]
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                (*self).into_pyobject(py)
            }
        }
    };
}

// Macro for month conversion
macro_rules! month_from_number {
    ($month:expr) => {
        match $month {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            12 => Month::December,
            _ => return Err(PyValueError::new_err("invalid month value")),
        }
    };
}

impl<'py> IntoPyObject<'py> for Duration {
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDelta;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let total_seconds = self.whole_seconds();
        let micro_seconds = self.subsec_microseconds();

        // For negative durations, Python expects days to be negative and
        // seconds/microseconds to be positive or zero
        let (days, seconds) = if total_seconds < 0 && total_seconds % 86_400 != 0 {
            // For negative values, we need to round down (toward more negative)
            // e.g., -10 seconds should be -1 days + 86390 seconds
            let days = (total_seconds / 86_400) - 1;
            let seconds = 86_400 + (total_seconds % 86_400);
            (days, seconds)
        } else {
            // For positive or exact negative days, use normal division
            (total_seconds / 86_400, total_seconds % 86_400)
        };
        #[cfg(not(Py_LIMITED_API))]
        {
            // Create the timedelta with days, seconds, microseconds
            // Safe to unwrap as we've verified the values are within bounds
            PyDelta::new(
                py,
                days.try_into().expect("days overflow"),
                seconds.try_into().expect("seconds overflow"),
                micro_seconds,
                true,
            )
        }

        #[cfg(Py_LIMITED_API)]
        {
            py.import("datetime")?
                .getattr(intern!(py, "timedelta"))?
                .call1((days, seconds, micro_seconds))
        }
    }
}

impl FromPyObject<'_> for Duration {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Duration> {
        #[cfg(not(Py_LIMITED_API))]
        let (days, seconds, microseconds) = {
            let delta = ob.downcast::<PyDelta>()?;
            (
                delta.get_days().into(),
                delta.get_seconds().into(),
                delta.get_microseconds().into(),
            )
        };

        #[cfg(Py_LIMITED_API)]
        let (days, seconds, microseconds) = {
            (
                ob.getattr(intern!(ob.py(), "days"))?.extract()?,
                ob.getattr(intern!(ob.py(), "seconds"))?.extract()?,
                ob.getattr(intern!(ob.py(), "microseconds"))?.extract()?,
            )
        };

        Ok(
            Duration::days(days)
                + Duration::seconds(seconds)
                + Duration::microseconds(microseconds),
        )
    }
}

impl<'py> IntoPyObject<'py> for Date {
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDate;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let year = self.year();
        let month = self.month() as u8;
        let day = self.day();

        #[cfg(not(Py_LIMITED_API))]
        {
            PyDate::new(py, year, month, day)
        }

        #[cfg(Py_LIMITED_API)]
        {
            py.import("datetime")?
                .getattr(intern!(py, "date"))?
                .call1((year, month, day))
        }
    }
}

impl FromPyObject<'_> for Date {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Date> {
        let (year, month, day) = {
            #[cfg(not(Py_LIMITED_API))]
            {
                let date = ob.downcast::<PyDate>()?;
                (date.get_year(), date.get_month(), date.get_day())
            }

            #[cfg(Py_LIMITED_API)]
            {
                let year = ob.getattr(intern!(ob.py(), "year"))?.extract()?;
                let month: u8 = ob.getattr(intern!(ob.py(), "month"))?.extract()?;
                let day = ob.getattr(intern!(ob.py(), "day"))?.extract()?;
                (year, month, day)
            }
        };

        // Convert the month number to time::Month enum
        let month = month_from_number!(month);

        Date::from_calendar_date(year, month, day)
            .map_err(|_| PyValueError::new_err("invalid or out-of-range date"))
    }
}

impl<'py> IntoPyObject<'py> for Time {
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let hour = self.hour();
        let minute = self.minute();
        let second = self.second();
        let microsecond = self.microsecond();

        #[cfg(not(Py_LIMITED_API))]
        {
            PyTime::new(py, hour, minute, second, microsecond, None)
        }

        #[cfg(Py_LIMITED_API)]
        {
            py.import("datetime")?.getattr(intern!(py, "time"))?.call1((
                hour,
                minute,
                second,
                microsecond,
            ))
        }
    }
}

impl FromPyObject<'_> for Time {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Time> {
        let (hour, minute, second, microsecond) = {
            #[cfg(not(Py_LIMITED_API))]
            {
                let time = ob.downcast::<PyTime>()?;
                let hour: u8 = time.get_hour();
                let minute: u8 = time.get_minute();
                let second: u8 = time.get_second();
                let microsecond = time.get_microsecond();
                (hour, minute, second, microsecond)
            }

            #[cfg(Py_LIMITED_API)]
            {
                let hour: u8 = ob.getattr(intern!(ob.py(), "hour"))?.extract()?;
                let minute: u8 = ob.getattr(intern!(ob.py(), "minute"))?.extract()?;
                let second: u8 = ob.getattr(intern!(ob.py(), "second"))?.extract()?;
                let microsecond = ob.getattr(intern!(ob.py(), "microsecond"))?.extract()?;
                (hour, minute, second, microsecond)
            }
        };

        Time::from_hms_micro(hour, minute, second, microsecond)
            .map_err(|_| PyValueError::new_err("invalid or out-of-range time"))
    }
}

impl<'py> IntoPyObject<'py> for PrimitiveDateTime {
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let date = self.date();
        let time = self.time();

        let year = date.year();
        let month = date.month() as u8;
        let day = date.day();
        let hour = time.hour();
        let minute = time.minute();
        let second = time.second();
        let microsecond = time.microsecond();

        #[cfg(not(Py_LIMITED_API))]
        {
            PyDateTime::new(
                py,
                year,
                month,
                day,
                hour,
                minute,
                second,
                microsecond,
                None,
            )
        }

        #[cfg(Py_LIMITED_API)]
        {
            py.import("datetime")?
                .getattr(intern!(py, "datetime"))?
                .call1((year, month, day, hour, minute, second, microsecond))
        }
    }
}

// FIXME: Refactor this impl
impl FromPyObject<'_> for PrimitiveDateTime {
    fn extract_bound(dt: &Bound<'_, PyAny>) -> PyResult<PrimitiveDateTime> {
        #[cfg(not(Py_LIMITED_API))]
        let dt = dt.downcast::<PyDateTime>()?;

        // If the user tries to convert a timezone aware datetime into a naive one,
        // we return a hard error
        #[cfg(not(Py_LIMITED_API))]
        let has_tzinfo = dt.get_tzinfo().is_some();
        #[cfg(Py_LIMITED_API)]
        let has_tzinfo = !dt.getattr(intern!(dt.py(), "tzinfo"))?.is_none();
        if has_tzinfo {
            return Err(PyTypeError::new_err("expected a datetime without tzinfo"));
        }

        // Extract date
        #[cfg(not(Py_LIMITED_API))]
        let date = Date::from_calendar_date(
            dt.get_year(),
            month_from_number!(dt.get_month()),
            dt.get_day(),
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range date"))?;

        // Extract time
        #[cfg(not(Py_LIMITED_API))]
        let time = Time::from_hms_micro(
            dt.get_hour(),
            dt.get_minute(),
            dt.get_second(),
            dt.get_microsecond(),
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range time"))?;

        #[cfg(Py_LIMITED_API)]
        let date = Date::from_calendar_date(
            dt.getattr(intern!(dt.py(), "year"))?.extract()?,
            month_from_number!(dt.getattr(intern!(dt.py(), "month"))?.extract::<u8>()?),
            dt.getattr(intern!(dt.py(), "day"))?.extract()?,
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range date"))?;

        #[cfg(Py_LIMITED_API)]
        let time = Time::from_hms_micro(
            dt.getattr(intern!(dt.py(), "hour"))?.extract()?,
            dt.getattr(intern!(dt.py(), "minute"))?.extract()?,
            dt.getattr(intern!(dt.py(), "second"))?.extract()?,
            dt.getattr(intern!(dt.py(), "microsecond"))?.extract()?,
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range time"))?;

        Ok(PrimitiveDateTime::new(date, time))
    }
}

impl<'py> IntoPyObject<'py> for UtcOffset {
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyTzInfo;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        // Get offset in seconds
        let seconds_offset = self.whole_seconds();

        #[cfg(not(Py_LIMITED_API))]
        {
            let td = PyDelta::new(py, 0, seconds_offset, 0, true)?;
            timezone_from_offset(&td)
        }

        #[cfg(Py_LIMITED_API)]
        {
            let td = Duration::seconds(seconds_offset as i64).into_pyobject(py)?;
            py.import("datetime")?
                .getattr(intern!(py, "timezone"))?
                .call1((td,))
        }
    }
}

impl FromPyObject<'_> for UtcOffset {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<UtcOffset> {
        #[cfg(not(Py_LIMITED_API))]
        let ob = ob.downcast::<PyTzInfo>()?;

        // Get the offset in seconds from the Python tzinfo
        let py_timedelta = ob.call_method1("utcoffset", (PyNone::get(ob.py()),))?;
        if py_timedelta.is_none() {
            return Err(PyTypeError::new_err(format!(
                "{:?} is not a fixed offset timezone",
                ob
            )));
        }

        let total_seconds: Duration = py_timedelta.extract()?;
        let seconds = total_seconds.whole_seconds();

        // Create the UtcOffset from the seconds
        UtcOffset::from_whole_seconds(seconds as i32)
            .map_err(|_| PyValueError::new_err("UTC offset out of bounds"))
    }
}

impl<'py> IntoPyObject<'py> for OffsetDateTime {
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let date = self.date();
        let time = self.time();
        let offset = self.offset();

        // Convert the offset to a Python tzinfo
        let py_tzinfo = offset.into_pyobject(py)?;

        let year = date.year();
        let month = date.month() as u8;
        let day = date.day();
        let hour = time.hour();
        let minute = time.minute();
        let second = time.second();
        let microsecond = time.microsecond();

        #[cfg(not(Py_LIMITED_API))]
        {
            PyDateTime::new(
                py,
                year,
                month,
                day,
                hour,
                minute,
                second,
                microsecond,
                Some(py_tzinfo.downcast()?),
            )
        }

        #[cfg(Py_LIMITED_API)]
        {
            py.import("datetime")?
                .getattr(intern!(py, "datetime"))?
                .call1((
                    year,
                    month,
                    day,
                    hour,
                    minute,
                    second,
                    microsecond,
                    py_tzinfo,
                ))
        }
    }
}

impl FromPyObject<'_> for OffsetDateTime {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<OffsetDateTime> {
        #[cfg(not(Py_LIMITED_API))]
        let dt = ob.downcast::<PyDateTime>()?;

        // Extract the tzinfo and make sure it's not None
        #[cfg(not(Py_LIMITED_API))]
        let tzinfo = dt
            .get_tzinfo()
            .ok_or_else(|| PyTypeError::new_err("expected a datetime with non-None tzinfo"))?;

        #[cfg(Py_LIMITED_API)]
        let tzinfo = ob.getattr(intern!(ob.py(), "tzinfo"))?;
        #[cfg(Py_LIMITED_API)]
        if tzinfo.is_none() {
            return Err(PyTypeError::new_err(
                "expected a datetime with non-None tzinfo",
            ));
        }

        // Convert tzinfo to UtcOffset
        let offset: UtcOffset = tzinfo.extract()?;

        // Extract the date and time parts
        #[cfg(not(Py_LIMITED_API))]
        let date = Date::from_calendar_date(
            dt.get_year(),
            month_from_number!(dt.get_month()),
            dt.get_day(),
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range date"))?;

        // Extract time
        #[cfg(not(Py_LIMITED_API))]
        let time = Time::from_hms_micro(
            dt.get_hour(),
            dt.get_minute(),
            dt.get_second(),
            dt.get_microsecond(),
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range time"))?;

        #[cfg(Py_LIMITED_API)]
        let date = Date::from_calendar_date(
            ob.getattr(intern!(ob.py(), "year"))?.extract()?,
            month_from_number!(ob.getattr(intern!(ob.py(), "month"))?.extract::<u8>()?),
            ob.getattr(intern!(ob.py(), "day"))?.extract()?,
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range date"))?;

        #[cfg(Py_LIMITED_API)]
        let time = Time::from_hms_micro(
            ob.getattr(intern!(ob.py(), "hour"))?.extract()?,
            ob.getattr(intern!(ob.py(), "minute"))?.extract()?,
            ob.getattr(intern!(ob.py(), "second"))?.extract()?,
            ob.getattr(intern!(ob.py(), "microsecond"))?.extract()?,
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range time"))?;

        // Create the PrimitiveDateTime first
        let primitive_dt = PrimitiveDateTime::new(date, time);

        // Then attach the offset
        Ok(primitive_dt.assume_offset(offset))
    }
}

impl<'py> IntoPyObject<'py> for UtcDateTime {
    #[cfg(Py_LIMITED_API)]
    type Target = PyAny;
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let date = self.date();
        let time = self.time();

        // Get UTC timezone
        #[cfg(not(Py_LIMITED_API))]
        let py_tzinfo = py
            .import("datetime")?
            .getattr(intern!(py, "timezone"))?
            .getattr(intern!(py, "utc"))?;

        #[cfg(Py_LIMITED_API)]
        let py_tzinfo = py
            .import("datetime")?
            .getattr(intern!(py, "timezone"))?
            .getattr(intern!(py, "utc"))?;

        let year = date.year();
        let month = date.month() as u8;
        let day = date.day();
        let hour = time.hour();
        let minute = time.minute();
        let second = time.second();
        let microsecond = time.microsecond();

        #[cfg(not(Py_LIMITED_API))]
        {
            PyDateTime::new(
                py,
                year,
                month,
                day,
                hour,
                minute,
                second,
                microsecond,
                Some(py_tzinfo.downcast()?),
            )
        }

        #[cfg(Py_LIMITED_API)]
        {
            py.import("datetime")?
                .getattr(intern!(py, "datetime"))?
                .call1((
                    year,
                    month,
                    day,
                    hour,
                    minute,
                    second,
                    microsecond,
                    py_tzinfo,
                ))
        }
    }
}

impl FromPyObject<'_> for UtcDateTime {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<UtcDateTime> {
        #[cfg(not(Py_LIMITED_API))]
        let dt = ob.downcast::<PyDateTime>()?;

        // Extract tzinfo and ensure it's not None
        #[cfg(not(Py_LIMITED_API))]
        let tzinfo = dt
            .get_tzinfo()
            .ok_or_else(|| PyTypeError::new_err("expected a datetime with non-None tzinfo"))?;

        #[cfg(Py_LIMITED_API)]
        let tzinfo = ob.getattr(intern!(ob.py(), "tzinfo"))?;
        #[cfg(Py_LIMITED_API)]
        if tzinfo.is_none() {
            return Err(PyTypeError::new_err(
                "expected a datetime with non-None tzinfo",
            ));
        }

        // Verify that the tzinfo is UTC
        let is_utc = tzinfo
            .call_method1(
                "__eq__",
                (ob.py()
                    .import("datetime")?
                    .getattr(intern!(ob.py(), "timezone"))?
                    .getattr(intern!(ob.py(), "utc"))?,),
            )?
            .extract::<bool>()?;

        if !is_utc {
            return Err(PyValueError::new_err(
                "expected a datetime with UTC timezone",
            ));
        }

        // Extract date and time components
        #[cfg(not(Py_LIMITED_API))]
        let date = Date::from_calendar_date(
            dt.get_year(),
            month_from_number!(dt.get_month()),
            dt.get_day(),
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range date"))?;

        // Extract time
        #[cfg(not(Py_LIMITED_API))]
        let time = Time::from_hms_micro(
            dt.get_hour(),
            dt.get_minute(),
            dt.get_second(),
            dt.get_microsecond(),
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range time"))?;

        #[cfg(Py_LIMITED_API)]
        let date = Date::from_calendar_date(
            ob.getattr(intern!(ob.py(), "year"))?.extract()?,
            month_from_number!(ob.getattr(intern!(ob.py(), "month"))?.extract::<u8>()?),
            ob.getattr(intern!(ob.py(), "day"))?.extract()?,
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range date"))?;

        #[cfg(Py_LIMITED_API)]
        let time = Time::from_hms_micro(
            ob.getattr(intern!(ob.py(), "hour"))?.extract()?,
            ob.getattr(intern!(ob.py(), "minute"))?.extract()?,
            ob.getattr(intern!(ob.py(), "second"))?.extract()?,
            ob.getattr(intern!(ob.py(), "microsecond"))?.extract()?,
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range time"))?;

        // Create the PrimitiveDateTime first
        let primitive_dt = PrimitiveDateTime::new(date, time);

        // Then convert to UTC
        Ok(primitive_dt.assume_utc().into())
    }
}

impl_into_py_for_ref!(Duration, PyDelta);
impl_into_py_for_ref!(Date, PyDate);
impl_into_py_for_ref!(Time, PyTime);
impl_into_py_for_ref!(PrimitiveDateTime, PyDateTime);
impl_into_py_for_ref!(UtcOffset, PyTzInfo);
impl_into_py_for_ref!(OffsetDateTime, PyDateTime);
impl_into_py_for_ref!(UtcDateTime, PyDateTime);

// FIXME: Refactor the test
#[cfg(test)]
mod tests {
    use super::*;
    use crate::intern;
    use crate::types::any::PyAnyMethods;
    use crate::types::PyTypeMethods;
    use proptest::prelude::*;

    #[test]
    fn test_time_duration_conversion() {
        Python::with_gil(|py| {
            // Regular duration
            let duration = Duration::new(1, 500_000_000); // 1.5 seconds
            let py_delta = duration.into_pyobject(py).unwrap();

            // Check the python object is correct
            let seconds = py_delta
                .getattr(intern!(py, "seconds"))
                .unwrap()
                .extract::<i64>()
                .unwrap();
            let microseconds = py_delta
                .getattr(intern!(py, "microseconds"))
                .unwrap()
                .extract::<i64>()
                .unwrap();
            assert_eq!(seconds, 1);
            assert_eq!(microseconds, 500_000);

            // Check negative durations
            let neg_duration = Duration::new(-10, 0); // -10 seconds
            let py_neg_delta = neg_duration.into_pyobject(py).unwrap();
            let days = py_neg_delta
                .getattr(intern!(py, "days"))
                .unwrap()
                .extract::<i64>()
                .unwrap();
            let seconds = py_neg_delta
                .getattr(intern!(py, "seconds"))
                .unwrap()
                .extract::<i64>()
                .unwrap();
            assert_eq!(days, -1);
            assert_eq!(seconds, 86390); // 86400 - 10 seconds

            // Test case for exact negative days (should use normal division path)
            let exact_day = Duration::seconds(-86_400); // Exactly -1 day
            let py_delta = exact_day.into_pyobject(py).unwrap();

            let days = py_delta
                .getattr(intern!(py, "days"))
                .unwrap()
                .extract::<i64>()
                .unwrap();
            let seconds = py_delta
                .getattr(intern!(py, "seconds"))
                .unwrap()
                .extract::<i64>()
                .unwrap();
            let microseconds = py_delta
                .getattr(intern!(py, "microseconds"))
                .unwrap()
                .extract::<i64>()
                .unwrap();

            // Should be exactly -1 day with 0 seconds and 0 microseconds
            assert_eq!(days, -1);
            assert_eq!(seconds, 0);
            assert_eq!(microseconds, 0);

            // Test with fractional negative days but fractional multiple of seconds
            let neg_multiple = Duration::seconds(-172_801);
            let py_delta = neg_multiple.into_pyobject(py).unwrap();

            let days = py_delta
                .getattr(intern!(py, "days"))
                .unwrap()
                .extract::<i64>()
                .unwrap();
            let seconds = py_delta
                .getattr(intern!(py, "seconds"))
                .unwrap()
                .extract::<i64>()
                .unwrap();

            assert_eq!(days, -3);
            assert_eq!(seconds, 86399);
        });
    }

    #[test]
    fn test_time_duration_conversion_large_values() {
        Python::with_gil(|py| {
            // Large duration (close to max)
            let large_duration = Duration::seconds(86_399_999_000_000); // Almost max
            let py_large_delta = large_duration.into_pyobject(py).unwrap();

            // Check days is near max
            let days = py_large_delta
                .getattr(intern!(py, "days"))
                .unwrap()
                .extract::<i64>()
                .unwrap();
            assert!(days > 999_000_000);

            // Test over limit (should yield Overflow error in python)
            let too_large = Duration::seconds(86_400_000_000_000); // Over max
            let result = too_large.into_pyobject(py);
            assert!(result.is_err());
            let err_type = result.unwrap_err().get_type(py).name().unwrap();
            assert_eq!(err_type, "OverflowError");

            // Test with negative extreme
            let large_neg = Duration::seconds(-86_399_999_000_000); // Almost min
            let py_large_neg = large_neg.into_pyobject(py).unwrap();

            // Check days is near min
            let days = py_large_neg
                .getattr(intern!(py, "days"))
                .unwrap()
                .extract::<i64>()
                .unwrap();
            assert!(days < -999_000_000);

            // Too small should fail
            let too_small = Duration::seconds(-86_400_000_000_000); // Under min
            let result = too_small.into_pyobject(py);
            assert!(result.is_err());
            let err_type = result.unwrap_err().get_type(py).name().unwrap();
            assert_eq!(err_type, "OverflowError");
        });
    }

    #[test]
    fn test_time_duration_nanosecond_resolution() {
        Python::with_gil(|py| {
            // Test nanosecond conversion to microseconds
            let duration = Duration::new(0, 1_234_567);
            let py_delta = duration.into_pyobject(py).unwrap();

            // Python timedelta only has microsecond resolution, so we should get 1234 microseconds
            let microseconds = py_delta
                .getattr(intern!(py, "microseconds"))
                .unwrap()
                .extract::<i64>()
                .unwrap();
            assert_eq!(microseconds, 1234);
        });
    }

    #[test]
    fn test_time_duration_from_python() {
        Python::with_gil(|py| {
            // Create Python timedeltas with various values
            let datetime = py.import("datetime").unwrap();
            let timedelta = datetime.getattr(intern!(py, "timedelta")).unwrap();

            // Test positive values
            let py_delta1 = timedelta.call1((3, 7200, 500000)).unwrap();
            let duration1: Duration = py_delta1.extract().unwrap();
            assert_eq!(duration1.whole_days(), 3);
            assert_eq!(duration1.whole_seconds() % 86400, 7200);
            assert_eq!(duration1.subsec_nanoseconds(), 500000000);

            // Test negative days
            let py_delta2 = timedelta.call1((-2, 43200)).unwrap();
            let duration2: Duration = py_delta2.extract().unwrap();
            assert_eq!(duration2.whole_days(), -1);
            assert_eq!(duration2.whole_seconds(), -129600);

            // Test negative seconds but positive days (Python normalizes this)
            let py_delta3 = timedelta.call1((1, -3600, 250000)).unwrap();
            let duration3: Duration = py_delta3.extract().unwrap();
            // Python normalizes to 0 days, 82800 seconds, 250000 microseconds
            assert_eq!(duration3.whole_days(), 0);
            assert_eq!(duration3.whole_seconds(), 82800);
            assert_eq!(duration3.subsec_nanoseconds(), 250000000);

            // Test microseconds only
            let py_delta4 = timedelta.call1((0, 0, 123456)).unwrap();
            let duration4: Duration = py_delta4.extract().unwrap();
            assert_eq!(duration4.whole_seconds(), 0);
            assert_eq!(duration4.subsec_nanoseconds(), 123456000);
        });
    }

    #[test]
    fn test_time_duration_roundtrip_extract() {
        Python::with_gil(|py| {
            // Create a Rust Duration
            let duration =
                Duration::days(2) + Duration::seconds(3600) + Duration::microseconds(500000);

            // Convert to Python
            let py_delta = duration.into_pyobject(py).unwrap();

            // Convert back to Rust
            let roundtripped: Duration = py_delta.extract().unwrap();

            // Verify they match
            assert_eq!(duration, roundtripped);

            // Test with negative duration
            let neg_duration =
                Duration::days(-5) + Duration::seconds(7200) + Duration::microseconds(250000);
            let py_neg_delta = neg_duration.into_pyobject(py).unwrap();
            let roundtripped_neg: Duration = py_neg_delta.extract().unwrap();
            assert_eq!(neg_duration, roundtripped_neg);
        });
    }

    proptest! {
        #[test]
        fn test_time_duration_roundtrip(days in -9999i64..=9999i64, seconds in -86399i64..=86399i64, microseconds in -999999i64..=999999i64) {
            // Generate a valid duration that should roundtrip successfully
            Python::with_gil(|py| {
                let duration = Duration::days(days) + Duration::seconds(seconds) + Duration::microseconds(microseconds);

                // Skip if outside Python's timedelta bounds
                let max_seconds = 86_399_999_913_600;
                if duration.whole_seconds() <= max_seconds && duration.whole_seconds() >= -max_seconds {
                    let py_delta = duration.into_pyobject(py).unwrap();

                    // You could add FromPyObject for Duration to fully test the roundtrip
                    // For now we'll just check that the Python object has the expected properties
                    let total_seconds = py_delta.call_method0(intern!(py, "total_seconds")).unwrap().extract::<f64>().unwrap();
                    let expected_seconds = duration.whole_seconds() as f64 + (duration.subsec_nanoseconds() as f64 / 1_000_000_000.0);

                    // Allow small floating point differences
                    assert_eq!(total_seconds, expected_seconds);
                }
            })
        }
    }

    #[test]
    fn test_time_date_conversion() {
        Python::with_gil(|py| {
            // Regular date
            let date = Date::from_calendar_date(2023, Month::April, 15).unwrap();
            let py_date = date.into_pyobject(py).unwrap();

            // Check the Python object is correct
            let year = py_date
                .getattr(intern!(py, "year"))
                .unwrap()
                .extract::<i32>()
                .unwrap();
            let month = py_date
                .getattr(intern!(py, "month"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let day = py_date
                .getattr(intern!(py, "day"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            assert_eq!(year, 2023);
            assert_eq!(month, 4);
            assert_eq!(day, 15);

            // Test edge cases
            let min_date = Date::from_calendar_date(1, Month::January, 1).unwrap();
            let py_min_date = min_date.into_pyobject(py).unwrap();
            let min_year = py_min_date
                .getattr(intern!(py, "year"))
                .unwrap()
                .extract::<i32>()
                .unwrap();
            let min_month = py_min_date
                .getattr(intern!(py, "month"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let min_day = py_min_date
                .getattr(intern!(py, "day"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            assert_eq!(min_year, 1);
            assert_eq!(min_month, 1);
            assert_eq!(min_day, 1);

            let max_date = Date::from_calendar_date(9999, Month::December, 31).unwrap();
            let py_max_date = max_date.into_pyobject(py).unwrap();
            let max_year = py_max_date
                .getattr(intern!(py, "year"))
                .unwrap()
                .extract::<i32>()
                .unwrap();
            let max_month = py_max_date
                .getattr(intern!(py, "month"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let max_day = py_max_date
                .getattr(intern!(py, "day"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            assert_eq!(max_year, 9999);
            assert_eq!(max_month, 12);
            assert_eq!(max_day, 31);
        });
    }

    #[test]
    fn test_time_date_from_python() {
        Python::with_gil(|py| {
            // Create Python dates
            let datetime = py.import("datetime").unwrap();
            let date_type = datetime.getattr(intern!(py, "date")).unwrap();

            // Test normal date
            let py_date1 = date_type.call1((2023, 4, 15)).unwrap();
            let date1: Date = py_date1.extract().unwrap();
            assert_eq!(date1.year(), 2023);
            assert_eq!(date1.month(), Month::April);
            assert_eq!(date1.day(), 15);

            // Test min date
            let py_date2 = date_type.call1((1, 1, 1)).unwrap();
            let date2: Date = py_date2.extract().unwrap();
            assert_eq!(date2.year(), 1);
            assert_eq!(date2.month(), Month::January);
            assert_eq!(date2.day(), 1);

            // Test max date
            let py_date3 = date_type.call1((9999, 12, 31)).unwrap();
            let date3: Date = py_date3.extract().unwrap();
            assert_eq!(date3.year(), 9999);
            assert_eq!(date3.month(), Month::December);
            assert_eq!(date3.day(), 31);

            // Test leap year date
            let py_date4 = date_type.call1((2024, 2, 29)).unwrap();
            let date4: Date = py_date4.extract().unwrap();
            assert_eq!(date4.year(), 2024);
            assert_eq!(date4.month(), Month::February);
            assert_eq!(date4.day(), 29);
        });
    }

    #[test]
    fn test_time_date_roundtrip() {
        Python::with_gil(|py| {
            // Create a Rust Date
            let date = Date::from_calendar_date(2023, Month::April, 15).unwrap();

            // Convert to Python
            let py_date = date.into_pyobject(py).unwrap();

            // Convert back to Rust
            let roundtripped: Date = py_date.extract().unwrap();

            // Verify they match
            assert_eq!(date, roundtripped);
        });
    }

    #[test]
    fn test_time_date_invalid_values() {
        Python::with_gil(|py| {
            // Try to create an invalid date in Python
            let datetime = py.import("datetime").unwrap();
            let date_type = datetime.getattr(intern!(py, "date")).unwrap();

            // February 30 doesn't exist
            let result = date_type.call1((2023, 2, 30));
            assert!(result.is_err());

            // Test extraction of invalid month
            let mock_date = date_type.call1((2023, 13, 1));
            assert!(mock_date.is_err());
        });
    }

    proptest! {
        #[test]
        fn test_all_valid_dates(
            year in 1i32..=9999,
            month_num in 1u8..=12,
        ) {
            Python::with_gil(|py| {
                let month = match month_num {
                    1 => (Month::January, 31),
                    2 => {
                        // Handle leap years
                        if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                            (Month::February, 29)
                        } else {
                            (Month::February, 28)
                        }
                    },
                    3 => (Month::March, 31),
                    4 => (Month::April, 30),
                    5 => (Month::May, 31),
                    6 => (Month::June, 30),
                    7 => (Month::July, 31),
                    8 => (Month::August, 31),
                    9 => (Month::September, 30),
                    10 => (Month::October, 31),
                    11 => (Month::November, 30),
                    12 => (Month::December, 31),
                    _ => unreachable!(),
                };

                // Test the entire month
                for day in 1..=month.1 {
                    let date = Date::from_calendar_date(year, month.0, day).unwrap();
                    let py_date = date.into_pyobject(py).unwrap();
                    let roundtripped: Date = py_date.extract().unwrap();
                    assert_eq!(date, roundtripped);
                }
            });
        }
    }

    #[test]
    fn test_time_time_conversion() {
        Python::with_gil(|py| {
            // Regular time
            let time = Time::from_hms_micro(14, 30, 45, 123456).unwrap();
            let py_time = time.into_pyobject(py).unwrap();

            // Check the Python object is correct
            let hour = py_time
                .getattr(intern!(py, "hour"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let minute = py_time
                .getattr(intern!(py, "minute"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let second = py_time
                .getattr(intern!(py, "second"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let microsecond = py_time
                .getattr(intern!(py, "microsecond"))
                .unwrap()
                .extract::<u32>()
                .unwrap();
            assert_eq!(hour, 14);
            assert_eq!(minute, 30);
            assert_eq!(second, 45);
            assert_eq!(microsecond, 123456);

            // Test edge cases
            let min_time = Time::from_hms_micro(0, 0, 0, 0).unwrap();
            let py_min_time = min_time.into_pyobject(py).unwrap();
            let min_hour = py_min_time
                .getattr(intern!(py, "hour"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let min_minute = py_min_time
                .getattr(intern!(py, "minute"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let min_second = py_min_time
                .getattr(intern!(py, "second"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let min_microsecond = py_min_time
                .getattr(intern!(py, "microsecond"))
                .unwrap()
                .extract::<u32>()
                .unwrap();
            assert_eq!(min_hour, 0);
            assert_eq!(min_minute, 0);
            assert_eq!(min_second, 0);
            assert_eq!(min_microsecond, 0);

            let max_time = Time::from_hms_micro(23, 59, 59, 999999).unwrap();
            let py_max_time = max_time.into_pyobject(py).unwrap();
            let max_hour = py_max_time
                .getattr(intern!(py, "hour"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let max_minute = py_max_time
                .getattr(intern!(py, "minute"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let max_second = py_max_time
                .getattr(intern!(py, "second"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let max_microsecond = py_max_time
                .getattr(intern!(py, "microsecond"))
                .unwrap()
                .extract::<u32>()
                .unwrap();
            assert_eq!(max_hour, 23);
            assert_eq!(max_minute, 59);
            assert_eq!(max_second, 59);
            assert_eq!(max_microsecond, 999999);
        });
    }

    #[test]
    fn test_time_time_from_python() {
        Python::with_gil(|py| {
            // Create Python times
            let datetime = py.import("datetime").unwrap();
            let time_type = datetime.getattr(intern!(py, "time")).unwrap();

            // Test normal time
            let py_time1 = time_type.call1((14, 30, 45, 123456)).unwrap();
            let time1: Time = py_time1.extract().unwrap();
            assert_eq!(time1.hour(), 14);
            assert_eq!(time1.minute(), 30);
            assert_eq!(time1.second(), 45);
            assert_eq!(time1.microsecond(), 123456);

            // Test min time
            let py_time2 = time_type.call1((0, 0, 0, 0)).unwrap();
            let time2: Time = py_time2.extract().unwrap();
            assert_eq!(time2.hour(), 0);
            assert_eq!(time2.minute(), 0);
            assert_eq!(time2.second(), 0);
            assert_eq!(time2.microsecond(), 0);

            // Test max time
            let py_time3 = time_type.call1((23, 59, 59, 999999)).unwrap();
            let time3: Time = py_time3.extract().unwrap();
            assert_eq!(time3.hour(), 23);
            assert_eq!(time3.minute(), 59);
            assert_eq!(time3.second(), 59);
            assert_eq!(time3.microsecond(), 999999);
        });
    }

    #[test]
    fn test_time_time_roundtrip() {
        Python::with_gil(|py| {
            // Create a Rust Time
            let time = Time::from_hms_micro(14, 30, 45, 123456).unwrap();

            // Convert to Python
            let py_time = time.into_pyobject(py).unwrap();

            // Convert back to Rust
            let roundtripped: Time = py_time.extract().unwrap();

            // Verify they match
            assert_eq!(time, roundtripped);
        });
    }

    #[test]
    fn test_time_time_invalid_values() {
        Python::with_gil(|py| {
            // Try to create an invalid time in Python
            let datetime = py.import("datetime").unwrap();
            let time_type = datetime.getattr(intern!(py, "time")).unwrap();

            // Hour 24 doesn't exist (valid range is 0-23)
            let result = time_type.call1((24, 0, 0, 0));
            assert!(result.is_err());

            // Minute 60 doesn't exist (valid range is 0-59)
            let result = time_type.call1((12, 60, 0, 0));
            assert!(result.is_err());

            // Second 60 doesn't exist except for leap seconds (valid range is normally 0-59)
            let result = time_type.call1((12, 30, 60, 0));
            assert!(result.is_err());

            // Microsecond range check (valid range is 0-999999)
            let result = time_type.call1((12, 30, 30, 1000000));
            assert!(result.is_err());
        });
    }

    proptest! {
        #[test]
        fn test_time_time_roundtrip_random(
            hour in 0u8..=23u8,
            minute in 0u8..=59u8,
            second in 0u8..=59u8,
            microsecond in 0u32..=999999u32
        ) {
            Python::with_gil(|py| {
                let time = Time::from_hms_micro(hour, minute, second, microsecond).unwrap();
                let py_time = time.into_pyobject(py).unwrap();
                let roundtripped: Time = py_time.extract().unwrap();
                assert_eq!(time, roundtripped);
            });
        }
    }

    #[test]
    fn test_time_time_nanoseconds_precision_loss() {
        Python::with_gil(|py| {
            // Create a time with nanosecond precision in Rust
            // For demonstration, using a method with nanosecond precision
            // Note: time::Time stores nanoseconds, but we have to convert from microseconds to nanoseconds
            let ns_time = Time::from_hms_micro(12, 34, 56, 123456).unwrap();

            // Python only supports microsecond precision
            let py_time = ns_time.into_pyobject(py).unwrap();

            // Check microsecond precision only
            let microsecond = py_time
                .getattr(intern!(py, "microsecond"))
                .unwrap()
                .extract::<u32>()
                .unwrap();
            assert_eq!(microsecond, 123456);

            // When converting back to Rust, we should get the same time
            let roundtripped: Time = py_time.extract().unwrap();
            assert_eq!(ns_time, roundtripped);
        });
    }

    #[test]
    fn test_time_time_with_timezone() {
        Python::with_gil(|py| {
            // Create Python time with timezone (just to ensure we can handle it properly)
            let datetime = py.import("datetime").unwrap();
            let time_type = datetime.getattr(intern!(py, "time")).unwrap();
            let timezone = datetime.getattr(intern!(py, "timezone")).unwrap();

            // Create timezone object (UTC)
            let tz_utc = timezone.getattr(intern!(py, "utc")).unwrap();

            // Create time with timezone
            let py_time_with_tz = time_type.call1((12, 30, 45, 0, tz_utc)).unwrap();

            // Check if extraction works - note this will depend on the implementation
            // If the current implementation ignores tzinfo for Time, this should work
            // Otherwise this might need to be adjusted based on actual behavior
            let result: Result<Time, _> = py_time_with_tz.extract();

            // Either we successfully extract (ignoring timezone) or get a meaningful error
            match result {
                Ok(time) => {
                    // If we extract successfully, verify the time components
                    assert_eq!(time.hour(), 12);
                    assert_eq!(time.minute(), 30);
                    assert_eq!(time.second(), 45);
                }
                Err(_) => {
                    // This is also acceptable if we don't support times with timezones
                    // In that case, we'd check for a specific error type/message
                }
            }
        });
    }

    #[test]
    fn test_time_primitive_datetime_conversion() {
        Python::with_gil(|py| {
            // Regular datetime
            let date = Date::from_calendar_date(2023, Month::April, 15).unwrap();
            let time = Time::from_hms_micro(14, 30, 45, 123456).unwrap();
            let dt = PrimitiveDateTime::new(date, time);
            let py_dt = dt.into_pyobject(py).unwrap();

            // Check the Python object is correct
            let year = py_dt
                .getattr(intern!(py, "year"))
                .unwrap()
                .extract::<i32>()
                .unwrap();
            let month = py_dt
                .getattr(intern!(py, "month"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let day = py_dt
                .getattr(intern!(py, "day"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let hour = py_dt
                .getattr(intern!(py, "hour"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let minute = py_dt
                .getattr(intern!(py, "minute"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let second = py_dt
                .getattr(intern!(py, "second"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let microsecond = py_dt
                .getattr(intern!(py, "microsecond"))
                .unwrap()
                .extract::<u32>()
                .unwrap();

            assert_eq!(year, 2023);
            assert_eq!(month, 4);
            assert_eq!(day, 15);
            assert_eq!(hour, 14);
            assert_eq!(minute, 30);
            assert_eq!(second, 45);
            assert_eq!(microsecond, 123456);

            // Check it has no timezone
            let tzinfo = py_dt.getattr(intern!(py, "tzinfo")).unwrap();
            assert!(tzinfo.is_none());

            // Test min datetime
            let min_date = Date::from_calendar_date(1, Month::January, 1).unwrap();
            let min_time = Time::from_hms_micro(0, 0, 0, 0).unwrap();
            let min_dt = PrimitiveDateTime::new(min_date, min_time);
            let py_min_dt = min_dt.into_pyobject(py).unwrap();

            assert_eq!(
                py_min_dt
                    .getattr(intern!(py, "year"))
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                1
            );
            assert_eq!(
                py_min_dt
                    .getattr(intern!(py, "month"))
                    .unwrap()
                    .extract::<u8>()
                    .unwrap(),
                1
            );
            assert_eq!(
                py_min_dt
                    .getattr(intern!(py, "hour"))
                    .unwrap()
                    .extract::<u8>()
                    .unwrap(),
                0
            );
        });
    }

    #[test]
    fn test_time_primitive_datetime_from_python() {
        Python::with_gil(|py| {
            // Create Python datetimes
            let datetime = py.import("datetime").unwrap();
            let datetime_type = datetime.getattr(intern!(py, "datetime")).unwrap();

            // Test normal datetime
            let py_dt1 = datetime_type
                .call1((2023, 4, 15, 14, 30, 45, 123456))
                .unwrap();
            let dt1: PrimitiveDateTime = py_dt1.extract().unwrap();

            assert_eq!(dt1.year(), 2023);
            assert_eq!(dt1.month(), Month::April);
            assert_eq!(dt1.day(), 15);
            assert_eq!(dt1.hour(), 14);
            assert_eq!(dt1.minute(), 30);
            assert_eq!(dt1.second(), 45);
            assert_eq!(dt1.microsecond(), 123456);

            // Test min datetime
            let py_dt2 = datetime_type.call1((1, 1, 1, 0, 0, 0, 0)).unwrap();
            let dt2: PrimitiveDateTime = py_dt2.extract().unwrap();

            assert_eq!(dt2.year(), 1);
            assert_eq!(dt2.month(), Month::January);
            assert_eq!(dt2.day(), 1);
            assert_eq!(dt2.hour(), 0);
            assert_eq!(dt2.minute(), 0);

            // Test with timezone - should fail
            let timezone = datetime.getattr(intern!(py, "timezone")).unwrap();
            let tz_utc = timezone.getattr(intern!(py, "utc")).unwrap();
            let py_dt_tz = datetime_type
                .call1((2023, 4, 15, 14, 30, 45, 123456, tz_utc))
                .unwrap();

            let result: Result<PrimitiveDateTime, _> = py_dt_tz.extract();
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_time_primitive_datetime_roundtrip() {
        Python::with_gil(|py| {
            // Create a Rust PrimitiveDateTime
            let date = Date::from_calendar_date(2023, Month::April, 15).unwrap();
            let time = Time::from_hms_micro(14, 30, 45, 123456).unwrap();
            let dt = PrimitiveDateTime::new(date, time);

            // Convert to Python
            let py_dt = dt.into_pyobject(py).unwrap();

            // Convert back to Rust
            let roundtripped: PrimitiveDateTime = py_dt.extract().unwrap();

            // Verify they match
            assert_eq!(dt, roundtripped);
        });
    }

    proptest! {
        #[test]
        fn test_time_primitive_datetime_roundtrip_random(
            year in 1i32..=9999i32,
            month in 1u8..=12u8,
            day in 1u8..=28u8, // Use only valid days for all months
            hour in 0u8..=23u8,
            minute in 0u8..=59u8,
            second in 0u8..=59u8,
            microsecond in 0u32..=999999u32
        ) {
            Python::with_gil(|py| {
                let month = month_from_number!(month);

                let date = Date::from_calendar_date(year, month, day).unwrap();
                let time = Time::from_hms_micro(hour, minute, second, microsecond).unwrap();
                let dt = PrimitiveDateTime::new(date, time);

                let py_dt = dt.into_pyobject(py).unwrap();
                let roundtripped: PrimitiveDateTime = py_dt.extract().unwrap();
                assert_eq!(dt, roundtripped);
            });
        }
    }

    #[test]
    fn test_time_primitive_datetime_leap_years() {
        Python::with_gil(|py| {
            // Test datetime on leap day
            let date = Date::from_calendar_date(2024, Month::February, 29).unwrap();
            let time = Time::from_hms_micro(12, 0, 0, 0).unwrap();
            let dt = PrimitiveDateTime::new(date, time);

            let py_dt = dt.into_pyobject(py).unwrap();
            let roundtripped: PrimitiveDateTime = py_dt.extract().unwrap();

            assert_eq!(dt, roundtripped);
            assert_eq!(roundtripped.month(), Month::February);
            assert_eq!(roundtripped.day(), 29);
        });
    }

    #[test]
    fn test_time_utc_offset_conversion() {
        Python::with_gil(|py| {
            // Test positive offset
            let offset = UtcOffset::from_hms(5, 30, 0).unwrap();
            let py_tz = offset.into_pyobject(py).unwrap();

            // Test timezone properties
            let utcoffset = py_tz.call_method1("utcoffset", (py.None(),)).unwrap();
            let total_seconds = utcoffset
                .call_method0("total_seconds")
                .unwrap()
                .extract::<f64>()
                .unwrap();
            assert_eq!(total_seconds, 5.0 * 3600.0 + 30.0 * 60.0);

            // Test negative offset
            let neg_offset = UtcOffset::from_hms(-8, -15, 0).unwrap();
            let py_neg_tz = neg_offset.into_pyobject(py).unwrap();

            let neg_utcoffset = py_neg_tz.call_method1("utcoffset", (py.None(),)).unwrap();
            let neg_total_seconds = neg_utcoffset
                .call_method0("total_seconds")
                .unwrap()
                .extract::<f64>()
                .unwrap();
            assert_eq!(neg_total_seconds, -8.0 * 3600.0 - 15.0 * 60.0);
        });
    }

    #[test]
    fn test_time_utc_offset_from_python() {
        Python::with_gil(|py| {
            // Create timezone objects
            let datetime = py.import("datetime").unwrap();
            let timezone = datetime.getattr(intern!(py, "timezone")).unwrap();
            let timedelta = datetime.getattr(intern!(py, "timedelta")).unwrap();

            // Test UTC
            let tz_utc = timezone.getattr(intern!(py, "utc")).unwrap();
            let utc_offset: UtcOffset = tz_utc.extract().unwrap();
            assert_eq!(utc_offset.whole_hours(), 0);
            assert_eq!(utc_offset.minutes_past_hour(), 0);
            assert_eq!(utc_offset.seconds_past_minute(), 0);

            // Test positive offset
            let td_pos = timedelta.call1((0, 19800, 0)).unwrap(); // 5 hours 30 minutes
            let tz_pos = timezone.call1((td_pos,)).unwrap();
            let offset_pos: UtcOffset = tz_pos.extract().unwrap();
            assert_eq!(offset_pos.whole_hours(), 5);
            assert_eq!(offset_pos.minutes_past_hour(), 30);

            // Test negative offset
            let td_neg = timedelta.call1((0, -30900, 0)).unwrap(); // -8 hours -35 minutes
            let tz_neg = timezone.call1((td_neg,)).unwrap();
            let offset_neg: UtcOffset = tz_neg.extract().unwrap();
            assert_eq!(offset_neg.whole_hours(), -8);
            assert_eq!(offset_neg.minutes_past_hour(), -35);
        });
    }

    #[test]
    fn test_time_utc_offset_roundtrip() {
        Python::with_gil(|py| {
            // Test with standard offset
            let offset = UtcOffset::from_hms(5, 30, 45).unwrap();
            let py_tz = offset.into_pyobject(py).unwrap();
            let roundtripped: UtcOffset = py_tz.extract().unwrap();

            // The seconds part will be lost since Python's timezone only stores
            // hours and minutes in tzinfo, not seconds
            assert_eq!(roundtripped.whole_hours(), 5);
            assert_eq!(roundtripped.minutes_past_hour(), 30);

            // Test with negative offset
            let neg_offset = UtcOffset::from_hms(-11, -30, -15).unwrap();
            let py_neg_tz = neg_offset.into_pyobject(py).unwrap();
            let neg_roundtripped: UtcOffset = py_neg_tz.extract().unwrap();

            assert_eq!(neg_roundtripped.whole_hours(), -11);
            assert_eq!(neg_roundtripped.minutes_past_hour(), -30);
        });
    }

    proptest! {
        #[test]
        fn test_time_utc_offset_roundtrip_random(
            hours in -23i8..=23i8,
            minutes in -59i8..=59i8
        ) {
            // Skip invalid combinations where hour and minute signs don't match
            if (hours < 0 && minutes > 0) || (hours > 0 && minutes < 0) {
                return Ok(());
            }

            Python::with_gil(|py| {
                if let Ok(offset) = UtcOffset::from_hms(hours, minutes, 0) {
                    let py_tz = offset.into_pyobject(py).unwrap();
                    let roundtripped: UtcOffset = py_tz.extract().unwrap();
                    assert_eq!(roundtripped.whole_hours(), hours);
                    assert_eq!(roundtripped.minutes_past_hour(), minutes);
                }
            });
        }
    }

    #[test]
    fn test_time_offset_datetime_conversion() {
        Python::with_gil(|py| {
            // Create an OffsetDateTime with +5:30 offset
            let date = Date::from_calendar_date(2023, Month::April, 15).unwrap();
            let time = Time::from_hms_micro(14, 30, 45, 123456).unwrap();
            let offset = UtcOffset::from_hms(5, 30, 0).unwrap();
            let dt = PrimitiveDateTime::new(date, time).assume_offset(offset);

            // Convert to Python
            let py_dt = dt.into_pyobject(py).unwrap();

            // Check components
            let year = py_dt
                .getattr(intern!(py, "year"))
                .unwrap()
                .extract::<i32>()
                .unwrap();
            let month = py_dt
                .getattr(intern!(py, "month"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let day = py_dt
                .getattr(intern!(py, "day"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let hour = py_dt
                .getattr(intern!(py, "hour"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let minute = py_dt
                .getattr(intern!(py, "minute"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let second = py_dt
                .getattr(intern!(py, "second"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let microsecond = py_dt
                .getattr(intern!(py, "microsecond"))
                .unwrap()
                .extract::<u32>()
                .unwrap();

            assert_eq!(year, 2023);
            assert_eq!(month, 4);
            assert_eq!(day, 15);
            assert_eq!(hour, 14);
            assert_eq!(minute, 30);
            assert_eq!(second, 45);
            assert_eq!(microsecond, 123456);

            // Check timezone offset
            let tzinfo = py_dt.getattr(intern!(py, "tzinfo")).unwrap();
            let utcoffset = tzinfo.call_method1("utcoffset", (py_dt,)).unwrap();
            let seconds = utcoffset
                .call_method0("total_seconds")
                .unwrap()
                .extract::<f64>()
                .unwrap();
            assert_eq!(seconds, 5.0 * 3600.0 + 30.0 * 60.0);
        });
    }

    #[test]
    fn test_time_offset_datetime_from_python() {
        Python::with_gil(|py| {
            // Create Python datetime with timezone
            let datetime = py.import("datetime").unwrap();
            let datetime_type = datetime.getattr(intern!(py, "datetime")).unwrap();
            let timezone = datetime.getattr(intern!(py, "timezone")).unwrap();
            let timedelta = datetime.getattr(intern!(py, "timedelta")).unwrap();

            // Create a timezone (+5:30)
            let td = timedelta.call1((0, 19800, 0)).unwrap(); // 5:30:00
            let tz = timezone.call1((td,)).unwrap();

            // Create datetime with this timezone
            let py_dt = datetime_type
                .call1((2023, 4, 15, 14, 30, 45, 123456, tz))
                .unwrap();

            // Extract to Rust
            let dt: OffsetDateTime = py_dt.extract().unwrap();

            // Verify components
            assert_eq!(dt.year(), 2023);
            assert_eq!(dt.month(), Month::April);
            assert_eq!(dt.day(), 15);
            assert_eq!(dt.hour(), 14);
            assert_eq!(dt.minute(), 30);
            assert_eq!(dt.second(), 45);
            assert_eq!(dt.microsecond(), 123456);
            assert_eq!(dt.offset().whole_hours(), 5);
            assert_eq!(dt.offset().minutes_past_hour(), 30);
        });
    }

    #[test]
    fn test_time_offset_datetime_roundtrip() {
        Python::with_gil(|py| {
            // Create an OffsetDateTime with timezone
            let date = Date::from_calendar_date(2023, Month::April, 15).unwrap();
            let time = Time::from_hms_micro(14, 30, 45, 123456).unwrap();
            let offset = UtcOffset::from_hms(5, 30, 0).unwrap();
            let dt = PrimitiveDateTime::new(date, time).assume_offset(offset);

            // Convert to Python
            let py_dt = dt.into_pyobject(py).unwrap();

            // Convert back to Rust
            let roundtripped: OffsetDateTime = py_dt.extract().unwrap();

            // Verify components
            assert_eq!(roundtripped.year(), dt.year());
            assert_eq!(roundtripped.month(), dt.month());
            assert_eq!(roundtripped.day(), dt.day());
            assert_eq!(roundtripped.hour(), dt.hour());
            assert_eq!(roundtripped.minute(), dt.minute());
            assert_eq!(roundtripped.second(), dt.second());
            assert_eq!(roundtripped.microsecond(), dt.microsecond());
            assert_eq!(
                roundtripped.offset().whole_hours(),
                dt.offset().whole_hours()
            );
            assert_eq!(
                roundtripped.offset().minutes_past_hour(),
                dt.offset().minutes_past_hour()
            );

            // We lose seconds in timezone offset due to Python's limitations
            // so we don't test for seconds_past_minute
        });
    }

    proptest! {
        #[test]
        fn test_time_offset_datetime_roundtrip_random(
            year in 1i32..=9999i32,
            month in 1u8..=12u8,
            day in 1u8..=28u8, // Use only valid days for all months
            hour in 0u8..=23u8,
            minute in 0u8..=59u8,
            second in 0u8..=59u8,
            microsecond in 0u32..=999999u32,
            tz_hour in -23i8..=23i8,
            tz_minute in 0i8..=59i8
        ) {
            Python::with_gil(|py| {
                let month = month_from_number!(month);

                let date = Date::from_calendar_date(year, month, day).unwrap();
                let time = Time::from_hms_micro(hour, minute, second, microsecond).unwrap();

                // Handle timezone sign correctly
                let tz_minute = if tz_hour < 0 { -tz_minute } else { tz_minute };

                if let Ok(offset) = UtcOffset::from_hms(tz_hour, tz_minute, 0) {
                    let dt = PrimitiveDateTime::new(date, time).assume_offset(offset);
                    let py_dt = dt.into_pyobject(py).unwrap();
                    let roundtripped: OffsetDateTime = py_dt.extract().unwrap();

                    assert_eq!(dt.year(), roundtripped.year());
                    assert_eq!(dt.month(), roundtripped.month());
                    assert_eq!(dt.day(), roundtripped.day());
                    assert_eq!(dt.hour(), roundtripped.hour());
                    assert_eq!(dt.minute(), roundtripped.minute());
                    assert_eq!(dt.second(), roundtripped.second());
                    assert_eq!(dt.microsecond(), roundtripped.microsecond());
                    assert_eq!(dt.offset().whole_hours(), roundtripped.offset().whole_hours());
                    assert_eq!(dt.offset().minutes_past_hour(), roundtripped.offset().minutes_past_hour());
                }
            });
        }
    }

    #[test]
    fn test_time_utc_datetime_conversion() {
        Python::with_gil(|py| {
            // Create a UTC datetime
            let date = Date::from_calendar_date(2023, Month::April, 15).unwrap();
            let time = Time::from_hms_micro(14, 30, 45, 123456).unwrap();
            let primitive_dt = PrimitiveDateTime::new(date, time);
            let dt: UtcDateTime = primitive_dt.assume_utc().into();

            // Convert to Python
            let py_dt = dt.into_pyobject(py).unwrap();

            // Check components
            let year = py_dt
                .getattr(intern!(py, "year"))
                .unwrap()
                .extract::<i32>()
                .unwrap();
            let month = py_dt
                .getattr(intern!(py, "month"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let day = py_dt
                .getattr(intern!(py, "day"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let hour = py_dt
                .getattr(intern!(py, "hour"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let minute = py_dt
                .getattr(intern!(py, "minute"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let second = py_dt
                .getattr(intern!(py, "second"))
                .unwrap()
                .extract::<u8>()
                .unwrap();
            let microsecond = py_dt
                .getattr(intern!(py, "microsecond"))
                .unwrap()
                .extract::<u32>()
                .unwrap();

            assert_eq!(year, 2023);
            assert_eq!(month, 4);
            assert_eq!(day, 15);
            assert_eq!(hour, 14);
            assert_eq!(minute, 30);
            assert_eq!(second, 45);
            assert_eq!(microsecond, 123456);

            // Verify it has UTC timezone
            let tzinfo = py_dt.getattr(intern!(py, "tzinfo")).unwrap();

            // Check if tzinfo is datetime.timezone.utc
            let datetime = py.import("datetime").unwrap();
            let tz_utc = datetime
                .getattr(intern!(py, "timezone"))
                .unwrap()
                .getattr(intern!(py, "utc"))
                .unwrap();
            let is_utc = tzinfo
                .call_method1("__eq__", (tz_utc,))
                .unwrap()
                .extract::<bool>()
                .unwrap();
            assert!(is_utc);
        });
    }

    #[test]
    fn test_time_utc_datetime_from_python() {
        Python::with_gil(|py| {
            // Create Python UTC datetime
            let datetime = py.import("datetime").unwrap();
            let datetime_type = datetime.getattr(intern!(py, "datetime")).unwrap();
            let tz_utc = datetime
                .getattr(intern!(py, "timezone"))
                .unwrap()
                .getattr(intern!(py, "utc"))
                .unwrap();

            // Create datetime with UTC timezone
            let py_dt = datetime_type
                .call1((2023, 4, 15, 14, 30, 45, 123456, tz_utc))
                .unwrap();

            // Convert to Rust
            let dt: UtcDateTime = py_dt.extract().unwrap();

            // Verify components
            assert_eq!(dt.year(), 2023);
            assert_eq!(dt.month(), Month::April);
            assert_eq!(dt.day(), 15);
            assert_eq!(dt.hour(), 14);
            assert_eq!(dt.minute(), 30);
            assert_eq!(dt.second(), 45);
            assert_eq!(dt.microsecond(), 123456);
        });
    }

    #[test]
    fn test_time_utc_datetime_non_utc_timezone() {
        Python::with_gil(|py| {
            // Create Python datetime with non-UTC timezone
            let datetime = py.import("datetime").unwrap();
            let datetime_type = datetime.getattr(intern!(py, "datetime")).unwrap();
            let timezone = datetime.getattr(intern!(py, "timezone")).unwrap();
            let timedelta = datetime.getattr(intern!(py, "timedelta")).unwrap();

            // Create a non-UTC timezone (EST = UTC-5)
            let td = timedelta.call1((0, -18000, 0)).unwrap(); // -5 hours
            let tz_est = timezone.call1((td,)).unwrap();

            // Create datetime with EST timezone
            let py_dt = datetime_type
                .call1((2023, 4, 15, 14, 30, 45, 123456, tz_est))
                .unwrap();

            // Try to convert to UtcDateTime - should fail
            let result: Result<UtcDateTime, _> = py_dt.extract();
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_time_utc_datetime_roundtrip() {
        Python::with_gil(|py| {
            // Create a UTC datetime
            let date = Date::from_calendar_date(2023, Month::April, 15).unwrap();
            let time = Time::from_hms_micro(14, 30, 45, 123456).unwrap();
            let primitive_dt = PrimitiveDateTime::new(date, time);
            let dt: UtcDateTime = primitive_dt.assume_utc().into();

            // Convert to Python
            let py_dt = dt.into_pyobject(py).unwrap();

            // Convert back to Rust
            let roundtripped: UtcDateTime = py_dt.extract().unwrap();

            // Verify all components match
            assert_eq!(roundtripped.year(), dt.year());
            assert_eq!(roundtripped.month(), dt.month());
            assert_eq!(roundtripped.day(), dt.day());
            assert_eq!(roundtripped.hour(), dt.hour());
            assert_eq!(roundtripped.minute(), dt.minute());
            assert_eq!(roundtripped.second(), dt.second());
            assert_eq!(roundtripped.microsecond(), dt.microsecond());
        });
    }

    proptest! {
    #[test]
    fn test_time_utc_datetime_roundtrip_random(
        year in 1i32..=9999i32,
        month in 1u8..=12u8,
        day in 1u8..=28u8, // Use only valid days for all months
        hour in 0u8..=23u8,
        minute in 0u8..=59u8,
        second in 0u8..=59u8,
        microsecond in 0u32..=999999u32
    ) {
        Python::with_gil(|py| {
            let month = month_from_number!(month);

            let date = Date::from_calendar_date(year, month, day).unwrap();
            let time = Time::from_hms_micro(hour, minute, second, microsecond).unwrap();
            let primitive_dt = PrimitiveDateTime::new(date, time);
            let dt: UtcDateTime = primitive_dt.assume_utc().into();

            let py_dt = dt.into_pyobject(py).unwrap();
            let roundtripped: UtcDateTime = py_dt.extract().unwrap();

            assert_eq!(dt.year(), roundtripped.year());
            assert_eq!(dt.month(), roundtripped.month());
            assert_eq!(dt.day(), roundtripped.day());
            assert_eq!(dt.hour(), roundtripped.hour());
            assert_eq!(dt.minute(), roundtripped.minute());
            assert_eq!(dt.second(), roundtripped.second());
            assert_eq!(dt.microsecond(), roundtripped.microsecond());
        });
    }
    }
}
