// the inner mod enables the #![allow(dead_code)] to
// be applied - `test_utils.rs` uses `include!` to pull in this file

/// Common macros and helpers for tests
#[allow(dead_code)] // many tests do not use the complete set of functionality offered here
#[macro_use]
mod inner {

    #[allow(unused_imports)] // pulls in `use crate as pyo3` in `test_utils.rs`
    use super::*;

    use pyo3::prelude::*;

    use pyo3::types::{IntoPyDict, PyList};

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
    macro_rules! assert_py_eq {
        ($val:expr, $expected:expr) => {
            assert!($val.eq($expected).unwrap());
        };
    }

    #[macro_export]
    macro_rules! py_expect_exception {
        // Case1: idents & no err_msg
        ($py:expr, $($val:ident)+, $code:expr, $err:ident) => {{
            use pyo3::types::IntoPyDict;
            let d = [$((stringify!($val), $val.to_object($py)),)+].into_py_dict_bound($py);
            py_expect_exception!($py, *d, $code, $err)
        }};
        // Case2: dict & no err_msg
        ($py:expr, *$dict:expr, $code:expr, $err:ident) => {{
            let res = $py.run_bound($code, None, Some(&$dict.as_borrowed()));
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
    #[pyclass(crate = "pyo3")]
    pub struct UnraisableCapture {
        pub capture: Option<(PyErr, PyObject)>,
        old_hook: Option<PyObject>,
    }

    #[cfg(all(feature = "macros", Py_3_8))]
    #[pymethods(crate = "pyo3")]
    impl UnraisableCapture {
        pub fn hook(&mut self, unraisable: &PyAny) {
            let err =
                PyErr::from_value_bound(&unraisable.getattr("exc_value").unwrap().as_borrowed());
            let instance = unraisable.getattr("object").unwrap();
            self.capture = Some((err, instance.into()));
        }
    }

    #[cfg(all(feature = "macros", Py_3_8))]
    impl UnraisableCapture {
        pub fn install(py: Python<'_>) -> Py<Self> {
            let sys = py.import_bound("sys").unwrap();
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

            let sys = py.import_bound("sys").unwrap();
            sys.setattr("unraisablehook", old_hook).unwrap();
        }
    }

    pub struct CatchWarnings<'py> {
        catch_warnings: Bound<'py, PyAny>,
    }

    impl<'py> CatchWarnings<'py> {
        pub fn enter<R>(py: Python<'py>, f: impl FnOnce(&PyList) -> PyResult<R>) -> PyResult<R> {
            let warnings = py.import_bound("warnings")?;
            let kwargs = [("record", true)].into_py_dict_bound(py);
            let catch_warnings = warnings
                .getattr("catch_warnings")?
                .call((), Some(&kwargs))?;
            let list = catch_warnings.call_method0("__enter__")?.extract()?;
            let _guard = Self { catch_warnings };
            f(list)
        }
    }

    impl Drop for CatchWarnings<'_> {
        fn drop(&mut self) {
            let py = self.catch_warnings.py();
            self.catch_warnings
                .call_method1("__exit__", (py.None(), py.None(), py.None()))
                .unwrap();
        }
    }

    #[macro_export]
    macro_rules! assert_warnings {
        ($py:expr, $body:expr, [$(($category:ty, $message:literal)),+] $(,)? ) => {{
            $crate::tests::common::CatchWarnings::enter($py, |w| {
                $body;
                let expected_warnings = [$((<$category as $crate::type_object::PyTypeInfo>::type_object_bound($py), $message)),+];
                assert_eq!(w.len(), expected_warnings.len());
                for (warning, (category, message)) in w.iter().zip(expected_warnings) {

                    assert!(warning.getattr("category").unwrap().is(&category));
                    assert_eq!(
                        warning.getattr("message").unwrap().str().unwrap().to_string_lossy(),
                        message
                    );
                }

                Ok(())
            })
            .unwrap();
        }};
    }
}

#[allow(unused_imports)] // some tests use just the macros and none of the other functionality
pub use inner::*;
