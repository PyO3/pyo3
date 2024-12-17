#![cfg(feature = "jiff")]

use crate::exceptions::{PyTypeError, PyValueError};
use crate::pybacked::PyBackedStr;
use crate::sync::GILOnceCell;
#[cfg(not(Py_LIMITED_API))]
use crate::types::datetime::timezone_from_offset;
use crate::types::{PyAnyMethods, PyNone, PyType};
#[cfg(not(Py_LIMITED_API))]
use crate::types::{
    PyDate, PyDateAccess, PyDateTime, PyTime, PyTimeAccess, PyTzInfo, PyTzInfoAccess,
};
use crate::{intern, Bound, FromPyObject, IntoPyObject, Py, PyAny, PyErr, PyResult, Python};
use jiff::civil::{Date, DateTime, Time};
use jiff::tz::{AmbiguousOffset, Offset, TimeZone};
use jiff::{Timestamp, Zoned};
use std::time::Duration;

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
            .map(|tz| {
                if tz.iana_name().is_some() {
                    tz.into_pyobject(py)
                } else {
                    // TODO after https://github.com/BurntSushi/jiff/pull/170 we can remove this special case
                    let (offset, _, _) = tz.to_offset(tz.to_timestamp(*datetime)?);
                    offset.into_pyobject(py)
                }
            })
            .transpose()?
            .as_ref(),
        fold,
    )
}

#[cfg(Py_LIMITED_API)]
fn datetime_to_pydatetime<'py>(
    _py: Python<'py>,
    _datetime: &DateTime,
    _fold: bool,
    _timezone: Option<&TimeZone>,
) -> PyResult<Bound<'py, PyAny>> {
    todo!()
}

#[cfg(not(Py_LIMITED_API))]
fn pytime_to_time(time: &dyn PyTimeAccess) -> PyResult<Time> {
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
        time.getattr(intern!(py, "microsecond"))?.extract()? * 1000,
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
            todo!()
        }
    }
}

impl<'py> FromPyObject<'py> for Date {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        {
            let date = ob.downcast::<PyDate>()?;
            Date::new(
                date.get_year().try_into()?,
                date.get_month().try_into()?,
                date.get_day().try_into()?,
            )
            .map_err(Into::into)
        }

        #[cfg(Py_LIMITED_API)]
        Date::new(
            ob.getattr(intern!(ob.py(), "year"))?.extract()?,
            ob.getattr(intern!(ob.py(), "month"))?.extract()?,
            ob.getattr(intern!(ob.py(), "day"))?.extract()?,
        )
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
            todo!()
        }
    }
}

impl<'py> FromPyObject<'py> for Time {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        let ob = ob.downcast::<PyTime>()?;

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
        let ambiguous_offset = self
            .time_zone()
            .to_ambiguous_zoned(self.datetime())
            .offset();

        let fold = match ambiguous_offset {
            AmbiguousOffset::Unambiguous { .. } => false,
            AmbiguousOffset::Fold { after, .. } => after == self.offset(),
            AmbiguousOffset::Gap { .. } => unreachable!(),
        };
        datetime_to_pydatetime(py, &self.datetime(), fold, Some(self.time_zone()))
    }
}

impl<'py> FromPyObject<'py> for Zoned {
    fn extract_bound(dt: &Bound<'py, PyAny>) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        let dt = dt.downcast::<PyDateTime>()?;

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
        if let Some(iana_name) = self.iana_name() {
            static ZONE_INFO: GILOnceCell<Py<PyType>> = GILOnceCell::new();
            let tz = ZONE_INFO
                .import(py, "zoneinfo", "ZoneInfo")
                .and_then(|obj| obj.call1((iana_name,)))?;

            #[cfg(not(Py_LIMITED_API))]
            let tz = tz.downcast_into()?;

            Ok(tz)
        } else {
            // TODO add support for fixed offsets after https://github.com/BurntSushi/jiff/pull/170 is merged

            Err(PyValueError::new_err(
                "Cannot convert a TimeZone without an IANA name to a python ZoneInfo",
            ))
        }
    }
}

impl<'py> FromPyObject<'py> for TimeZone {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
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
        let duration: Duration = self.duration_since(Offset::UTC).try_into()?;
        let delta = duration.into_pyobject(py)?;

        #[cfg(not(Py_LIMITED_API))]
        {
            timezone_from_offset(&delta)
        }

        #[cfg(Py_LIMITED_API)]
        {
            todo!()
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

        let py_timedelta = ob.call_method1(intern!(py, "utcoffset"), (PyNone::get(py),))?;
        if py_timedelta.is_none() {
            return Err(PyTypeError::new_err(format!(
                "{:?} is not a fixed offset timezone",
                ob
            )));
        }

        let total_seconds: Duration = py_timedelta.extract()?;
        // This cast is safe since the timedelta is limited to -24 hours and 24 hours.
        let total_seconds = total_seconds.as_secs() as i32;
        Ok(Offset::from_seconds(total_seconds)?)
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
            let result: PyResult<FixedOffset> = locals.get_item("zi").unwrap().unwrap().extract();
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
        // Test that if a user tries to convert a python's timezone aware datetime into a naive
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
            let datetime = NaiveDate::from_ymd_opt(2024, 12, 11)
                .unwrap()
                .and_hms_opt(23, 3, 13)
                .unwrap()
                .and_local_timezone(chrono_tz::Tz::Europe__London)
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
}
