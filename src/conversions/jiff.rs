#![cfg(feature = "jiff")]

use crate::exceptions::{PyTypeError, PyValueError};
use crate::types::{PyAnyMethods, PyType};

use crate::pybacked::PyBackedStr;
use crate::sync::GILOnceCell;
#[cfg(not(Py_LIMITED_API))]
use crate::types::{
    PyDate, PyDateAccess, PyDateTime, PyTime, PyTimeAccess, PyTzInfo, PyTzInfoAccess,
};
use crate::{intern, Bound, FromPyObject, IntoPyObject, Py, PyAny, PyErr, PyResult, Python};
use jiff::civil::{Date, DateTime, Time};
use jiff::tz::{AmbiguousOffset, TimeZone};
use jiff::{Timestamp, Zoned};

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
        datetime.microsecond().try_into()?,
        timezone
            .map(|tz| tz.into_pyobject(py))
            .transpose()?
            .as_ref(),
        fold,
    )
}

impl<'py> IntoPyObject<'py> for Timestamp {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Timestamp {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(not(Py_LIMITED_API))]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyDateTime::from_timestamp(py, self.as_nanosecond() as f64 / 1_000_000.0, None)
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
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Date {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDate;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        #[cfg(not(Py_LIMITED_API))]
        PyDate::new(
            py,
            self.year().into(),
            self.month().try_into()?,
            self.day().try_into()?,
        )
    }
}

impl<'py> FromPyObject<'py> for Date {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        let date = ob.downcast::<PyDate>()?;

        Date::new(
            date.get_year().try_into()?,
            date.get_month().try_into()?,
            date.get_day().try_into()?,
        )
        .map_err(Into::into)
    }
}

impl<'py> IntoPyObject<'py> for Time {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Time {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        #[cfg(not(Py_LIMITED_API))]
        PyTime::new(
            py,
            self.hour().try_into()?,
            self.minute().try_into()?,
            self.second().try_into()?,
            self.microsecond().try_into()?,
            None,
        )
    }
}

impl<'py> FromPyObject<'py> for Time {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        let time = ob.downcast::<PyTime>()?;
        Time::new(
            time.get_hour().try_into()?,
            time.get_minute().try_into()?,
            time.get_second().try_into()?,
            time.get_microsecond().try_into()?,
        )
        .map_err(Into::into)
    }
}

impl<'py> IntoPyObject<'py> for DateTime {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &DateTime {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        datetime_to_pydatetime(py, self, false, None)
    }
}

impl<'py> FromPyObject<'py> for DateTime {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        Ok(DateTime::from_parts(ob.extract()?, ob.extract()?))
    }
}

impl<'py> IntoPyObject<'py> for Zoned {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Zoned {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyDateTime;
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
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        let dt = ob.downcast::<PyDateTime>()?;
        let tz = {
            let tzinfo: Option<_> = dt.get_tzinfo();

            tzinfo
                .map(|tz| tz.extract::<TimeZone>())
                .unwrap_or_else(|| {
                    Err(PyTypeError::new_err(
                        "expected a datetime with non-None tzinfo",
                    ))
                })?
        };
        let datetime = ob.extract::<DateTime>()?;
        let zoned = tz.to_ambiguous_zoned(datetime);
        match zoned.offset() {
            AmbiguousOffset::Unambiguous { .. } => Ok(tz.to_zoned(datetime)?),
            AmbiguousOffset::Fold { .. } => {
                let fold = false;
                match fold {
                    true => Ok(zoned.later()?),
                    false => Ok(zoned.earlier()?),
                }
            }
            AmbiguousOffset::Gap { .. } => Err(PyValueError::new_err(format!(
                "The datetime {:?} contains an incompatible timezone",
                dt
            ))),
        }
    }
}

impl<'py> IntoPyObject<'py> for TimeZone {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyTzInfo;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &TimeZone {
    #[cfg(not(Py_LIMITED_API))]
    type Target = PyTzInfo;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        static ZONE_INFO: GILOnceCell<Py<PyType>> = GILOnceCell::new();
        let tz = ZONE_INFO
            .import(py, "zoneinfo", "ZoneInfo")
            .and_then(|obj| obj.call1((self.iana_name(),)))?;

        #[cfg(not(Py_LIMITED_API))]
        let tz = tz.downcast_into()?;

        Ok(tz)
    }
}

impl<'py> FromPyObject<'py> for TimeZone {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        TimeZone::get(
            &ob.getattr(intern!(ob.py(), "key"))?
                .extract::<PyBackedStr>()?,
        )
        .map_err(Into::into)
    }
}

impl From<jiff::Error> for PyErr {
    fn from(e: jiff::Error) -> Self {
        PyValueError::new_err(e.to_string())
    }
}
