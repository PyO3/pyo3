#![feature(concat_idents)]

extern crate pyo3;

use pyo3::prelude::*;

use pyo3::ffi::*;

fn _get_subclasses<'p>(py: &'p Python, py_type: &str, args: &str) ->
    (&'p PyObjectRef, &'p PyObjectRef, &'p PyObjectRef) {
    macro_rules! unwrap_py {
        ($e:expr) => { ($e).map_err(|e| e.print(*py)).unwrap() }
    };

    // Import the class from Python and create some subclasses
    let datetime = unwrap_py!(py.import("datetime"));

    let locals = PyDict::new(*py);
    locals.set_item(py_type, datetime.get(py_type).unwrap())
        .unwrap();

    let make_subclass_py =
        format!("class Subklass({}):\n    pass", py_type);

    let make_sub_subclass_py =
        "class SubSubklass(Subklass):\n    pass";

    unwrap_py!(py.run(&make_subclass_py, None, Some(&locals)));
    unwrap_py!(py.run(&make_sub_subclass_py, None, Some(&locals)));

    // Construct an instance of the base class
    let obj = unwrap_py!(
        py.eval(&format!("{}({})", py_type, args), None, Some(&locals))
    );

    // Construct an instance of the subclass
    let sub_obj = unwrap_py!(
        py.eval(&format!("Subklass({})", args), None, Some(&locals))
    );

    // Construct an instance of the sub-subclass
    let sub_sub_obj = unwrap_py!(
        py.eval(&format!("SubSubklass({})", args), None, Some(&locals))
    );

    (obj, sub_obj, sub_sub_obj)
}

macro_rules! assert_check_exact {
    ($check_func:ident, $obj: expr) => {
        unsafe {
            assert!($check_func(($obj).as_ptr()) != 0);
            assert!(concat_idents!($check_func, Exact)(($obj).as_ptr()) != 0);
        }
    }
}

macro_rules! assert_check_only {
    ($check_func:ident, $obj: expr) => {
        unsafe {
            assert!($check_func(($obj).as_ptr()) != 0);
            assert!(concat_idents!($check_func, Exact)(($obj).as_ptr()) == 0);
        }
    }
}


#[test]
fn test_date_check() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let (obj, sub_obj, sub_sub_obj) = _get_subclasses(&py,
        "date", "2018, 1, 1"
        );

    assert_check_exact!(PyDate_Check, obj);
    assert_check_only!(PyDate_Check, sub_obj);
    assert_check_only!(PyDate_Check, sub_sub_obj);
}

#[test]
fn test_time_check() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let (obj, sub_obj, sub_sub_obj) = _get_subclasses(&py,
        "time", "12, 30, 15"
        );

    assert_check_exact!(PyTime_Check, obj);
    assert_check_only!(PyTime_Check, sub_obj);
    assert_check_only!(PyTime_Check, sub_sub_obj);
}

#[test]
fn test_datetime_check() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let (obj, sub_obj, sub_sub_obj) = _get_subclasses(&py,
        "datetime", "2018, 1, 1, 13, 30, 15"
        );

    assert_check_only!(PyDate_Check, obj);
    assert_check_exact!(PyDateTime_Check, obj);
    assert_check_only!(PyDateTime_Check, sub_obj);
    assert_check_only!(PyDateTime_Check, sub_sub_obj);
}

#[test]
fn test_delta_check() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let (obj, sub_obj, sub_sub_obj) = _get_subclasses(&py,
        "timedelta", "1, -3"
        );

    assert_check_exact!(PyDelta_Check, obj);
    assert_check_only!(PyDelta_Check, sub_obj);
    assert_check_only!(PyDelta_Check, sub_sub_obj);
}
