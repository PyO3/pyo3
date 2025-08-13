#![cfg(feature = "time")]

//! Conversions to and from [time](https://docs.rs/time/)’s `Date`,
//! `Duration`, `OffsetDateTime`, `PrimitiveDateTime`, `Time`, `UtcDateTime` and `UtcOffset`.
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
//! ```rust
//! use time::{Duration, OffsetDateTime, PrimitiveDateTime, Date, Time, Month};
//! use pyo3::{Python, PyResult, IntoPyObject, types::PyAnyMethods};
//!
//! fn main() -> PyResult<()> {
//!     Python::initialize();
//!     Python::attach(|py| {
//!         // Create a fixed date and time (2022-01-01 12:00:00 UTC)
//!         let date = Date::from_calendar_date(2022, Month::January, 1).unwrap();
//!         let time = Time::from_hms(12, 0, 0).unwrap();
//!         let primitive_dt = PrimitiveDateTime::new(date, time);
//!
//!         // Convert to OffsetDateTime with UTC offset
//!         let datetime = primitive_dt.assume_utc();
//!
//!         // Create a duration of 1 hour
//!         let duration = Duration::hours(1);
//!
//!         // Convert to Python objects
//!         let py_datetime = datetime.into_pyobject(py)?;
//!         let py_timedelta = duration.into_pyobject(py)?;
//!
//!         // Add the duration to the datetime in Python
//!         let py_result = py_datetime.add(py_timedelta)?;
//!
//!         // Convert the result back to Rust
//!         let result: OffsetDateTime = py_result.extract()?;
//!         assert_eq!(result.hour(), 13);
//!
//!         Ok(())
//!     })
//! }
//! ```

use crate::exceptions::{PyTypeError, PyValueError};
#[cfg(Py_LIMITED_API)]
use crate::intern;
#[cfg(not(Py_LIMITED_API))]
use crate::types::datetime::{PyDateAccess, PyDeltaAccess};
use crate::types::{PyAnyMethods, PyDate, PyDateTime, PyDelta, PyNone, PyTime, PyTzInfo};
#[cfg(not(Py_LIMITED_API))]
use crate::types::{PyTimeAccess, PyTzInfoAccess};
use crate::{Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python};
use time::{
    Date, Duration, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcDateTime, UtcOffset,
};

const SECONDS_PER_DAY: i64 = 86_400;

// Macro for reference implementation
macro_rules! impl_into_py_for_ref {
    ($type:ty, $target:ty) => {
        impl<'py> IntoPyObject<'py> for &$type {
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

fn extract_date_time(dt: &Bound<'_, PyAny>) -> PyResult<(Date, Time)> {
    #[cfg(not(Py_LIMITED_API))]
    {
        let dt = dt.cast::<PyDateTime>()?;
        let date = Date::from_calendar_date(
            dt.get_year(),
            month_from_number!(dt.get_month()),
            dt.get_day(),
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range date"))?;

        let time = Time::from_hms_micro(
            dt.get_hour(),
            dt.get_minute(),
            dt.get_second(),
            dt.get_microsecond(),
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range time"))?;
        Ok((date, time))
    }

    #[cfg(Py_LIMITED_API)]
    {
        let date = Date::from_calendar_date(
            dt.getattr(intern!(dt.py(), "year"))?.extract()?,
            month_from_number!(dt.getattr(intern!(dt.py(), "month"))?.extract::<u8>()?),
            dt.getattr(intern!(dt.py(), "day"))?.extract()?,
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range date"))?;

        let time = Time::from_hms_micro(
            dt.getattr(intern!(dt.py(), "hour"))?.extract()?,
            dt.getattr(intern!(dt.py(), "minute"))?.extract()?,
            dt.getattr(intern!(dt.py(), "second"))?.extract()?,
            dt.getattr(intern!(dt.py(), "microsecond"))?.extract()?,
        )
        .map_err(|_| PyValueError::new_err("invalid or out-of-range time"))?;

        Ok((date, time))
    }
}

impl<'py> IntoPyObject<'py> for Duration {
    type Target = PyDelta;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let total_seconds = self.whole_seconds();
        let micro_seconds = self.subsec_microseconds();

        // For negative durations, Python expects days to be negative and
        // seconds/microseconds to be positive or zero
        let (days, seconds) = if total_seconds < 0 && total_seconds % SECONDS_PER_DAY != 0 {
            // For negative values, we need to round down (toward more negative)
            // e.g., -10 seconds should be -1 days + 86390 seconds
            let days = total_seconds.div_euclid(SECONDS_PER_DAY);
            let seconds = total_seconds.rem_euclid(SECONDS_PER_DAY);
            (days, seconds)
        } else {
            // For positive or exact negative days, use normal division
            (
                total_seconds / SECONDS_PER_DAY,
                total_seconds % SECONDS_PER_DAY,
            )
        };
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
}

impl FromPyObject<'_> for Duration {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Duration> {
        #[cfg(not(Py_LIMITED_API))]
        let (days, seconds, microseconds) = {
            let delta = ob.cast::<PyDelta>()?;
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
    type Target = PyDate;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let year = self.year();
        let month = self.month() as u8;
        let day = self.day();

        PyDate::new(py, year, month, day)
    }
}

impl FromPyObject<'_> for Date {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Date> {
        let (year, month, day) = {
            #[cfg(not(Py_LIMITED_API))]
            {
                let date = ob.cast::<PyDate>()?;
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
    type Target = PyTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let hour = self.hour();
        let minute = self.minute();
        let second = self.second();
        let microsecond = self.microsecond();

        PyTime::new(py, hour, minute, second, microsecond, None)
    }
}

impl FromPyObject<'_> for Time {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Time> {
        let (hour, minute, second, microsecond) = {
            #[cfg(not(Py_LIMITED_API))]
            {
                let time = ob.cast::<PyTime>()?;
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
}

impl FromPyObject<'_> for PrimitiveDateTime {
    fn extract_bound(dt: &Bound<'_, PyAny>) -> PyResult<PrimitiveDateTime> {
        let has_tzinfo = {
            #[cfg(not(Py_LIMITED_API))]
            {
                let dt = dt.cast::<PyDateTime>()?;
                dt.get_tzinfo().is_some()
            }
            #[cfg(Py_LIMITED_API)]
            {
                !dt.getattr(intern!(dt.py(), "tzinfo"))?.is_none()
            }
        };

        if has_tzinfo {
            return Err(PyTypeError::new_err("expected a datetime without tzinfo"));
        }

        let (date, time) = extract_date_time(dt)?;

        Ok(PrimitiveDateTime::new(date, time))
    }
}

impl<'py> IntoPyObject<'py> for UtcOffset {
    type Target = PyTzInfo;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        // Get offset in seconds
        let seconds_offset = self.whole_seconds();
        let td = PyDelta::new(py, 0, seconds_offset, 0, true)?;
        PyTzInfo::fixed_offset(py, td)
    }
}

impl FromPyObject<'_> for UtcOffset {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<UtcOffset> {
        #[cfg(not(Py_LIMITED_API))]
        let ob = ob.cast::<PyTzInfo>()?;

        // Get the offset in seconds from the Python tzinfo
        let py_timedelta = ob.call_method1("utcoffset", (PyNone::get(ob.py()),))?;
        if py_timedelta.is_none() {
            return Err(PyTypeError::new_err(format!(
                "{ob:?} is not a fixed offset timezone"
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

        PyDateTime::new(
            py,
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond,
            Some(py_tzinfo.cast()?),
        )
    }
}

impl FromPyObject<'_> for OffsetDateTime {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<OffsetDateTime> {
        let offset: UtcOffset = {
            #[cfg(not(Py_LIMITED_API))]
            {
                let dt = ob.cast::<PyDateTime>()?;
                let tzinfo = dt.get_tzinfo().ok_or_else(|| {
                    PyTypeError::new_err("expected a datetime with non-None tzinfo")
                })?;
                tzinfo.extract()?
            }
            #[cfg(Py_LIMITED_API)]
            {
                let tzinfo = ob.getattr(intern!(ob.py(), "tzinfo"))?;
                if tzinfo.is_none() {
                    return Err(PyTypeError::new_err(
                        "expected a datetime with non-None tzinfo",
                    ));
                }
                tzinfo.extract()?
            }
        };

        let (date, time) = extract_date_time(ob)?;

        let primitive_dt = PrimitiveDateTime::new(date, time);
        Ok(primitive_dt.assume_offset(offset))
    }
}

impl<'py> IntoPyObject<'py> for UtcDateTime {
    type Target = PyDateTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let date = self.date();
        let time = self.time();

        let py_tzinfo = PyTzInfo::utc(py)?;

        let year = date.year();
        let month = date.month() as u8;
        let day = date.day();
        let hour = time.hour();
        let minute = time.minute();
        let second = time.second();
        let microsecond = time.microsecond();

        PyDateTime::new(
            py,
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond,
            Some(&py_tzinfo),
        )
    }
}

impl FromPyObject<'_> for UtcDateTime {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<UtcDateTime> {
        let tzinfo = {
            #[cfg(not(Py_LIMITED_API))]
            {
                let dt = ob.cast::<PyDateTime>()?;
                dt.get_tzinfo().ok_or_else(|| {
                    PyTypeError::new_err("expected a datetime with non-None tzinfo")
                })?
            }

            #[cfg(Py_LIMITED_API)]
            {
                let tzinfo = ob.getattr(intern!(ob.py(), "tzinfo"))?;
                if tzinfo.is_none() {
                    return Err(PyTypeError::new_err(
                        "expected a datetime with non-None tzinfo",
                    ));
                }
                tzinfo
            }
        };

        // Verify that the tzinfo is UTC
        let is_utc = tzinfo.eq(PyTzInfo::utc(ob.py())?)?;

        if !is_utc {
            return Err(PyValueError::new_err(
                "expected a datetime with UTC timezone",
            ));
        }

        let (date, time) = extract_date_time(ob)?;
        let primitive_dt = PrimitiveDateTime::new(date, time);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intern;
    use crate::types::any::PyAnyMethods;
    use crate::types::PyTypeMethods;

    mod utils {
        use super::*;

        pub(crate) fn extract_py_delta_from_duration(
            duration: Duration,
            py: Python<'_>,
        ) -> (i64, i64, i64) {
            let py_delta = duration.into_pyobject(py).unwrap();
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
            (days, seconds, microseconds)
        }

        pub(crate) fn extract_py_date_from_date(date: Date, py: Python<'_>) -> (i32, u8, u8) {
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
            (year, month, day)
        }

        pub(crate) fn create_date_from_py_date(
            py: Python<'_>,
            year: i32,
            month: u8,
            day: u8,
        ) -> PyResult<Date> {
            let datetime = py.import("datetime").unwrap();
            let date_type = datetime.getattr(intern!(py, "date")).unwrap();
            let py_date = date_type.call1((year, month, day));
            match py_date {
                Ok(py_date) => py_date.extract(),
                Err(err) => Err(err),
            }
        }

        pub(crate) fn create_time_from_py_time(
            py: Python<'_>,
            hour: u8,
            minute: u8,
            second: u8,
            microseocnd: u32,
        ) -> PyResult<Time> {
            let datetime = py.import("datetime").unwrap();
            let time_type = datetime.getattr(intern!(py, "time")).unwrap();
            let py_time = time_type.call1((hour, minute, second, microseocnd));
            match py_time {
                Ok(py_time) => py_time.extract(),
                Err(err) => Err(err),
            }
        }

        pub(crate) fn extract_py_time_from_time(time: Time, py: Python<'_>) -> (u8, u8, u8, u32) {
            let py_time = time.into_pyobject(py).unwrap();
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
            (hour, minute, second, microsecond)
        }

        pub(crate) fn extract_date_time_from_primitive_date_time(
            dt: PrimitiveDateTime,
            py: Python<'_>,
        ) -> (u32, u8, u8, u8, u8, u8, u32) {
            let py_dt = dt.into_pyobject(py).unwrap();
            let year = py_dt
                .getattr(intern!(py, "year"))
                .unwrap()
                .extract::<u32>()
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
            (year, month, day, hour, minute, second, microsecond)
        }

        #[allow(clippy::too_many_arguments)]
        pub(crate) fn create_primitive_date_time_from_py(
            py: Python<'_>,
            year: u32,
            month: u8,
            day: u8,
            hour: u8,
            minute: u8,
            second: u8,
            microsecond: u32,
        ) -> PyResult<PrimitiveDateTime> {
            let datetime = py.import("datetime").unwrap();
            let datetime_type = datetime.getattr(intern!(py, "datetime")).unwrap();
            let py_dt = datetime_type.call1((year, month, day, hour, minute, second, microsecond));
            match py_dt {
                Ok(py_dt) => py_dt.extract(),
                Err(err) => Err(err),
            }
        }

        pub(crate) fn extract_total_seconds_from_utcoffset(
            offset: UtcOffset,
            py: Python<'_>,
        ) -> f64 {
            let py_tz = offset.into_pyobject(py).unwrap();
            let utc_offset = py_tz.call_method1("utcoffset", (py.None(),)).unwrap();
            let total_seconds = utc_offset
                .getattr(intern!(py, "total_seconds"))
                .unwrap()
                .call0()
                .unwrap()
                .extract::<f64>()
                .unwrap();
            total_seconds
        }

        pub(crate) fn extract_from_utc_date_time(
            dt: UtcDateTime,
            py: Python<'_>,
        ) -> (u32, u8, u8, u8, u8, u8, u32) {
            let py_dt = dt.into_pyobject(py).unwrap();
            let year = py_dt
                .getattr(intern!(py, "year"))
                .unwrap()
                .extract::<u32>()
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
            (year, month, day, hour, minute, second, microsecond)
        }
    }
    #[test]
    fn test_time_duration_conversion() {
        Python::attach(|py| {
            // Regular duration
            let duration = Duration::new(1, 500_000_000); // 1.5 seconds
            let (_, seconds, microseconds) = utils::extract_py_delta_from_duration(duration, py);
            assert_eq!(seconds, 1);
            assert_eq!(microseconds, 500_000);

            // Check negative durations
            let neg_duration = Duration::new(-10, 0); // -10 seconds
            let (days, seconds, _) = utils::extract_py_delta_from_duration(neg_duration, py);
            assert_eq!(days, -1);
            assert_eq!(seconds, 86390); // 86400 - 10 seconds

            // Test case for exact negative days (should use normal division path)
            let exact_day = Duration::seconds(-86_400); // Exactly -1 day
            let (days, seconds, microseconds) =
                utils::extract_py_delta_from_duration(exact_day, py);
            assert_eq!(days, -1);
            assert_eq!(seconds, 0);
            assert_eq!(microseconds, 0);
        });
    }

    #[test]
    fn test_time_duration_conversion_large_values() {
        Python::attach(|py| {
            // Large duration (close to max)
            let large_duration = Duration::seconds(86_399_999_000_000); // Almost max
            let (days, _, _) = utils::extract_py_delta_from_duration(large_duration, py);
            assert!(days > 999_000_000);

            // Test over limit (should yield Overflow error in python)
            let too_large = Duration::seconds(86_400_000_000_000); // Over max
            let result = too_large.into_pyobject(py);
            assert!(result.is_err());
            let err_type = result.unwrap_err().get_type(py).name().unwrap();
            assert_eq!(err_type, "OverflowError");
        });
    }

    #[test]
    fn test_time_duration_nanosecond_resolution() {
        Python::attach(|py| {
            // Test nanosecond conversion to microseconds
            let duration = Duration::new(0, 1_234_567);
            let (_, _, microseconds) = utils::extract_py_delta_from_duration(duration, py);
            // Python timedelta only has microsecond resolution, so we should get 1234 microseconds
            assert_eq!(microseconds, 1234);
        });
    }

    #[test]
    fn test_time_duration_from_python() {
        Python::attach(|py| {
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
        });
    }

    #[test]
    fn test_time_date_conversion() {
        Python::attach(|py| {
            // Regular date
            let date = Date::from_calendar_date(2023, Month::April, 15).unwrap();
            let (year, month, day) = utils::extract_py_date_from_date(date, py);
            assert_eq!(year, 2023);
            assert_eq!(month, 4);
            assert_eq!(day, 15);

            // Test edge cases
            let min_date = Date::from_calendar_date(1, Month::January, 1).unwrap();
            let (min_year, min_month, min_day) = utils::extract_py_date_from_date(min_date, py);
            assert_eq!(min_year, 1);
            assert_eq!(min_month, 1);
            assert_eq!(min_day, 1);

            let max_date = Date::from_calendar_date(9999, Month::December, 31).unwrap();
            let (max_year, max_month, max_day) = utils::extract_py_date_from_date(max_date, py);
            assert_eq!(max_year, 9999);
            assert_eq!(max_month, 12);
            assert_eq!(max_day, 31);
        });
    }

    #[test]
    fn test_time_date_from_python() {
        Python::attach(|py| {
            let date1 = utils::create_date_from_py_date(py, 2023, 4, 15).unwrap();
            assert_eq!(date1.year(), 2023);
            assert_eq!(date1.month(), Month::April);
            assert_eq!(date1.day(), 15);

            // Test min date
            let date2 = utils::create_date_from_py_date(py, 1, 1, 1).unwrap();
            assert_eq!(date2.year(), 1);
            assert_eq!(date2.month(), Month::January);
            assert_eq!(date2.day(), 1);

            // Test max date
            let date3 = utils::create_date_from_py_date(py, 9999, 12, 31).unwrap();
            assert_eq!(date3.year(), 9999);
            assert_eq!(date3.month(), Month::December);
            assert_eq!(date3.day(), 31);

            // Test leap year date
            let date4 = utils::create_date_from_py_date(py, 2024, 2, 29).unwrap();
            assert_eq!(date4.year(), 2024);
            assert_eq!(date4.month(), Month::February);
            assert_eq!(date4.day(), 29);
        });
    }

    #[test]
    fn test_time_date_invalid_values() {
        Python::attach(|py| {
            let invalid_date = utils::create_date_from_py_date(py, 2023, 2, 30);
            assert!(invalid_date.is_err());

            // Test extraction of invalid month
            let another_invalid_date = utils::create_date_from_py_date(py, 2023, 13, 1);
            assert!(another_invalid_date.is_err());
        });
    }

    #[test]
    fn test_time_time_conversion() {
        Python::attach(|py| {
            // Regular time
            let time = Time::from_hms_micro(14, 30, 45, 123456).unwrap();
            let (hour, minute, second, microsecond) = utils::extract_py_time_from_time(time, py);
            assert_eq!(hour, 14);
            assert_eq!(minute, 30);
            assert_eq!(second, 45);
            assert_eq!(microsecond, 123456);

            // Test edge cases
            let min_time = Time::from_hms_micro(0, 0, 0, 0).unwrap();
            let (min_hour, min_minute, min_second, min_microsecond) =
                utils::extract_py_time_from_time(min_time, py);
            assert_eq!(min_hour, 0);
            assert_eq!(min_minute, 0);
            assert_eq!(min_second, 0);
            assert_eq!(min_microsecond, 0);

            let max_time = Time::from_hms_micro(23, 59, 59, 999999).unwrap();
            let (max_hour, max_minute, max_second, max_microsecond) =
                utils::extract_py_time_from_time(max_time, py);
            assert_eq!(max_hour, 23);
            assert_eq!(max_minute, 59);
            assert_eq!(max_second, 59);
            assert_eq!(max_microsecond, 999999);
        });
    }

    #[test]
    fn test_time_time_from_python() {
        Python::attach(|py| {
            let time1 = utils::create_time_from_py_time(py, 14, 30, 45, 123456).unwrap();
            assert_eq!(time1.hour(), 14);
            assert_eq!(time1.minute(), 30);
            assert_eq!(time1.second(), 45);
            assert_eq!(time1.microsecond(), 123456);

            // Test min time
            let time2 = utils::create_time_from_py_time(py, 0, 0, 0, 0).unwrap();
            assert_eq!(time2.hour(), 0);
            assert_eq!(time2.minute(), 0);
            assert_eq!(time2.second(), 0);
            assert_eq!(time2.microsecond(), 0);

            // Test max time
            let time3 = utils::create_time_from_py_time(py, 23, 59, 59, 999999).unwrap();
            assert_eq!(time3.hour(), 23);
            assert_eq!(time3.minute(), 59);
            assert_eq!(time3.second(), 59);
            assert_eq!(time3.microsecond(), 999999);
        });
    }

    #[test]
    fn test_time_time_invalid_values() {
        Python::attach(|py| {
            let result = utils::create_time_from_py_time(py, 24, 0, 0, 0);
            assert!(result.is_err());
            let result = utils::create_time_from_py_time(py, 12, 60, 0, 0);
            assert!(result.is_err());
            let result = utils::create_time_from_py_time(py, 12, 30, 60, 0);
            assert!(result.is_err());
            let result = utils::create_time_from_py_time(py, 12, 30, 30, 1000000);
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_time_time_with_timezone() {
        Python::attach(|py| {
            // Create Python time with timezone (just to ensure we can handle it properly)
            let datetime = py.import("datetime").unwrap();
            let time_type = datetime.getattr(intern!(py, "time")).unwrap();
            let tz_utc = PyTzInfo::utc(py).unwrap();

            // Create time with timezone
            let py_time_with_tz = time_type.call1((12, 30, 45, 0, tz_utc)).unwrap();
            let time: Time = py_time_with_tz.extract().unwrap();

            assert_eq!(time.hour(), 12);
            assert_eq!(time.minute(), 30);
            assert_eq!(time.second(), 45);
        });
    }

    #[test]
    fn test_time_primitive_datetime_conversion() {
        Python::attach(|py| {
            // Regular datetime
            let date = Date::from_calendar_date(2023, Month::April, 15).unwrap();
            let time = Time::from_hms_micro(14, 30, 45, 123456).unwrap();
            let dt = PrimitiveDateTime::new(date, time);
            let (year, month, day, hour, minute, second, microsecond) =
                utils::extract_date_time_from_primitive_date_time(dt, py);

            assert_eq!(year, 2023);
            assert_eq!(month, 4);
            assert_eq!(day, 15);
            assert_eq!(hour, 14);
            assert_eq!(minute, 30);
            assert_eq!(second, 45);
            assert_eq!(microsecond, 123456);

            // Test min datetime
            let min_date = Date::from_calendar_date(1, Month::January, 1).unwrap();
            let min_time = Time::from_hms_micro(0, 0, 0, 0).unwrap();
            let min_dt = PrimitiveDateTime::new(min_date, min_time);
            let (year, month, day, hour, minute, second, microsecond) =
                utils::extract_date_time_from_primitive_date_time(min_dt, py);
            assert_eq!(year, 1);
            assert_eq!(month, 1);
            assert_eq!(day, 1);
            assert_eq!(hour, 0);
            assert_eq!(minute, 0);
            assert_eq!(second, 0);
            assert_eq!(microsecond, 0);
        });
    }

    #[test]
    fn test_time_primitive_datetime_from_python() {
        Python::attach(|py| {
            let dt1 =
                utils::create_primitive_date_time_from_py(py, 2023, 4, 15, 14, 30, 45, 123456)
                    .unwrap();
            assert_eq!(dt1.year(), 2023);
            assert_eq!(dt1.month(), Month::April);
            assert_eq!(dt1.day(), 15);
            assert_eq!(dt1.hour(), 14);
            assert_eq!(dt1.minute(), 30);
            assert_eq!(dt1.second(), 45);
            assert_eq!(dt1.microsecond(), 123456);

            let dt2 = utils::create_primitive_date_time_from_py(py, 1, 1, 1, 0, 0, 0, 0).unwrap();
            assert_eq!(dt2.year(), 1);
            assert_eq!(dt2.month(), Month::January);
            assert_eq!(dt2.day(), 1);
            assert_eq!(dt2.hour(), 0);
            assert_eq!(dt2.minute(), 0);
        });
    }

    #[test]
    fn test_time_utc_offset_conversion() {
        Python::attach(|py| {
            // Test positive offset
            let offset = UtcOffset::from_hms(5, 30, 0).unwrap();
            let total_seconds = utils::extract_total_seconds_from_utcoffset(offset, py);
            assert_eq!(total_seconds, 5.0 * 3600.0 + 30.0 * 60.0);

            // Test negative offset
            let neg_offset = UtcOffset::from_hms(-8, -15, 0).unwrap();
            let neg_total_seconds = utils::extract_total_seconds_from_utcoffset(neg_offset, py);
            assert_eq!(neg_total_seconds, -8.0 * 3600.0 - 15.0 * 60.0);
        });
    }

    #[test]
    fn test_time_utc_offset_from_python() {
        Python::attach(|py| {
            // Create timezone objects
            let datetime = py.import("datetime").unwrap();
            let timezone = datetime.getattr(intern!(py, "timezone")).unwrap();
            let timedelta = datetime.getattr(intern!(py, "timedelta")).unwrap();

            // Test UTC
            let tz_utc = PyTzInfo::utc(py).unwrap();
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
    fn test_time_offset_datetime_conversion() {
        Python::attach(|py| {
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
        Python::attach(|py| {
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
    fn test_time_utc_datetime_conversion() {
        Python::attach(|py| {
            let date = Date::from_calendar_date(2023, Month::April, 15).unwrap();
            let time = Time::from_hms_micro(14, 30, 45, 123456).unwrap();
            let primitive_dt = PrimitiveDateTime::new(date, time);
            let dt: UtcDateTime = primitive_dt.assume_utc().into();
            let (year, month, day, hour, minute, second, microsecond) =
                utils::extract_from_utc_date_time(dt, py);

            assert_eq!(year, 2023);
            assert_eq!(month, 4);
            assert_eq!(day, 15);
            assert_eq!(hour, 14);
            assert_eq!(minute, 30);
            assert_eq!(second, 45);
            assert_eq!(microsecond, 123456);
        });
    }

    #[test]
    fn test_time_utc_datetime_from_python() {
        Python::attach(|py| {
            // Create Python UTC datetime
            let datetime = py.import("datetime").unwrap();
            let datetime_type = datetime.getattr(intern!(py, "datetime")).unwrap();
            let tz_utc = PyTzInfo::utc(py).unwrap();

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
        Python::attach(|py| {
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

    #[cfg(not(any(target_arch = "wasm32", Py_GIL_DISABLED)))]
    mod proptests {
        use super::*;
        use proptest::proptest;

        proptest! {
            #[test]
            fn test_time_duration_roundtrip(days in -9999i64..=9999i64, seconds in -86399i64..=86399i64, microseconds in -999999i64..=999999i64) {
                // Generate a valid duration that should roundtrip successfully
                Python::attach(|py| {
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

            #[test]
            fn test_all_valid_dates(
                year in 1i32..=9999,
                month_num in 1u8..=12,
            ) {
                Python::attach(|py| {
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

            #[test]
            fn test_time_time_roundtrip_random(
                hour in 0u8..=23u8,
                minute in 0u8..=59u8,
                second in 0u8..=59u8,
                microsecond in 0u32..=999999u32
            ) {
                Python::attach(|py| {
                    let time = Time::from_hms_micro(hour, minute, second, microsecond).unwrap();
                    let py_time = time.into_pyobject(py).unwrap();
                    let roundtripped: Time = py_time.extract().unwrap();
                    assert_eq!(time, roundtripped);
                });
            }

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
                Python::attach(|py| {
                    let month = match month {
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
                        _ => unreachable!(),
                    };

                    let date = Date::from_calendar_date(year, month, day).unwrap();
                    let time = Time::from_hms_micro(hour, minute, second, microsecond).unwrap();
                    let dt = PrimitiveDateTime::new(date, time);

                    let py_dt = dt.into_pyobject(py).unwrap();
                    let roundtripped: PrimitiveDateTime = py_dt.extract().unwrap();
                    assert_eq!(dt, roundtripped);
                });
            }

            #[test]
            fn test_time_utc_offset_roundtrip_random(
                hours in -23i8..=23i8,
                minutes in -59i8..=59i8
            ) {
                // Skip invalid combinations where hour and minute signs don't match
                if (hours < 0 && minutes > 0) || (hours > 0 && minutes < 0) {
                    return Ok(());
                }

                Python::attach(|py| {
                    if let Ok(offset) = UtcOffset::from_hms(hours, minutes, 0) {
                        let py_tz = offset.into_pyobject(py).unwrap();
                        let roundtripped: UtcOffset = py_tz.extract().unwrap();
                        assert_eq!(roundtripped.whole_hours(), hours);
                        assert_eq!(roundtripped.minutes_past_hour(), minutes);
                    }
                });
            }

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
                Python::attach(|py| {
                    let month = match month {
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
                        _ => unreachable!(),
                    };

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
                Python::attach(|py| {
                    let month = match month {
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
                        _ => unreachable!(),
                    };

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
                })
            }
        }
    }
}
