//! Useful tips for writing tests:
//!  - Tests are run in parallel; There's still a race condition in test_owned with some other test
//!  - You need to use flush=True to get any output from print

/// Removes indentation from multiline strings in pyrun commands
#[allow(unused)] // macro scoping is fooling the compiler
pub fn indoc(commands: &str) -> String {
    let indent;
    if let Some(second) = commands.lines().nth(1) {
        indent = second
            .chars()
            .take_while(char::is_ascii_whitespace)
            .collect::<String>();
    } else {
        indent = "".to_string();
    }

    commands
        .trim_end()
        .replace(&("\n".to_string() + &indent), "\n")
        + "\n"
}

#[macro_export]
macro_rules! py_run {
    ($py:expr, $val:expr, $code:expr) => {{
        use pyo3::types::IntoPyDict;
        let d = [(stringify!($val), &$val)].into_py_dict($py);

        $py.run(&common::indoc($code), None, Some(d))
            .map_err(|e| {
                e.print($py);
                // So when this c api function the last line called printed the error to stderr,
                // the output is only written into a buffer which is never flushed because we
                // panic before flushing. This is where this hack comes into place
                $py.run("import sys; sys.stderr.flush()", None, None)
                    .unwrap();
            })
            .expect(&common::indoc($code))
    }};
}

#[macro_export]
macro_rules! py_assert {
    ($py:expr, $val:ident, $assertion:expr) => {
        py_run!($py, $val, concat!("assert ", $assertion))
    };
}

#[macro_export]
macro_rules! py_expect_exception {
    ($py:expr, $val:ident, $code:expr, $err:ident) => {{
        use pyo3::types::IntoPyDict;
        let d = [(stringify!($val), &$val)].into_py_dict($py);

        let res = $py.run($code, None, Some(d));
        let err = res.unwrap_err();
        if !err.matches($py, $py.get_type::<pyo3::exceptions::$err>()) {
            panic!(format!("Expected {} but got {:?}", stringify!($err), err))
        }
    }};
}
