#![cfg(feature = "macros")]

use std::collections::HashMap;

#[cfg(not(Py_LIMITED_API))]
use pyo3::buffer::PyBuffer;
use pyo3::prelude::*;
#[cfg(not(Py_LIMITED_API))]
use pyo3::types::PyDateTime;
#[cfg(not(any(Py_LIMITED_API, PyPy)))]
use pyo3::types::PyFunction;
use pyo3::types::{self, PyCFunction};

#[path = "../src/tests/common.rs"]
mod common;

#[pyfunction(name = "struct")]
fn struct_function() {}

#[test]
fn test_rust_keyword_name() {
    Python::with_gil(|py| {
        let f = wrap_pyfunction_bound!(struct_function)(py).unwrap();

        py_assert!(py, f, "f.__name__ == 'struct'");
    });
}

#[pyfunction(signature = (arg = true))]
fn optional_bool(arg: Option<bool>) -> String {
    format!("{:?}", arg)
}

#[test]
fn test_optional_bool() {
    // Regression test for issue #932
    Python::with_gil(|py| {
        let f = wrap_pyfunction_bound!(optional_bool)(py).unwrap();

        py_assert!(py, f, "f() == 'Some(true)'");
        py_assert!(py, f, "f(True) == 'Some(true)'");
        py_assert!(py, f, "f(False) == 'Some(false)'");
        py_assert!(py, f, "f(None) == 'None'");
    });
}

#[cfg(not(Py_LIMITED_API))]
#[pyfunction]
fn buffer_inplace_add(py: Python<'_>, x: PyBuffer<i32>, y: PyBuffer<i32>) {
    let x = x.as_mut_slice(py).unwrap();
    let y = y.as_slice(py).unwrap();
    for (xi, yi) in x.iter().zip(y) {
        let xi_plus_yi = xi.get() + yi.get();
        xi.set(xi_plus_yi);
    }
}

#[cfg(not(Py_LIMITED_API))]
#[test]
fn test_buffer_add() {
    Python::with_gil(|py| {
        let f = wrap_pyfunction_bound!(buffer_inplace_add)(py).unwrap();

        py_expect_exception!(
            py,
            f,
            r#"
import array
a = array.array("i", [0, 1, 2, 3])
b = array.array("I", [0, 1, 2, 3])
f(a, b)
"#,
            PyBufferError
        );

        pyo3::py_run!(
            py,
            f,
            r#"
import array
a = array.array("i", [0, 1, 2, 3])
b = array.array("i", [2, 3, 4, 5])
f(a, b)
assert a, array.array("i", [2, 4, 6, 8])
"#
        );
    });
}

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
#[pyfunction]
fn function_with_pyfunction_arg<'py>(fun: &Bound<'py, PyFunction>) -> PyResult<Bound<'py, PyAny>> {
    fun.call((), None)
}

#[pyfunction]
fn function_with_pycfunction_arg<'py>(
    fun: &Bound<'py, PyCFunction>,
) -> PyResult<Bound<'py, PyAny>> {
    fun.call((), None)
}

#[test]
fn test_functions_with_function_args() {
    Python::with_gil(|py| {
        let py_cfunc_arg = wrap_pyfunction_bound!(function_with_pycfunction_arg)(py).unwrap();
        let bool_to_string = wrap_pyfunction_bound!(optional_bool)(py).unwrap();

        pyo3::py_run!(
            py,
            py_cfunc_arg
            bool_to_string,
            r#"
        assert py_cfunc_arg(bool_to_string) == "Some(true)"
        "#
        );

        #[cfg(not(any(Py_LIMITED_API, PyPy)))]
        {
            let py_func_arg = wrap_pyfunction_bound!(function_with_pyfunction_arg)(py).unwrap();

            pyo3::py_run!(
                py,
                py_func_arg,
                r#"
            def foo(): return "bar"
            assert py_func_arg(foo) == "bar"
            "#
            );
        }
    });
}

#[cfg(not(Py_LIMITED_API))]
fn datetime_to_timestamp(dt: &Bound<'_, PyAny>) -> PyResult<i64> {
    let dt = dt.downcast::<PyDateTime>()?;
    let ts: f64 = dt.call_method0("timestamp")?.extract()?;

    Ok(ts as i64)
}

#[cfg(not(Py_LIMITED_API))]
#[pyfunction]
fn function_with_custom_conversion(
    #[pyo3(from_py_with = "datetime_to_timestamp")] timestamp: i64,
) -> i64 {
    timestamp
}

#[cfg(not(Py_LIMITED_API))]
#[test]
fn test_function_with_custom_conversion() {
    Python::with_gil(|py| {
        let custom_conv_func = wrap_pyfunction_bound!(function_with_custom_conversion)(py).unwrap();

        pyo3::py_run!(
            py,
            custom_conv_func,
            r#"
        import datetime

        dt = datetime.datetime.fromtimestamp(1612040400)
        assert custom_conv_func(dt) == 1612040400
        "#
        )
    });
}

#[cfg(not(Py_LIMITED_API))]
#[test]
fn test_function_with_custom_conversion_error() {
    Python::with_gil(|py| {
        let custom_conv_func = wrap_pyfunction_bound!(function_with_custom_conversion)(py).unwrap();

        py_expect_exception!(
            py,
            custom_conv_func,
            "custom_conv_func(['a'])",
            PyTypeError,
            "argument 'timestamp': 'list' object cannot be converted to 'PyDateTime'"
        );
    });
}

#[test]
fn test_from_py_with_defaults() {
    fn optional_int(x: &Bound<'_, PyAny>) -> PyResult<Option<i32>> {
        if x.is_none() {
            Ok(None)
        } else {
            Some(x.extract()).transpose()
        }
    }

    // issue 2280 combination of from_py_with and Option<T> did not compile
    #[pyfunction]
    #[pyo3(signature = (int=None))]
    fn from_py_with_option(#[pyo3(from_py_with = "optional_int")] int: Option<i32>) -> i32 {
        int.unwrap_or(0)
    }

    #[pyfunction(signature = (len=0))]
    fn from_py_with_default(
        #[pyo3(from_py_with = "<Bound<'_, _> as PyAnyMethods>::len")] len: usize,
    ) -> usize {
        len
    }

    Python::with_gil(|py| {
        let f = wrap_pyfunction_bound!(from_py_with_option)(py).unwrap();

        assert_eq!(f.call0().unwrap().extract::<i32>().unwrap(), 0);
        assert_eq!(f.call1((123,)).unwrap().extract::<i32>().unwrap(), 123);
        assert_eq!(f.call1((999,)).unwrap().extract::<i32>().unwrap(), 999);

        let f2 = wrap_pyfunction_bound!(from_py_with_default)(py).unwrap();

        assert_eq!(f2.call0().unwrap().extract::<usize>().unwrap(), 0);
        assert_eq!(f2.call1(("123",)).unwrap().extract::<usize>().unwrap(), 3);
        assert_eq!(f2.call1(("1234",)).unwrap().extract::<usize>().unwrap(), 4);
    });
}

#[pyclass]
#[derive(Debug, FromPyObject)]
struct ValueClass {
    #[pyo3(get)]
    value: usize,
}

#[pyfunction]
#[pyo3(signature=(str_arg, int_arg, tuple_arg, option_arg = None, struct_arg = None))]
fn conversion_error(
    str_arg: &str,
    int_arg: i64,
    tuple_arg: (String, f64),
    option_arg: Option<i64>,
    struct_arg: Option<ValueClass>,
) {
    println!(
        "{:?} {:?} {:?} {:?} {:?}",
        str_arg, int_arg, tuple_arg, option_arg, struct_arg
    );
}

#[test]
fn test_conversion_error() {
    Python::with_gil(|py| {
        let conversion_error = wrap_pyfunction_bound!(conversion_error)(py).unwrap();
        py_expect_exception!(
            py,
            conversion_error,
            "conversion_error(None, None, None, None, None)",
            PyTypeError,
            "argument 'str_arg': 'NoneType' object cannot be converted to 'PyString'"
        );
        py_expect_exception!(
            py,
            conversion_error,
            "conversion_error(100, None, None, None, None)",
            PyTypeError,
            "argument 'str_arg': 'int' object cannot be converted to 'PyString'"
        );
        py_expect_exception!(
            py,
            conversion_error,
            "conversion_error('string1', 'string2', None, None, None)",
            PyTypeError,
            "argument 'int_arg': 'str' object cannot be interpreted as an integer"
        );
        py_expect_exception!(
            py,
            conversion_error,
            "conversion_error('string1', -100, 'string2', None, None)",
            PyTypeError,
            "argument 'tuple_arg': 'str' object cannot be converted to 'PyTuple'"
        );
        py_expect_exception!(
            py,
            conversion_error,
            "conversion_error('string1', -100, ('string2', 10.), 'string3', None)",
            PyTypeError,
            "argument 'option_arg': 'str' object cannot be interpreted as an integer"
        );
        let exception = py_expect_exception!(
            py,
            conversion_error,
            "
class ValueClass:
    def __init__(self, value):
        self.value = value
conversion_error('string1', -100, ('string2', 10.), None, ValueClass(\"no_expected_type\"))",
            PyTypeError
        );
        assert_eq!(
            extract_traceback(py, exception),
            "TypeError: argument 'struct_arg': failed to \
    extract field ValueClass.value: TypeError: 'str' object cannot be interpreted as an integer"
        );

        let exception = py_expect_exception!(
            py,
            conversion_error,
            "
class ValueClass:
    def __init__(self, value):
        self.value = value
conversion_error('string1', -100, ('string2', 10.), None, ValueClass(-5))",
            PyTypeError
        );
        assert_eq!(
            extract_traceback(py, exception),
            "TypeError: argument 'struct_arg': failed to \
    extract field ValueClass.value: OverflowError: can't convert negative int to unsigned"
        );
    });
}

/// Helper function that concatenates the error message from
/// each error in the traceback into a single string that can
/// be tested.
fn extract_traceback(py: Python<'_>, mut error: PyErr) -> String {
    let mut error_msg = error.to_string();
    while let Some(cause) = error.cause(py) {
        error_msg.push_str(": ");
        error_msg.push_str(&cause.to_string());
        error = cause
    }
    error_msg
}

#[test]
fn test_pycfunction_new() {
    use pyo3::ffi;

    Python::with_gil(|py| {
        unsafe extern "C" fn c_fn(
            _self: *mut ffi::PyObject,
            _args: *mut ffi::PyObject,
        ) -> *mut ffi::PyObject {
            ffi::PyLong_FromLong(4200)
        }

        let py_fn = PyCFunction::new_bound(
            py,
            c_fn,
            "py_fn",
            "py_fn for test (this is the docstring)",
            None,
        )
        .unwrap();

        py_assert!(py, py_fn, "py_fn() == 4200");
        py_assert!(
            py,
            py_fn,
            "py_fn.__doc__ == 'py_fn for test (this is the docstring)'"
        );
    });
}

#[test]
fn test_pycfunction_new_with_keywords() {
    use pyo3::ffi;
    use std::ffi::CString;
    use std::os::raw::{c_char, c_long};
    use std::ptr;

    Python::with_gil(|py| {
        unsafe extern "C" fn c_fn(
            _self: *mut ffi::PyObject,
            args: *mut ffi::PyObject,
            kwds: *mut ffi::PyObject,
        ) -> *mut ffi::PyObject {
            let mut foo: c_long = 0;
            let mut bar: c_long = 0;
            let foo_ptr: *mut c_long = &mut foo;
            let bar_ptr: *mut c_long = &mut bar;

            let foo_name = CString::new("foo").unwrap();
            let foo_name_raw: *mut c_char = foo_name.into_raw();
            let kw_bar_name = CString::new("kw_bar").unwrap();
            let kw_bar_name_raw: *mut c_char = kw_bar_name.into_raw();

            let mut arglist = vec![foo_name_raw, kw_bar_name_raw, ptr::null_mut()];
            let arglist_ptr: *mut *mut c_char = arglist.as_mut_ptr();

            let arg_pattern: *const c_char = CString::new("l|l").unwrap().into_raw();

            ffi::PyArg_ParseTupleAndKeywords(
                args,
                kwds,
                arg_pattern,
                arglist_ptr,
                foo_ptr,
                bar_ptr,
            );

            ffi::PyLong_FromLong(foo * bar)
        }

        let py_fn = PyCFunction::new_with_keywords_bound(
            py,
            c_fn,
            "py_fn",
            "py_fn for test (this is the docstring)",
            None,
        )
        .unwrap();

        py_assert!(py, py_fn, "py_fn(42, kw_bar=100) == 4200");
        py_assert!(py, py_fn, "py_fn(foo=42, kw_bar=100) == 4200");
        py_assert!(
            py,
            py_fn,
            "py_fn.__doc__ == 'py_fn for test (this is the docstring)'"
        );
    });
}

#[test]
fn test_closure() {
    Python::with_gil(|py| {
        let f = |args: &Bound<'_, types::PyTuple>,
                 _kwargs: Option<&Bound<'_, types::PyDict>>|
         -> PyResult<_> {
            Python::with_gil(|py| {
                let res: Vec<_> = args
                    .iter()
                    .map(|elem| {
                        if let Ok(i) = elem.extract::<i64>() {
                            (i + 1).into_py(py)
                        } else if let Ok(f) = elem.extract::<f64>() {
                            (2. * f).into_py(py)
                        } else if let Ok(mut s) = elem.extract::<String>() {
                            s.push_str("-py");
                            s.into_py(py)
                        } else {
                            panic!("unexpected argument type for {:?}", elem)
                        }
                    })
                    .collect();
                Ok(res)
            })
        };
        let closure_py =
            PyCFunction::new_closure_bound(py, Some("test_fn"), Some("test_fn doc"), f).unwrap();

        py_assert!(py, closure_py, "closure_py(42) == [43]");
        py_assert!(py, closure_py, "closure_py.__name__ == 'test_fn'");
        py_assert!(py, closure_py, "closure_py.__doc__ == 'test_fn doc'");
        py_assert!(
            py,
            closure_py,
            "closure_py(42, 3.14, 'foo') == [43, 6.28, 'foo-py']"
        );
    });
}

#[test]
fn test_closure_counter() {
    Python::with_gil(|py| {
        let counter = std::cell::RefCell::new(0);
        let counter_fn = move |_args: &Bound<'_, types::PyTuple>,
                               _kwargs: Option<&Bound<'_, types::PyDict>>|
              -> PyResult<i32> {
            let mut counter = counter.borrow_mut();
            *counter += 1;
            Ok(*counter)
        };
        let counter_py = PyCFunction::new_closure_bound(py, None, None, counter_fn).unwrap();

        py_assert!(py, counter_py, "counter_py() == 1");
        py_assert!(py, counter_py, "counter_py() == 2");
        py_assert!(py, counter_py, "counter_py() == 3");
    });
}

#[test]
fn use_pyfunction() {
    mod function_in_module {
        use pyo3::prelude::*;

        #[pyfunction]
        pub fn foo(x: i32) -> i32 {
            x
        }
    }

    Python::with_gil(|py| {
        use function_in_module::foo;

        // check imported name can be wrapped
        let f = wrap_pyfunction_bound!(foo, py).unwrap();
        assert_eq!(f.call1((5,)).unwrap().extract::<i32>().unwrap(), 5);
        assert_eq!(f.call1((42,)).unwrap().extract::<i32>().unwrap(), 42);

        // check path import can be wrapped
        let f2 = wrap_pyfunction_bound!(function_in_module::foo, py).unwrap();
        assert_eq!(f2.call1((5,)).unwrap().extract::<i32>().unwrap(), 5);
        assert_eq!(f2.call1((42,)).unwrap().extract::<i32>().unwrap(), 42);
    })
}

#[pyclass]
struct Key(String);

#[pyclass]
struct Value(i32);

#[pyfunction]
fn return_value_borrows_from_arguments<'py>(
    py: Python<'py>,
    key: &'py Key,
    value: &'py Value,
) -> HashMap<&'py str, i32> {
    py.allow_threads(move || {
        let mut map = HashMap::new();
        map.insert(key.0.as_str(), value.0);
        map
    })
}

#[test]
fn test_return_value_borrows_from_arguments() {
    Python::with_gil(|py| {
        let function = wrap_pyfunction_bound!(return_value_borrows_from_arguments, py).unwrap();

        let key = Py::new(py, Key("key".to_owned())).unwrap();
        let value = Py::new(py, Value(42)).unwrap();

        py_assert!(py, function key value, "function(key, value) == { \"key\": 42 }");
    });
}

#[test]
fn test_some_wrap_arguments() {
    // https://github.com/PyO3/pyo3/issues/3460
    const NONE: Option<u8> = None;
    #[pyfunction(signature = (a = 1, b = Some(2), c = None, d = NONE))]
    fn some_wrap_arguments(
        a: Option<u8>,
        b: Option<u8>,
        c: Option<u8>,
        d: Option<u8>,
    ) -> [Option<u8>; 4] {
        [a, b, c, d]
    }

    Python::with_gil(|py| {
        let function = wrap_pyfunction_bound!(some_wrap_arguments, py).unwrap();
        py_assert!(py, function, "function() == [1, 2, None, None]");
    })
}

#[test]
fn test_reference_to_bound_arguments() {
    #[pyfunction]
    #[pyo3(signature = (x, y = None))]
    fn reference_args<'py>(
        x: &Bound<'py, PyAny>,
        y: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        y.map_or_else(|| Ok(x.clone()), |y| y.add(x))
    }

    Python::with_gil(|py| {
        let function = wrap_pyfunction_bound!(reference_args, py).unwrap();
        py_assert!(py, function, "function(1) == 1");
        py_assert!(py, function, "function(1, 2) == 3");
    })
}
