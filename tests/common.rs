//! Useful tips for writing tests:
//!  - Tests are run in parallel; There's still a race condition in test_owned with some other test
//!  - You need to use flush=True to get any output from print

#[macro_export]
macro_rules! py_assert {
    ($py:expr, $val:ident, $assertion:expr) => {
        pyo3::py_run!($py, $val, concat!("assert ", $assertion))
    };
}

#[macro_export]
macro_rules! py_expect_exception {
    ($py:expr, $val:ident, $code:expr, $err:ident) => {{
        use pyo3::types::IntoPyDict;
        let d = [(stringify!($val), &$val)].into_py_dict($py);

        let res = $py.run($code, None, Some(d));
        let err = res.expect_err(&format!("Did not raise {}", stringify!($err)));
        if !err.matches($py, $py.get_type::<pyo3::exceptions::$err>()) {
            panic!("Expected {} but got {:?}", stringify!($err), err)
        }
        err
    }};
    ($py:expr, $val:ident, $code:expr, $err:ident, $err_msg:expr) => {{
        let err = py_expect_exception!($py, $val, $code, $err);
        assert_eq!(
            err.instance($py)
                .str()
                .expect("error str() failed")
                .to_str()
                .expect("message was not valid utf8"),
            $err_msg
        );
        err
    }};
}
