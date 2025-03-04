use crate::sync::GILOnceCell;
use crate::types::PyAnyMethods;
use crate::{Bound, DowncastError, PyAny, PyErr, PyObject, PyResult, Python};

pub(crate) struct DatetimeTypes {
    pub(crate) date: PyObject,
    pub(crate) datetime: PyObject,
    pub(crate) time: PyObject,
    pub(crate) timedelta: PyObject,
    pub(crate) timezone: PyObject,
    pub(crate) timezone_utc: PyObject,
    pub(crate) tzinfo: PyObject,
}

impl DatetimeTypes {
    pub(crate) fn get(py: Python<'_>) -> &Self {
        Self::try_get(py).expect("failed to load datetime module")
    }

    pub(crate) fn try_get(py: Python<'_>) -> PyResult<&Self> {
        static TYPES: GILOnceCell<DatetimeTypes> = GILOnceCell::new();
        TYPES.get_or_try_init(py, || {
            let datetime = py.import("datetime")?;
            let timezone = datetime.getattr("timezone")?;
            Ok::<_, PyErr>(Self {
                date: datetime.getattr("date")?.into(),
                datetime: datetime.getattr("datetime")?.into(),
                time: datetime.getattr("time")?.into(),
                timedelta: datetime.getattr("timedelta")?.into(),
                timezone_utc: timezone.getattr("utc")?.into(),
                timezone: timezone.into(),
                tzinfo: datetime.getattr("tzinfo")?.into(),
            })
        })
    }
}

pub(crate) fn timezone_utc(py: Python<'_>) -> Bound<'_, PyAny> {
    DatetimeTypes::get(py).timezone_utc.bind(py).clone()
}

pub(crate) fn check_type(
    value: &Bound<'_, PyAny>,
    t: &PyObject,
    type_name: &'static str,
) -> PyResult<()> {
    if !value.is_instance(t.bind(value.py()))? {
        return Err(DowncastError::new(value, type_name).into());
    }
    Ok(())
}
