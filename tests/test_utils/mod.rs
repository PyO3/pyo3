// Common macros and helpers for tests
//
// This file is used in two different ways, which makes it a bit of a pain to build:
// - as a module `include!`-ed from `src/test_utils.rs`
// - as a module `mod test_utils` in various integration tests
//
// the inner mod enables the #![allow(dead_code)] to
// be applied - `src/test_utils.rs` uses `include!` to pull in this file

#[allow(dead_code, unused_macros)] // many tests do not use the complete set of functionality offered here
#[allow(missing_docs)] // only used in tests
#[macro_use]
mod inner {

    #[allow(unused_imports)]
    // pulls in `use crate as pyo3` in `src/test_utils.rs`, no function in integration tests
    use super::*;

    use pyo3::prelude::*;

    use pyo3::sync::MutexExt;
    use pyo3::types::{IntoPyDict, PyList};

    use std::sync::{Mutex, PoisonError};

    use uuid::Uuid;

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
            use pyo3::BoundObject;
            let d = [$((stringify!($val), (&$val).into_pyobject($py).unwrap().into_any().into_bound()),)+].into_py_dict($py).unwrap();
            py_expect_exception!($py, *d, $code, $err)
        }};
        // Case2: dict & no err_msg
        ($py:expr, *$dict:expr, $code:expr, $err:ident) => {{
            let res = $py.run(&std::ffi::CString::new($code).unwrap(), None, Some(&$dict.as_borrowed()));
            let err = res.expect_err(&format!("Did not raise {}", stringify!($err)));
            if !err.matches($py, $py.get_type::<pyo3::exceptions::$err>()).unwrap() {
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

    #[macro_export]
    macro_rules! py_expect_warning {
        ($py:expr, $($val:ident)+, $code:expr, [$(($warning_msg:literal, $warning_category:path)),+] $(,)?) => {{
            use pyo3::types::IntoPyDict;
            let d = [$((stringify!($val), ($val.as_ref() as &Bound<'_, PyAny>).into_pyobject($py).expect("Failed to create test dict element")),)+].into_py_dict($py).expect("Failed to create test dict");
            py_expect_warning!($py, *d, $code, [$(($warning_msg, $warning_category)),+])
        }};
        ($py:expr, *$dict:expr, $code:expr, [$(($warning_msg:literal, $warning_category:path)),+] $(,)?) => {{
            $crate::test_utils::CatchWarnings::enter($py, |warning_record| {
                $py.run(&std::ffi::CString::new($code).unwrap(), None, Some(&$dict.as_borrowed())).expect("Failed to run warning testing code");
                let expected_warnings = [$(($warning_msg, <$warning_category as pyo3::PyTypeInfo>::type_object($py))),+];

                assert_eq!(warning_record.len(), expected_warnings.len(), "Expecting {} warnings but got {}", expected_warnings.len(), warning_record.len());

                for ((index, warning), (msg, category)) in warning_record.iter().enumerate().zip(expected_warnings.iter()) {
                    let actual_msg = warning.getattr("message").unwrap().str().unwrap().to_string_lossy().to_string();
                    let actual_category = warning.getattr("category").unwrap();

                    assert_eq!(actual_msg, msg.to_string(), "Warning message mismatch at index {}, expecting `{}` but got `{}`", index, msg, actual_msg);
                    assert!(actual_category.is(category), "Warning category mismatch at index {}, expecting {:?} but got {:?}", index, category, actual_category);
                }

                Ok(())
            }).expect("failed to test warnings");
        }};
    }

    #[macro_export]
    macro_rules! py_expect_warning_for_fn {
        ($fn:ident, $($val:ident)+, [$(($warning_msg:literal, $warning_category:path)),+] $(,)?) => {
            pyo3::Python::attach(|py| {
                let f = wrap_pyfunction!($fn)(py).unwrap();
                py_expect_warning!(
                    py,
                    f,
                    "f()",
                    [$(($warning_msg, $warning_category)),+]
                );
            });
        };
    }

    // sys.unraisablehook not available until Python 3.8
    #[cfg(all(feature = "macros", Py_3_8, not(Py_GIL_DISABLED)))]
    #[pyclass(crate = "pyo3")]
    pub struct UnraisableCapture {
        pub capture: Option<(PyErr, Py<PyAny>)>,
        old_hook: Option<Py<PyAny>>,
    }

    #[cfg(all(feature = "macros", Py_3_8, not(Py_GIL_DISABLED)))]
    #[pymethods(crate = "pyo3")]
    impl UnraisableCapture {
        pub fn hook(&mut self, unraisable: Bound<'_, PyAny>) {
            let err = PyErr::from_value(unraisable.getattr("exc_value").unwrap());
            let instance = unraisable.getattr("object").unwrap();
            self.capture = Some((err, instance.into()));
        }
    }

    #[cfg(all(feature = "macros", Py_3_8, not(Py_GIL_DISABLED)))]
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

    pub struct CatchWarnings<'py> {
        catch_warnings: Bound<'py, PyAny>,
    }

    /// catch_warnings is not thread-safe, so only one thread can be using this struct at
    /// a time.
    static CATCH_WARNINGS_MUTEX: Mutex<()> = Mutex::new(());

    impl<'py> CatchWarnings<'py> {
        pub fn enter<R>(
            py: Python<'py>,
            f: impl FnOnce(&Bound<'py, PyList>) -> PyResult<R>,
        ) -> PyResult<R> {
            // NB this is best-effort, other tests could always call the warnings API directly.
            let _mutex_guard = CATCH_WARNINGS_MUTEX
                .lock_py_attached(py)
                .unwrap_or_else(PoisonError::into_inner);
            let warnings = py.import("warnings")?;
            let kwargs = [("record", true)].into_py_dict(py)?;
            let catch_warnings = warnings
                .getattr("catch_warnings")?
                .call((), Some(&kwargs))?;
            let list = catch_warnings.call_method0("__enter__")?.cast_into()?;
            let _guard = Self { catch_warnings };
            f(&list)
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

    macro_rules! assert_warnings {
        ($py:expr, $body:expr, [$(($category:ty, $message:literal)),+] $(,)? ) => {{
            $crate::test_utils::CatchWarnings::enter($py, |w| {
                use $crate::types::{PyListMethods, PyStringMethods};
                $body;
                let expected_warnings = [$((<$category as $crate::type_object::PyTypeInfo>::type_object($py), $message)),+];
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

    pub(crate) use assert_warnings;

    pub fn generate_unique_module_name(base: &str) -> std::ffi::CString {
        let uuid = Uuid::new_v4().simple().to_string();
        std::ffi::CString::new(format!("{base}_{uuid}")).unwrap()
    }
}

#[allow(unused_imports)] // some tests use just the macros and none of the other functionality
pub use inner::*;
