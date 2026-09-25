//! Tests for the `pyo3::stdio` module.
//!
//! These redirect Python's `sys.stdout` and `sys.stderr` to a `StringIO` object,
//! so they run in a separate process to avoid interfering with other tests.

use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::stdio::*;
use pyo3::types::IntoPyDict;
use std::ffi::CString;
use std::io::Write;

#[macro_use]
#[path = "../src/tests/common.rs"]
mod common;

#[test]
fn test_stdio() {
    let stream_fcns = [stdout, stderr, __stdout__, __stderr__];
    let stream_names = ["stdout", "stderr", "__stdout__", "__stderr__"];

    for (stream_fcn, stream_name) in stream_fcns.iter().zip(stream_names.iter()) {
        Python::attach(|py| {
            py.run(ffi::c_str!("import sys, io"), None, None).unwrap();

            // redirect stdout or stderr output to a StringIO object
            let target = py.eval(ffi::c_str!("io.StringIO()"), None, None).unwrap();
            let locals = [("target", &target)].into_py_dict(py).unwrap();
            py.run(
                &CString::new(format!("sys.{} = target", stream_name)).unwrap(),
                None,
                Some(&locals),
            )
            .unwrap();

            let mut stream = stream_fcn();
            assert!(writeln!(stream, "writing to {}", stream_name).is_ok());

            py.run(
                &CString::new(format!(
                    "assert target.getvalue() == 'writing to {}\\n'",
                    stream_name
                ))
                .unwrap(),
                Some(&locals),
                None,
            )
            .unwrap();
        });
    }
}
