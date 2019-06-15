use pyo3::prelude::*;
use pyo3::py_run;
use pyo3::type_object::initialize_type;

mod common;

#[pyclass]
struct EmptyClass {}

#[test]
fn empty_class() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<EmptyClass>();
    // By default, don't allow creating instances from python.
    assert!(typeobj.call((), None).is_err());

    py_assert!(py, typeobj, "typeobj.__name__ == 'EmptyClass'");
}

/// Line1
///Line2
///  Line3
// this is not doc string
#[pyclass]
struct ClassWithDocs {}

#[test]
fn class_with_docstr() {
    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let typeobj = py.get_type::<ClassWithDocs>();
        py_run!(
            py,
            typeobj,
            "assert typeobj.__doc__ == 'Line1\\nLine2\\n Line3'"
        );
    }
}

#[pyclass(name=CustomName)]
struct EmptyClass2 {}

#[test]
fn custom_class_name() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<EmptyClass2>();
    py_assert!(py, typeobj, "typeobj.__name__ == 'CustomName'");
}

#[pyclass]
struct EmptyClassInModule {}

#[test]
fn empty_class_in_module() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let module = PyModule::new(py, "test_module.nested").unwrap();
    module.add_class::<EmptyClassInModule>().unwrap();

    let ty = module.getattr("EmptyClassInModule").unwrap();
    assert_eq!(
        ty.getattr("__name__").unwrap().extract::<String>().unwrap(),
        "EmptyClassInModule"
    );

    let module: String = ty.getattr("__module__").unwrap().extract().unwrap();

    // Rationale: The class can be added to many modules, but will only be initialized once.
    // We currently have no way of determining a canonical module, so builtins is better
    // than using whatever calls init first.
    assert_eq!(module, "builtins");

    // The module name can also be set manually by calling `initialize_type`.
    initialize_type::<EmptyClassInModule>(py, Some("test_module.nested")).unwrap();
    let module: String = ty.getattr("__module__").unwrap().extract().unwrap();
    assert_eq!(module, "test_module.nested");
}
