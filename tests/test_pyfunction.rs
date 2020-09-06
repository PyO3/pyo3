use pyo3::buffer::PyBuffer;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

mod common;

#[pyfunction(arg = "true")]
fn optional_bool(arg: Option<bool>) -> String {
    format!("{:?}", arg)
}

#[test]
fn test_optional_bool() {
    // Regression test for issue #932
    let gil = Python::acquire_gil();
    let py = gil.python();
    let f = wrap_pyfunction!(optional_bool)(py).unwrap();

    py_assert!(py, f, "f() == 'Some(true)'");
    py_assert!(py, f, "f(True) == 'Some(true)'");
    py_assert!(py, f, "f(False) == 'Some(false)'");
    py_assert!(py, f, "f(None) == 'None'");
}

#[pyfunction]
fn buffer_inplace_add(py: Python, x: PyBuffer<i32>, y: PyBuffer<i32>) {
    let x = x.as_mut_slice(py).unwrap();
    let y = y.as_slice(py).unwrap();
    for (xi, yi) in x.iter().zip(y) {
        let xi_plus_yi = xi.get() + yi.get();
        xi.set(xi_plus_yi);
    }
}

#[test]
fn test_buffer_add() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let f = wrap_pyfunction!(buffer_inplace_add)(py).unwrap();

    py_expect_exception!(
        py,
        f,
        r#"
import array
a = array.array("i", [0, 1, 2, 3])
b = array.array("I", [0, 1, 2, 3])
f(a, b)
"#,
        PyBufferError
    );

    pyo3::py_run!(
        py,
        f,
        r#"
import array
a = array.array("i", [0, 1, 2, 3])
b = array.array("i", [2, 3, 4, 5])
f(a, b)
assert a, array.array("i", [2, 4, 6, 8])
"#
    );
}
