use crate::ffi::{self, *};
use crate::types::any::PyAnyMethods;
use crate::Python;

#[cfg(all(not(Py_LIMITED_API), any(not(any(PyPy, GraalPy)), feature = "macros")))]
use crate::types::PyString;

#[cfg(not(Py_LIMITED_API))]
use crate::{types::PyDict, Bound, PyAny};
#[cfg(not(any(Py_3_12, Py_LIMITED_API, GraalPy)))]
use libc::wchar_t;

#[cfg(not(Py_LIMITED_API))]
#[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
#[test]
fn test_datetime_fromtimestamp() {
    use crate::IntoPyObject;
    Python::attach(|py| {
        let args = (100,).into_pyobject(py).unwrap();
        let dt = unsafe {
            PyDateTime_IMPORT();
            Bound::from_owned_ptr(py, PyDateTime_FromTimestamp(args.as_ptr()))
        };
        let locals = PyDict::new(py);
        locals.set_item("dt", dt).unwrap();
        py.run(
            ffi::c_str!("import datetime; assert dt == datetime.datetime.fromtimestamp(100)"),
            None,
            Some(&locals),
        )
        .unwrap();
    })
}

#[cfg(not(Py_LIMITED_API))]
#[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
#[test]
fn test_date_fromtimestamp() {
    use crate::IntoPyObject;
    Python::attach(|py| {
        let args = (100,).into_pyobject(py).unwrap();
        let dt = unsafe {
            PyDateTime_IMPORT();
            Bound::from_owned_ptr(py, PyDate_FromTimestamp(args.as_ptr()))
        };
        let locals = PyDict::new(py);
        locals.set_item("dt", dt).unwrap();
        py.run(
            ffi::c_str!("import datetime; assert dt == datetime.date.fromtimestamp(100)"),
            None,
            Some(&locals),
        )
        .unwrap();
    })
}

#[cfg(not(Py_LIMITED_API))]
#[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
#[test]
fn test_utc_timezone() {
    Python::attach(|py| {
        let utc_timezone: Bound<'_, PyAny> = unsafe {
            PyDateTime_IMPORT();
            Bound::from_borrowed_ptr(py, PyDateTime_TimeZone_UTC())
        };
        let locals = PyDict::new(py);
        locals.set_item("utc_timezone", utc_timezone).unwrap();
        py.run(
            ffi::c_str!("import datetime; assert utc_timezone is datetime.timezone.utc"),
            None,
            Some(&locals),
        )
        .unwrap();
    })
}

#[test]
#[cfg(not(Py_LIMITED_API))]
#[cfg(feature = "macros")]
#[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
fn test_timezone_from_offset() {
    use crate::{ffi_ptr_ext::FfiPtrExt, types::PyDelta};

    Python::attach(|py| {
        let delta = PyDelta::new(py, 0, 100, 0, false).unwrap();
        let tz = unsafe { PyTimeZone_FromOffset(delta.as_ptr()).assume_owned(py) };
        crate::py_run!(
            py,
            tz,
            "import datetime; assert tz == datetime.timezone(datetime.timedelta(seconds=100))"
        );
    })
}

#[test]
#[cfg(not(Py_LIMITED_API))]
#[cfg(feature = "macros")]
#[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
fn test_timezone_from_offset_and_name() {
    use crate::{ffi_ptr_ext::FfiPtrExt, types::PyDelta};

    Python::attach(|py| {
        let delta = PyDelta::new(py, 0, 100, 0, false).unwrap();
        let tzname = PyString::new(py, "testtz");
        let tz = unsafe {
            PyTimeZone_FromOffsetAndName(delta.as_ptr(), tzname.as_ptr()).assume_owned(py)
        };
        crate::py_run!(
            py,
            tz,
            "import datetime; assert tz == datetime.timezone(datetime.timedelta(seconds=100), 'testtz')"
        );
    })
}

#[test]
#[cfg(not(any(Py_LIMITED_API, GraalPy)))]
fn ascii_object_bitfield() {
    let ob_base: PyObject = unsafe { std::mem::zeroed() };

    #[cfg_attr(Py_3_14, allow(unused_mut, unused_variables))]
    let mut o = PyASCIIObject {
        ob_base,
        length: 0,
        #[cfg(any(Py_3_11, not(PyPy)))]
        hash: 0,
        state: 0u32,
        #[cfg(not(Py_3_12))]
        wstr: std::ptr::null_mut() as *mut wchar_t,
    };

    #[cfg(not(Py_3_14))]
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

        #[cfg(Py_3_12)]
        o.set_statically_allocated(1);
        #[cfg(Py_3_12)]
        assert_eq!(o.statically_allocated(), 1);
    }
}

#[test]
#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
fn ascii() {
    Python::attach(|py| {
        // This test relies on implementation details of PyString.
        let s = PyString::new(py, "hello, world");
        let ptr = s.as_ptr();

        unsafe {
            #[cfg(not(Py_3_14))]
            {
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
            }

            assert!(!PyUnicode_1BYTE_DATA(ptr).is_null());
            // 2 and 4 byte macros return nonsense for this string instance.
            assert_eq!(PyUnicode_KIND(ptr), PyUnicode_1BYTE_KIND);

            #[cfg(not(Py_3_14))]
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
#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
fn ucs4() {
    Python::attach(|py| {
        let s = "哈哈🐈";
        let py_string = PyString::new(py, s);
        let ptr = py_string.as_ptr();

        unsafe {
            #[cfg(not(Py_3_14))]
            {
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
            }
            assert!(!PyUnicode_4BYTE_DATA(ptr).is_null());
            assert_eq!(PyUnicode_KIND(ptr), PyUnicode_4BYTE_KIND);

            #[cfg(not(Py_3_14))]
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
#[cfg(not(Py_LIMITED_API))]
#[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
#[cfg(not(all(PyPy, not(Py_3_10))))]
fn test_get_tzinfo() {
    use crate::types::PyTzInfo;

    crate::Python::attach(|py| {
        use crate::types::{PyDateTime, PyTime};

        let utc: &Bound<'_, _> = &PyTzInfo::utc(py).unwrap();

        let dt = PyDateTime::new(py, 2018, 1, 1, 0, 0, 0, 0, Some(utc)).unwrap();

        assert!(
            unsafe { Bound::from_borrowed_ptr(py, PyDateTime_DATE_GET_TZINFO(dt.as_ptr())) }
                .is(utc)
        );

        let dt = PyDateTime::new(py, 2018, 1, 1, 0, 0, 0, 0, None).unwrap();

        assert!(
            unsafe { Bound::from_borrowed_ptr(py, PyDateTime_DATE_GET_TZINFO(dt.as_ptr())) }
                .is_none()
        );

        let t = PyTime::new(py, 0, 0, 0, 0, Some(utc)).unwrap();

        assert!(
            unsafe { Bound::from_borrowed_ptr(py, PyDateTime_TIME_GET_TZINFO(t.as_ptr())) }.is(utc)
        );

        let t = PyTime::new(py, 0, 0, 0, 0, None).unwrap();

        assert!(
            unsafe { Bound::from_borrowed_ptr(py, PyDateTime_TIME_GET_TZINFO(t.as_ptr())) }
                .is_none()
        );
    })
}

#[test]
fn test_inc_dec_ref() {
    Python::attach(|py| {
        let obj = py.eval(ffi::c_str!("object()"), None, None).unwrap();

        let ref_count = obj.get_refcnt();
        let ptr = obj.as_ptr();

        unsafe { Py_INCREF(ptr) };

        assert_eq!(obj.get_refcnt(), ref_count + 1);

        unsafe { Py_DECREF(ptr) };

        assert_eq!(obj.get_refcnt(), ref_count);
    })
}

#[test]
#[cfg(Py_3_12)]
fn test_inc_dec_ref_immortal() {
    Python::attach(|py| {
        let obj = py.None();

        let ref_count = obj.get_refcnt(py);
        let ptr = obj.as_ptr();

        unsafe { Py_INCREF(ptr) };

        assert_eq!(obj.get_refcnt(py), ref_count);

        unsafe { Py_DECREF(ptr) };

        assert_eq!(obj.get_refcnt(py), ref_count);
    })
}
