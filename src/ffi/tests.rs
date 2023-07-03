use crate::ffi::*;
use crate::{types::PyDict, AsPyPointer, IntoPy, Py, PyAny, Python};

use crate::types::PyString;
#[cfg(not(Py_3_12))]
use libc::wchar_t;

#[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
#[test]
fn test_datetime_fromtimestamp() {
    Python::with_gil(|py| {
        let args: Py<PyAny> = (100,).into_py(py);
        let dt: &PyAny = unsafe {
            PyDateTime_IMPORT();
            py.from_owned_ptr(PyDateTime_FromTimestamp(args.as_ptr()))
        };
        let locals = PyDict::new(py);
        locals.set_item("dt", dt).unwrap();
        py.run(
            "import datetime; assert dt == datetime.datetime.fromtimestamp(100)",
            None,
            Some(locals),
        )
        .unwrap();
    })
}

#[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
#[test]
fn test_date_fromtimestamp() {
    Python::with_gil(|py| {
        let args: Py<PyAny> = (100,).into_py(py);
        let dt: &PyAny = unsafe {
            PyDateTime_IMPORT();
            py.from_owned_ptr(PyDate_FromTimestamp(args.as_ptr()))
        };
        let locals = PyDict::new(py);
        locals.set_item("dt", dt).unwrap();
        py.run(
            "import datetime; assert dt == datetime.date.fromtimestamp(100)",
            None,
            Some(locals),
        )
        .unwrap();
    })
}

#[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
#[test]
fn test_utc_timezone() {
    Python::with_gil(|py| {
        let utc_timezone: &PyAny = unsafe {
            PyDateTime_IMPORT();
            py.from_borrowed_ptr(PyDateTime_TimeZone_UTC())
        };
        let locals = PyDict::new(py);
        locals.set_item("utc_timezone", utc_timezone).unwrap();
        py.run(
            "import datetime; assert utc_timezone is datetime.timezone.utc",
            None,
            Some(locals),
        )
        .unwrap();
    })
}

#[test]
#[cfg(feature = "macros")]
#[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
fn test_timezone_from_offset() {
    use crate::types::PyDelta;

    Python::with_gil(|py| {
        let tz: &PyAny = unsafe {
            PyDateTime_IMPORT();
            py.from_borrowed_ptr(PyTimeZone_FromOffset(
                PyDelta::new(py, 0, 100, 0, false).unwrap().as_ptr(),
            ))
        };
        crate::py_run!(
            py,
            tz,
            "import datetime; assert tz == datetime.timezone(datetime.timedelta(seconds=100))"
        );
    })
}

#[test]
#[cfg(feature = "macros")]
#[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
fn test_timezone_from_offset_and_name() {
    use crate::types::PyDelta;

    Python::with_gil(|py| {
        let tz: &PyAny = unsafe {
            PyDateTime_IMPORT();
            py.from_borrowed_ptr(PyTimeZone_FromOffsetAndName(
                PyDelta::new(py, 0, 100, 0, false).unwrap().as_ptr(),
                PyString::new(py, "testtz").as_ptr(),
            ))
        };
        crate::py_run!(
            py,
            tz,
            "import datetime; assert tz == datetime.timezone(datetime.timedelta(seconds=100), 'testtz')"
        );
    })
}

#[test]
fn ascii_object_bitfield() {
    let ob_base: PyObject = unsafe { std::mem::zeroed() };

    let mut o = PyASCIIObject {
        ob_base,
        length: 0,
        #[cfg(not(PyPy))]
        hash: 0,
        state: 0u32,
        #[cfg(not(Py_3_12))]
        wstr: std::ptr::null_mut() as *mut wchar_t,
    };

    unsafe {
        assert_eq!(o.interned(), 0);
        assert_eq!(o.kind(), 0);
        assert_eq!(o.compact(), 0);
        assert_eq!(o.ascii(), 0);
        #[cfg(not(Py_3_12))]
        assert_eq!(o.ready(), 0);

        let interned_count = if cfg!(Py_3_12) { 2 } else { 4 };

        for i in 0..interned_count {
            o.set_interned(i);
            assert_eq!(o.interned(), i);
        }

        for i in 0..8 {
            o.set_kind(i);
            assert_eq!(o.kind(), i);
        }

        o.set_compact(1);
        assert_eq!(o.compact(), 1);

        o.set_ascii(1);
        assert_eq!(o.ascii(), 1);

        #[cfg(not(Py_3_12))]
        o.set_ready(1);
        #[cfg(not(Py_3_12))]
        assert_eq!(o.ready(), 1);
    }
}

#[test]
#[cfg_attr(Py_3_10, allow(deprecated))]
fn ascii() {
    Python::with_gil(|py| {
        // This test relies on implementation details of PyString.
        let s = PyString::new(py, "hello, world");
        let ptr = s.as_ptr();

        unsafe {
            let ascii_ptr = ptr as *mut PyASCIIObject;
            let ascii = ascii_ptr.as_ref().unwrap();

            assert_eq!(ascii.interned(), 0);
            assert_eq!(ascii.kind(), PyUnicode_1BYTE_KIND);
            assert_eq!(ascii.compact(), 1);
            assert_eq!(ascii.ascii(), 1);
            #[cfg(not(Py_3_12))]
            assert_eq!(ascii.ready(), 1);

            assert_eq!(PyUnicode_IS_ASCII(ptr), 1);
            assert_eq!(PyUnicode_IS_COMPACT(ptr), 1);
            assert_eq!(PyUnicode_IS_COMPACT_ASCII(ptr), 1);

            assert!(!PyUnicode_1BYTE_DATA(ptr).is_null());
            // 2 and 4 byte macros return nonsense for this string instance.
            assert_eq!(PyUnicode_KIND(ptr), PyUnicode_1BYTE_KIND);

            assert!(!_PyUnicode_COMPACT_DATA(ptr).is_null());
            // _PyUnicode_NONCOMPACT_DATA isn't valid for compact strings.
            assert!(!PyUnicode_DATA(ptr).is_null());

            assert_eq!(PyUnicode_GET_LENGTH(ptr), s.len().unwrap() as Py_ssize_t);
            assert_eq!(PyUnicode_IS_READY(ptr), 1);

            // This has potential to mutate object. But it should be a no-op since
            // we're already ready.
            assert_eq!(PyUnicode_READY(ptr), 0);
        }
    })
}

#[test]
#[cfg_attr(Py_3_10, allow(deprecated))]
fn ucs4() {
    Python::with_gil(|py| {
        let s = "哈哈🐈";
        let py_string = PyString::new(py, s);
        let ptr = py_string.as_ptr();

        unsafe {
            let ascii_ptr = ptr as *mut PyASCIIObject;
            let ascii = ascii_ptr.as_ref().unwrap();

            assert_eq!(ascii.interned(), 0);
            assert_eq!(ascii.kind(), PyUnicode_4BYTE_KIND);
            assert_eq!(ascii.compact(), 1);
            assert_eq!(ascii.ascii(), 0);
            #[cfg(not(Py_3_12))]
            assert_eq!(ascii.ready(), 1);

            assert_eq!(PyUnicode_IS_ASCII(ptr), 0);
            assert_eq!(PyUnicode_IS_COMPACT(ptr), 1);
            assert_eq!(PyUnicode_IS_COMPACT_ASCII(ptr), 0);

            assert!(!PyUnicode_4BYTE_DATA(ptr).is_null());
            assert_eq!(PyUnicode_KIND(ptr), PyUnicode_4BYTE_KIND);

            assert!(!_PyUnicode_COMPACT_DATA(ptr).is_null());
            // _PyUnicode_NONCOMPACT_DATA isn't valid for compact strings.
            assert!(!PyUnicode_DATA(ptr).is_null());

            assert_eq!(
                PyUnicode_GET_LENGTH(ptr),
                py_string.len().unwrap() as Py_ssize_t
            );
            assert_eq!(PyUnicode_IS_READY(ptr), 1);

            // This has potential to mutate object. But it should be a no-op since
            // we're already ready.
            assert_eq!(PyUnicode_READY(ptr), 0);
        }
    })
}

#[test]
#[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
#[cfg(not(PyPy))]
fn test_get_tzinfo() {
    use crate::types::timezone_utc;

    crate::Python::with_gil(|py| {
        use crate::types::{PyDateTime, PyTime};
        use crate::{AsPyPointer, PyAny};

        let utc = timezone_utc(py);

        let dt = PyDateTime::new(py, 2018, 1, 1, 0, 0, 0, 0, Some(utc)).unwrap();

        assert!(
            unsafe { py.from_borrowed_ptr::<PyAny>(PyDateTime_DATE_GET_TZINFO(dt.as_ptr())) }
                .is(utc)
        );

        let dt = PyDateTime::new(py, 2018, 1, 1, 0, 0, 0, 0, None).unwrap();

        assert!(
            unsafe { py.from_borrowed_ptr::<PyAny>(PyDateTime_DATE_GET_TZINFO(dt.as_ptr())) }
                .is_none()
        );

        let t = PyTime::new(py, 0, 0, 0, 0, Some(utc)).unwrap();

        assert!(
            unsafe { py.from_borrowed_ptr::<PyAny>(PyDateTime_TIME_GET_TZINFO(t.as_ptr())) }
                .is(utc)
        );

        let t = PyTime::new(py, 0, 0, 0, 0, None).unwrap();

        assert!(
            unsafe { py.from_borrowed_ptr::<PyAny>(PyDateTime_TIME_GET_TZINFO(t.as_ptr())) }
                .is_none()
        );
    })
}
