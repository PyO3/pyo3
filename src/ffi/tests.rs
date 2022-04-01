use crate::ffi::*;
use crate::{types::PyDict, AsPyPointer, IntoPy, Py, PyAny, Python};

#[cfg(target_endian = "little")]
use crate::types::PyString;
#[cfg(target_endian = "little")]
use libc::wchar_t;

#[test]
fn test_datetime_fromtimestamp() {
    Python::with_gil(|py| {
        let args: Py<PyAny> = (100,).into_py(py);
        unsafe { PyDateTime_IMPORT() };
        let dt: &PyAny = unsafe { py.from_owned_ptr(PyDateTime_FromTimestamp(args.as_ptr())) };
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

#[test]
fn test_date_fromtimestamp() {
    Python::with_gil(|py| {
        let args: Py<PyAny> = (100,).into_py(py);
        unsafe { PyDateTime_IMPORT() };
        let dt: &PyAny = unsafe { py.from_owned_ptr(PyDate_FromTimestamp(args.as_ptr())) };
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

#[test]
#[cfg(not(all(PyPy, not(Py_3_8))))]
fn test_utc_timezone() {
    Python::with_gil(|py| {
        let utc_timezone = unsafe {
            PyDateTime_IMPORT();
            &*(&PyDateTime_TimeZone_UTC() as *const *mut crate::ffi::PyObject
                as *const crate::PyObject)
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

#[cfg(target_endian = "little")]
#[test]
fn ascii_object_bitfield() {
    let ob_base: PyObject = unsafe { std::mem::zeroed() };

    let mut o = PyASCIIObject {
        ob_base,
        length: 0,
        hash: 0,
        state: 0,
        wstr: std::ptr::null_mut() as *mut wchar_t,
    };

    unsafe {
        assert_eq!(o.interned(), 0);
        assert_eq!(o.kind(), 0);
        assert_eq!(o.compact(), 0);
        assert_eq!(o.ascii(), 0);
        assert_eq!(o.ready(), 0);

        for i in 0..4 {
            o.state = i;
            assert_eq!(o.interned(), i);
        }

        for i in 0..8 {
            o.state = i << 2;
            assert_eq!(o.kind(), i);
        }

        o.state = 1 << 5;
        assert_eq!(o.compact(), 1);

        o.state = 1 << 6;
        assert_eq!(o.ascii(), 1);

        o.state = 1 << 7;
        assert_eq!(o.ready(), 1);
    }
}

#[cfg(target_endian = "little")]
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

            assert_eq!(PyUnicode_GET_LENGTH(ptr), s.len().unwrap() as _);
            assert_eq!(PyUnicode_IS_READY(ptr), 1);

            // This has potential to mutate object. But it should be a no-op since
            // we're already ready.
            assert_eq!(PyUnicode_READY(ptr), 0);
        }
    })
}

#[cfg(target_endian = "little")]
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
            assert_eq!(ascii.ready(), 1);

            assert_eq!(PyUnicode_IS_ASCII(ptr), 0);
            assert_eq!(PyUnicode_IS_COMPACT(ptr), 1);
            assert_eq!(PyUnicode_IS_COMPACT_ASCII(ptr), 0);

            assert!(!PyUnicode_4BYTE_DATA(ptr).is_null());
            assert_eq!(PyUnicode_KIND(ptr), PyUnicode_4BYTE_KIND);

            assert!(!_PyUnicode_COMPACT_DATA(ptr).is_null());
            // _PyUnicode_NONCOMPACT_DATA isn't valid for compact strings.
            assert!(!PyUnicode_DATA(ptr).is_null());

            assert_eq!(PyUnicode_GET_LENGTH(ptr), py_string.len().unwrap() as _);
            assert_eq!(PyUnicode_IS_READY(ptr), 1);

            // This has potential to mutate object. But it should be a no-op since
            // we're already ready.
            assert_eq!(PyUnicode_READY(ptr), 0);
        }
    })
}
