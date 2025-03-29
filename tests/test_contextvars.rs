#![cfg(not(Py_LIMITED_API))]

use pyo3::exceptions::PyRuntimeError;
use pyo3::types::{PyContext, PyContextMethods, PyContextToken, PyContextTokenMethods, PyContextVar, PyContextVarMethods};
use pyo3::prelude::*;
use pyo3_ffi::c_str;

#[test]
fn test_context() {
    Python::with_gil(|py| {
        let context = PyContext::new(py).unwrap();
        assert!(context.is_instance_of::<PyContext>());
        assert!(context.is_exact_instance_of::<PyContext>());
        assert!(!context.is_instance_of::<PyContextVar>());
        assert!(!context.is_exact_instance_of::<PyContextToken>());

        // Copy
        let context2 = context.copy().unwrap();
        assert!(context2.is_exact_instance_of::<PyContext>());
        assert!(!context.is(&context2));
    });
}

#[test]
fn test_context_copycurrent() {
    Python::with_gil(|py| {
        let current_context = PyContext::copy_current(py).unwrap();
        assert!(current_context.is_exact_instance_of::<PyContext>());

        let current_context2 = PyContext::copy_current(py).unwrap();
        assert!(!current_context.is(&current_context2));
    });
}

#[test]
fn test_contextvar_new() {
    Python::with_gil(|py| {
        let cv = PyContextVar::new(py, c_str!("test")).unwrap();
        assert!(cv.is_exact_instance_of::<PyContextVar>());

        assert!(cv.get().unwrap().is_none());
    });
}


#[test]
fn test_contextvar_set() {
    Python::with_gil(|py| {
        let cv = PyContextVar::new(py, c_str!("test")).unwrap();
        assert!(cv.is_exact_instance_of::<PyContextVar>());

        assert!(cv.get().unwrap().is_none());

        let token = cv.set(1_u64.into_pyobject(py).unwrap()).unwrap();
        assert!(token.is_exact_instance_of::<PyContextToken>());
        assert!(token.old_value().unwrap().is_none());
        assert!(token.var().unwrap().is(&cv));
        assert_eq!(cv.get().unwrap().unwrap().extract::<u64>().unwrap(), 1);

        // Reset to default state
        cv.reset(token.clone()).unwrap();
        assert!(cv.get().unwrap().is_none());

        // Check that we can't reset twice
        {
            let reset_err = cv.reset(token).unwrap_err();
            assert!(reset_err.is_instance_of::<PyRuntimeError>(py));
            assert!(reset_err.to_string().ends_with(" has already been used once"));
        }
    });
}