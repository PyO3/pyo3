#[macro_export]
macro_rules! py_run {
    ($py:expr, $val:ident, $code:expr) => {{
        let d = PyDict::new($py);
        d.set_item(stringify!($val), &$val).unwrap();
        $py.run($code, None, Some(d))
            .map_err(|e| e.print($py))
            .expect($code);
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
