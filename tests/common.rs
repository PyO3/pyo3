//! Some common macros for tests

#[cfg(all(feature = "macros", Py_3_8))]
use pyo3::prelude::*;

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
        let res = $py.run($code, None, Some($dict.as_gil_ref()));
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

// sys.unraisablehook not available until Python 3.8
#[cfg(all(feature = "macros", Py_3_8))]
#[pyclass]
pub struct UnraisableCapture {
    pub capture: Option<(PyErr, PyObject)>,
    old_hook: Option<PyObject>,
}

#[cfg(all(feature = "macros", Py_3_8))]
#[pymethods]
impl UnraisableCapture {
    pub fn hook(&mut self, unraisable: &PyAny) {
        let err = PyErr::from_value(unraisable.getattr("exc_value").unwrap());
        let instance = unraisable.getattr("object").unwrap();
        self.capture = Some((err, instance.into()));
    }
}

#[cfg(all(feature = "macros", Py_3_8))]
impl UnraisableCapture {
    pub fn install(py: Python<'_>) -> Py<Self> {
        let sys = py.import("sys").unwrap();
        let old_hook = sys.getattr("unraisablehook").unwrap().into();

        let capture = Py::new(
            py,
            UnraisableCapture {
                capture: None,
                old_hook: Some(old_hook),
            },
        )
        .unwrap();

        sys.setattr("unraisablehook", capture.getattr(py, "hook").unwrap())
            .unwrap();

        capture
    }

    pub fn uninstall(&mut self, py: Python<'_>) {
        let old_hook = self.old_hook.take().unwrap();

        let sys = py.import("sys").unwrap();
        sys.setattr("unraisablehook", old_hook).unwrap();
    }
}
