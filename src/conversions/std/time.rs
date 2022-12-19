#![cfg(not(Py_LIMITED_API))]

use crate::types::{PyDelta, PyDeltaAccess};
use crate::{FromPyObject, IntoPy, PyAny, PyObject, PyResult, Python, ToPyObject};
use std::convert::TryInto;
use std::time::Duration;

const DAY_SECONDS: u64 = 60 * 60 * 24;

impl ToPyObject for Duration {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let total_secs = self.as_secs();
        // Total number of days
        let days = total_secs / DAY_SECONDS;
        // Remainder of seconds
        // .try_into() converts i64 to i32, but this should never overflow
        // since it's at most the number of seconds per day
        let secs = (total_secs - days * DAY_SECONDS).try_into().unwrap();
        // Fractional part of the duration
        // Same for the conversion from i64 to i32
        let micros = self.subsec_micros().try_into().unwrap();

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
        let delta: &PyDelta = ob.downcast()?;
        // Python size are much lower than rust size so we do not need bound checks.
        // 0 <= microseconds < 1000000
        // 0 <= seconds < 3600*24
        // -999999999 <= days <= 999999999
        let days: u64 = delta.get_days().try_into().unwrap();
        let seconds: u64 = delta.get_seconds().try_into().unwrap();
        let microseconds: u32 = delta.get_microseconds().try_into().unwrap();
        Ok(Duration::new(
            days * DAY_SECONDS + seconds,
            microseconds * 1000,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    #[test]
    fn test_std_duration_topyobject() {
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

        check("delta zero", Duration::ZERO, 0, 0, 0);

        let delta = Duration::from_secs(DAY_SECONDS);
        check("delta 1 day", delta, 1, 0, 0);

        let delta = Duration::new(7 * DAY_SECONDS + 1, 999999000);
        check("delta 7 day and a seconds", delta, 7, 1, 999999);

        // Check the maximum value allowed by PyDelta, which is different
        // from the maximum value allowed in Duration. This should pass.
        let delta = Duration::new(86399999999999, 999999000);
        check("delta max value", delta, 999999999, 86399, 999999);

        // Check that trying to convert an out of bound value panics.
        Python::with_gil(|py| {
            assert!(panic::catch_unwind(|| Duration::MAX.to_object(py)).is_err());
        });
    }

    #[test]
    fn test_std_duration_frompyobject() {
        // Utility function used to check different durations.
        // The `name` parameter is used to identify the check in case of a failure.
        let check = |name: &'static str, delta: Duration, py_days, py_seconds, py_ms| {
            Python::with_gil(|py| {
                let py_delta = PyDelta::new(py, py_days, py_seconds, py_ms, true).unwrap();
                let py_delta: Duration = py_delta.extract().unwrap();
                assert_eq!(py_delta, delta, "{}: {:?} != {:?}", name, py_delta, delta);
            })
        };

        check("delta zero", Duration::ZERO, 0, 0, 0);

        let delta = Duration::from_secs(DAY_SECONDS);
        check("delta 1 day", delta, 1, 0, 0);

        let delta = Duration::new(7 * DAY_SECONDS + 1, 999999000);
        check("delta 7 day and a seconds", delta, 7, 1, 999999);

        // Check the maximum value allowed by PyDelta, which is different
        // from the maximum value allowed in Duration. This should pass.
        check(
            "max pydelta value",
            Duration::new(86399999999999, 999999000),
            999999999,
            86399,
            999999,
        );

        // This check is to assert that we can't construct every possible Duration from a PyDelta
        // since they have different bounds.
        Python::with_gil(|py| {
            let low_days: i32 = -1000000000;
            // This panics on PyDelta::new
            assert!(panic::catch_unwind(|| {
                let pydelta = PyDelta::new(py, low_days, 0, 0, true).unwrap();
                if let Ok(_duration) = pydelta.extract::<Duration>() {
                    // So we should never get here
                }
            })
            .is_err());

            let high_days: i32 = 1000000000;
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
}
