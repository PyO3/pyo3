use pyo3::buffer::PyBuffer;
use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyFunction};
use pyo3::{raw_pycfunction, wrap_pyfunction};

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

#[pyfunction]
fn function_with_pyfunction_arg(fun: &PyFunction) -> PyResult<&PyAny> {
    fun.call((), None)
}

#[pyfunction]
fn function_with_pycfunction_arg(fun: &PyCFunction) -> PyResult<&PyAny> {
    fun.call((), None)
}

#[test]
fn test_functions_with_function_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let py_func_arg = wrap_pyfunction!(function_with_pyfunction_arg)(py).unwrap();
    let py_cfunc_arg = wrap_pyfunction!(function_with_pycfunction_arg)(py).unwrap();
    let bool_to_string = wrap_pyfunction!(optional_bool)(py).unwrap();

    pyo3::py_run!(
        py,
        py_func_arg
        py_cfunc_arg
        bool_to_string,
        r#"
        def foo(): return "bar"
        assert py_func_arg(foo) == "bar"
        assert py_cfunc_arg(bool_to_string) == "Some(true)"
        "#
    )
}

#[test]
fn test_raw_function() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let raw_func = raw_pycfunction!(optional_bool);
    let fun = PyCFunction::new_with_keywords(raw_func, "fun", "", py.into()).unwrap();
    let res = fun.call((), None).unwrap().extract::<&str>().unwrap();
    assert_eq!(res, "Some(true)");
    let res = fun.call((false,), None).unwrap().extract::<&str>().unwrap();
    assert_eq!(res, "Some(false)");
    let no_module = fun.getattr("__module__").unwrap().is_none();
    assert!(no_module);

    let module = PyModule::new(py, "cool_module").unwrap();
    module.add_function(fun).unwrap();
    let res = module
        .getattr("fun")
        .unwrap()
        .call((), None)
        .unwrap()
        .extract::<&str>()
        .unwrap();
    assert_eq!(res, "Some(true)");
}
