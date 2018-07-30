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
        .trim_right()
        .replace(&("\n".to_string() + &indent), "\n")
        + "\n"
}

#[macro_export]
macro_rules! py_run {
    ($py:expr, $val:expr, $code:expr) => {{
        let d = PyDict::new($py);
        d.set_item(stringify!($val), &$val).unwrap();
        $py.run(&common::indoc($code), None, Some(d))
            .map_err(|e| e.print($py))
            .expect(&common::indoc($code));
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
        let d = PyDict::new($py);
        d.set_item(stringify!($val), &$val).unwrap();
        let res = $py.run($code, None, Some(d));
        let err = res.unwrap_err();
        if !err.matches($py, $py.get_type::<exc::$err>()) {
            panic!(format!("Expected {} but got {:?}", stringify!($err), err))
        }
    }};
}
