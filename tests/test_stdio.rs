#![cfg(feature = "macros")]

use pyo3::prelude::*;

#[macro_use]
#[path = "../src/tests/common.rs"]
mod common;

#[test]
fn test_stdio() {
    use pyo3::stdio::*;
    use pyo3::types::IntoPyDict;
    use std::io::Write;

    let stream_fcns = [stdout, stderr, __stdout__, __stderr__];
    let stream_names = ["stdout", "stderr", "__stdout__", "__stderr__"];

    for (stream_fcn, stream_name) in stream_fcns.iter().zip(stream_names.iter()) {
        Python::with_gil(|py| {
            py.run_bound("import sys, io", None, None).unwrap();

            // redirect stdout or stderr output to a StringIO object
            let target = py.eval_bound("io.StringIO()", None, None).unwrap();
            let locals = [("target", &target)].into_py_dict_bound(py);
            py.run_bound(
                &format!("sys.{} = target", stream_name),
                None,
                Some(&locals),
            )
            .unwrap();

            let mut stream = stream_fcn();
            assert!(writeln!(stream, "writing to {}", stream_name).is_ok());

            Python::run_bound(
                py,
                &format!(
                    "assert target.getvalue() == 'writing to {}\\n'",
                    stream_name
                ),
                Some(&locals),
                None,
            )
            .unwrap();
        });
    }
}
