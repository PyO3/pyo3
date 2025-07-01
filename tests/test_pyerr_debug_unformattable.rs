use pyo3::ffi;
use pyo3::prelude::*;

// This test mucks around with sys.modules, so run it separately to prevent it
// from potentially corrupting the state of the python interpreter used in other
// tests.

#[test]
fn err_debug_unformattable() {
    // Debug representation should be like the following (without the newlines):
    // PyErr {
    //     type: <class 'Exception'>,
    //     value: Exception('banana'),
    //     traceback: Some(\"<unformattable <traceback object at 0x...>>\")
    // }

    Python::attach(|py| {
        // PyTracebackMethods::format uses io.StringIO. Mock it out to trigger a
        // formatting failure:
        // TypeError: 'Mock' object cannot be converted to 'PyString'
        let err = py
            .run(
                ffi::c_str!(
                    r#"
import io, sys, unittest.mock
sys.modules['orig_io'] = sys.modules['io']
sys.modules['io'] = unittest.mock.Mock()
raise Exception('banana')"#
                ),
                None,
                None,
            )
            .expect_err("raising should have given us an error");

        let debug_str = format!("{err:?}");
        assert!(debug_str.starts_with("PyErr { "));
        assert!(debug_str.ends_with(" }"));

        // Strip "PyErr { " and " }". Split into 3 substrings to separate type,
        // value, and traceback while not splitting the string within traceback.
        let mut fields = debug_str["PyErr { ".len()..debug_str.len() - 2].splitn(3, ", ");

        assert_eq!(fields.next().unwrap(), "type: <class 'Exception'>");
        assert_eq!(fields.next().unwrap(), "value: Exception('banana')");

        let traceback = fields.next().unwrap();
        assert!(
            traceback.starts_with("traceback: Some(\"<unformattable <traceback object at 0x"),
            "assertion failed, actual traceback str: {traceback:?}"
        );
        assert!(fields.next().is_none());

        py.run(
            ffi::c_str!(
                r#"
import io, sys, unittest.mock
sys.modules['io'] = sys.modules['orig_io']
del sys.modules['orig_io']
"#
            ),
            None,
            None,
        )
        .unwrap();
    });
}
