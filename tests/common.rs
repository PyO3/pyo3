//! Some common macros for tests

#[macro_export]
macro_rules! py_assert {
    ($py:expr, $($val:ident)+, $assertion:literal) => {
        pyo3::py_run!($py, $($val)+, concat!("assert ", $assertion))
    };
    ($py:expr, *$dict:expr, $assertion:literal) => {
        pyo3::py_run!($py, *$dict, concat!("assert ", $assertion))
    };
}

#[macro_export]
macro_rules! py_expect_exception {
    // Case1: idents & no err_msg
    ($py:expr, $($val:ident)+, $code:expr, $err:ident) => {{
        use pyo3::types::IntoPyDict;
        let d = [$((stringify!($val), $val.to_object($py)),)+].into_py_dict($py);
        py_expect_exception!($py, *d, $code, $err)
    }};
    // Case2: dict & no err_msg
    ($py:expr, *$dict:expr, $code:expr, $err:ident) => {{
        let res = $py.run($code, None, Some($dict));
        let err = res.expect_err(&format!("Did not raise {}", stringify!($err)));
        if !err.matches($py, $py.get_type::<pyo3::exceptions::$err>()) {
            panic!("Expected {} but got {:?}", stringify!($err), err)
        }
        err
    }};
    // Case3: idents & err_msg
    ($py:expr, $($val:ident)+, $code:expr, $err:ident, $err_msg:literal) => {{
        let err = py_expect_exception!($py, $($val)+, $code, $err);
        // Suppose that the error message looks like 'TypeError: ~'
        assert_eq!(format!("Py{}", err), concat!(stringify!($err), ": ", $err_msg));
        err
    }};
    // Case4: dict & err_msg
    ($py:expr, *$dict:expr, $code:expr, $err:ident, $err_msg:literal) => {{
        let err = py_expect_exception!($py, *$dict, $code, $err);
        assert_eq!(format!("Py{}", err), concat!(stringify!($err), ": ", $err_msg));
        err
    }};
}
