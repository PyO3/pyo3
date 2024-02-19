#![cfg(feature = "macros")]

use pyo3::prelude::*;

use pyo3::py_run;
use pyo3::types::PyString;
use pyo3::types::{IntoPyDict, PyDict, PyTuple};

#[path = "../src/tests/common.rs"]
mod common;

#[pyclass]
struct AnonClass {}

#[pyclass]
struct ValueClass {
    value: usize,
}

#[pymethods]
impl ValueClass {
    #[new]
    fn new(value: usize) -> ValueClass {
        ValueClass { value }
    }
}

#[pyclass(module = "module")]
struct LocatedClass {}

#[pyfunction]
/// Doubles the given value
fn double(x: usize) -> usize {
    x * 2
}

/// This module is implemented in Rust.
#[pymodule]
fn module_with_functions(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    #[pyfn(m)]
    #[pyo3(name = "no_parameters")]
    fn function_with_name() -> usize {
        42
    }

    #[pyfn(m)]
    #[pyo3(pass_module)]
    fn with_module(module: &PyModule) -> PyResult<&str> {
        module.name()
    }

    #[pyfn(m)]
    fn double_value(v: &ValueClass) -> usize {
        v.value * 2
    }

    m.add_class::<AnonClass>().unwrap();
    m.add_class::<ValueClass>().unwrap();
    m.add_class::<LocatedClass>().unwrap();

    m.add("foo", "bar").unwrap();

    m.add_function(wrap_pyfunction!(double, m)?).unwrap();
    m.add("also_double", wrap_pyfunction!(double, m)?).unwrap();

    Ok(())
}

#[test]
fn test_module_with_functions() {
    use pyo3::wrap_pymodule;

    Python::with_gil(|py| {
        let d = [(
            "module_with_functions",
            wrap_pymodule!(module_with_functions)(py),
        )]
        .into_py_dict_bound(py);

        py_assert!(
            py,
            *d,
            "module_with_functions.__doc__ == 'This module is implemented in Rust.'"
        );
        py_assert!(py, *d, "module_with_functions.no_parameters() == 42");
        py_assert!(py, *d, "module_with_functions.foo == 'bar'");
        py_assert!(py, *d, "module_with_functions.AnonClass != None");
        py_assert!(py, *d, "module_with_functions.LocatedClass != None");
        py_assert!(
            py,
            *d,
            "module_with_functions.LocatedClass.__module__ == 'module'"
        );
        py_assert!(py, *d, "module_with_functions.double(3) == 6");
        py_assert!(
            py,
            *d,
            "module_with_functions.double.__doc__ == 'Doubles the given value'"
        );
        py_assert!(py, *d, "module_with_functions.also_double(3) == 6");
        py_assert!(
            py,
            *d,
            "module_with_functions.also_double.__doc__ == 'Doubles the given value'"
        );
        py_assert!(
            py,
            *d,
            "module_with_functions.double_value(module_with_functions.ValueClass(1)) == 2"
        );
        py_assert!(
            py,
            *d,
            "module_with_functions.with_module() == 'module_with_functions'"
        );
    });
}

#[pymodule]
#[pyo3(name = "other_name")]
fn some_name(_: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add("other_name", "other_name")?;
    Ok(())
}

#[test]
fn test_module_renaming() {
    use pyo3::wrap_pymodule;

    Python::with_gil(|py| {
        let d = [("different_name", wrap_pymodule!(some_name)(py))].into_py_dict_bound(py);

        py_run!(py, *d, "assert different_name.__name__ == 'other_name'");
    });
}

#[test]
fn test_module_from_code_bound() {
    Python::with_gil(|py| {
        let adder_mod = PyModule::from_code_bound(
            py,
            "def add(a,b):\n\treturn a+b",
            "adder_mod.py",
            "adder_mod",
        )
        .expect("Module code should be loaded");

        let add_func = adder_mod
            .getattr("add")
            .expect("Add function should be in the module")
            .to_object(py);

        let ret_value: i32 = add_func
            .call1(py, (1, 2))
            .expect("A value should be returned")
            .extract(py)
            .expect("The value should be able to be converted to an i32");

        assert_eq!(ret_value, 3);
    });
}

#[pyfunction]
fn r#move() -> usize {
    42
}

#[pymodule]
fn raw_ident_module(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(r#move, module)?)
}

#[test]
fn test_raw_idents() {
    use pyo3::wrap_pymodule;

    Python::with_gil(|py| {
        let module = wrap_pymodule!(raw_ident_module)(py);

        py_assert!(py, module, "module.move() == 42");
    });
}

#[pyfunction]
#[pyo3(name = "foobar")]
fn custom_named_fn() -> usize {
    42
}

#[test]
fn test_custom_names() {
    #[pymodule]
    fn custom_names(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(custom_named_fn, m)?)?;
        Ok(())
    }

    Python::with_gil(|py| {
        let module = pyo3::wrap_pymodule!(custom_names)(py);

        py_assert!(py, module, "not hasattr(module, 'custom_named_fn')");
        py_assert!(py, module, "module.foobar() == 42");
    });
}

#[test]
fn test_module_dict() {
    #[pymodule]
    fn module_dict(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
        m.dict().set_item("yay", "me")?;
        Ok(())
    }

    Python::with_gil(|py| {
        let module = pyo3::wrap_pymodule!(module_dict)(py);

        py_assert!(py, module, "module.yay == 'me'");
    });
}

#[test]
fn test_module_dunder_all() {
    Python::with_gil(|py| {
        #[pymodule]
        fn dunder_all(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
            m.dict().set_item("yay", "me")?;
            m.add_function(wrap_pyfunction!(custom_named_fn, m)?)?;
            Ok(())
        }

        let module = pyo3::wrap_pymodule!(dunder_all)(py);

        py_assert!(py, module, "module.__all__ == ['foobar']");
    });
}

#[pyfunction]
fn subfunction() -> String {
    "Subfunction".to_string()
}

fn submodule(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(&wrap_pyfunction!(subfunction, module.as_gil_ref())?.as_borrowed())?;
    Ok(())
}

#[pymodule]
fn submodule_with_init_fn(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(subfunction, module)?)?;
    Ok(())
}

#[pyfunction]
fn superfunction() -> String {
    "Superfunction".to_string()
}

#[pymodule]
fn supermodule(py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(superfunction, module)?)?;
    let module_to_add = PyModule::new_bound(py, "submodule")?;
    submodule(&module_to_add)?;
    module.add_submodule(module_to_add.as_gil_ref())?;
    let module_to_add = PyModule::new_bound(py, "submodule_with_init_fn")?;
    submodule_with_init_fn(py, module_to_add.as_gil_ref())?;
    module.add_submodule(module_to_add.as_gil_ref())?;
    Ok(())
}

#[test]
fn test_module_nesting() {
    use pyo3::wrap_pymodule;

    Python::with_gil(|py| {
        let supermodule = wrap_pymodule!(supermodule)(py);

        py_assert!(
            py,
            supermodule,
            "supermodule.superfunction() == 'Superfunction'"
        );
        py_assert!(
            py,
            supermodule,
            "supermodule.submodule.subfunction() == 'Subfunction'"
        );
        py_assert!(
            py,
            supermodule,
            "supermodule.submodule_with_init_fn.subfunction() == 'Subfunction'"
        );
    });
}

// Test that argument parsing specification works for pyfunctions

#[pyfunction(signature = (a=5, *args))]
fn ext_vararg_fn(py: Python<'_>, a: i32, args: &PyTuple) -> PyObject {
    [a.to_object(py), args.into()].to_object(py)
}

#[pymodule]
fn vararg_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, signature = (a=5, *args))]
    fn int_vararg_fn(py: Python<'_>, a: i32, args: &PyTuple) -> PyObject {
        ext_vararg_fn(py, a, args)
    }

    m.add_function(wrap_pyfunction!(ext_vararg_fn, m)?).unwrap();
    Ok(())
}

#[test]
fn test_vararg_module() {
    Python::with_gil(|py| {
        let m = pyo3::wrap_pymodule!(vararg_module)(py);

        py_assert!(py, m, "m.ext_vararg_fn() == [5, ()]");
        py_assert!(py, m, "m.ext_vararg_fn(1, 2) == [1, (2,)]");

        py_assert!(py, m, "m.int_vararg_fn() == [5, ()]");
        py_assert!(py, m, "m.int_vararg_fn(1, 2) == [1, (2,)]");
    });
}

#[test]
fn test_module_with_constant() {
    // Regression test for #1102

    #[pymodule]
    fn module_with_constant(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
        const ANON: AnonClass = AnonClass {};

        m.add("ANON", ANON)?;
        m.add_class::<AnonClass>()?;

        Ok(())
    }

    Python::with_gil(|py| {
        let m = pyo3::wrap_pymodule!(module_with_constant)(py);
        py_assert!(py, m, "isinstance(m.ANON, m.AnonClass)");
    });
}

#[pyfunction]
#[pyo3(pass_module)]
fn pyfunction_with_module<'py>(module: &Bound<'py, PyModule>) -> PyResult<Bound<'py, PyString>> {
    module.name()
}

#[pyfunction]
#[pyo3(pass_module)]
fn pyfunction_with_module_gil_ref(module: &PyModule) -> PyResult<&str> {
    module.name()
}

#[pyfunction]
#[pyo3(pass_module)]
fn pyfunction_with_module_owned(
    module: Py<PyModule>,
    py: Python<'_>,
) -> PyResult<Bound<'_, PyString>> {
    module.bind(py).name()
}

#[pyfunction]
#[pyo3(pass_module)]
fn pyfunction_with_module_and_py<'py>(
    module: &Bound<'py, PyModule>,
    _python: Python<'py>,
) -> PyResult<Bound<'py, PyString>> {
    module.name()
}

#[pyfunction]
#[pyo3(pass_module)]
fn pyfunction_with_module_and_arg<'py>(
    module: &Bound<'py, PyModule>,
    string: String,
) -> PyResult<(Bound<'py, PyString>, String)> {
    module.name().map(|s| (s, string))
}

#[pyfunction(signature = (string="foo"))]
#[pyo3(pass_module)]
fn pyfunction_with_module_and_default_arg<'py>(
    module: &Bound<'py, PyModule>,
    string: &str,
) -> PyResult<(Bound<'py, PyString>, String)> {
    module.name().map(|s| (s, string.into()))
}

#[pyfunction(signature = (*args, **kwargs))]
#[pyo3(pass_module)]
fn pyfunction_with_module_and_args_kwargs<'py>(
    module: &Bound<'py, PyModule>,
    args: &Bound<'py, PyTuple>,
    kwargs: Option<&Bound<'py, PyDict>>,
) -> PyResult<(Bound<'py, PyString>, usize, Option<usize>)> {
    module
        .name()
        .map(|s| (s, args.len(), kwargs.map(|d| d.len())))
}

#[pyfunction]
#[pyo3(pass_module)]
fn pyfunction_with_pass_module_in_attribute(module: &PyModule) -> PyResult<&str> {
    module.name()
}

#[pymodule]
fn module_with_functions_with_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pyfunction_with_module, m)?)?;
    m.add_function(wrap_pyfunction!(pyfunction_with_module_gil_ref, m)?)?;
    m.add_function(wrap_pyfunction!(pyfunction_with_module_owned, m)?)?;
    m.add_function(wrap_pyfunction!(pyfunction_with_module_and_py, m)?)?;
    m.add_function(wrap_pyfunction!(pyfunction_with_module_and_arg, m)?)?;
    m.add_function(wrap_pyfunction!(pyfunction_with_module_and_default_arg, m)?)?;
    m.add_function(wrap_pyfunction!(pyfunction_with_module_and_args_kwargs, m)?)?;
    m.add_function(wrap_pyfunction!(
        pyfunction_with_pass_module_in_attribute,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(pyfunction_with_module, m)?)?;
    Ok(())
}

#[test]
fn test_module_functions_with_module() {
    Python::with_gil(|py| {
        let m = pyo3::wrap_pymodule!(module_with_functions_with_module)(py);
        py_assert!(
            py,
            m,
            "m.pyfunction_with_module() == 'module_with_functions_with_module'"
        );
        py_assert!(
            py,
            m,
            "m.pyfunction_with_module_gil_ref() == 'module_with_functions_with_module'"
        );
        py_assert!(
            py,
            m,
            "m.pyfunction_with_module_owned() == 'module_with_functions_with_module'"
        );
        py_assert!(
            py,
            m,
            "m.pyfunction_with_module_and_py() == 'module_with_functions_with_module'"
        );
        py_assert!(
            py,
            m,
            "m.pyfunction_with_module_and_default_arg() \
                        == ('module_with_functions_with_module', 'foo')"
        );
        py_assert!(
            py,
            m,
            "m.pyfunction_with_module_and_args_kwargs(1, x=1, y=2) \
                        == ('module_with_functions_with_module', 1, 2)"
        );
        py_assert!(
            py,
            m,
            "m.pyfunction_with_pass_module_in_attribute() == 'module_with_functions_with_module'"
        );
    });
}

#[test]
fn test_module_doc_hidden() {
    #[doc(hidden)]
    #[allow(clippy::unnecessary_wraps)]
    #[pymodule]
    fn my_module(_py: Python<'_>, _m: &PyModule) -> PyResult<()> {
        Ok(())
    }

    Python::with_gil(|py| {
        let m = pyo3::wrap_pymodule!(my_module)(py);
        py_assert!(py, m, "m.__doc__ == ''");
    })
}
