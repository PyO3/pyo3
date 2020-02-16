use pyo3::prelude::*;
use pyo3::py_run;

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
struct ClassWithDocs {
    /// Property field
    #[pyo3(get, set)]
    value: i32,

    /// Read-only property field
    #[pyo3(get)]
    readonly: i32,

    /// Write-only property field
    #[pyo3(set)]
    writeonly: i32,
}

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
        py_run!(
            py,
            typeobj,
            "assert typeobj.value.__doc__ == 'Property field'"
        );
        py_run!(
            py,
            typeobj,
            "assert typeobj.readonly.__doc__ == 'Read-only property field'"
        );
        py_run!(
            py,
            typeobj,
            "assert typeobj.writeonly.__doc__ == 'Write-only property field'"
        );
    }
}

#[pyclass(name=CustomName)]
struct EmptyClass2 {}

#[pymethods]
impl EmptyClass2 {
    #[name = "custom_fn"]
    fn bar(&self) {}

    #[staticmethod]
    #[name = "custom_static"]
    fn bar_static() {}
}

#[test]
fn custom_names() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<EmptyClass2>();
    py_assert!(py, typeobj, "typeobj.__name__ == 'CustomName'");
    py_assert!(py, typeobj, "typeobj.custom_fn.__name__ == 'custom_fn'");
    py_assert!(
        py,
        typeobj,
        "typeobj.custom_static.__name__ == 'custom_static'"
    );
    py_assert!(py, typeobj, "not hasattr(typeobj, 'bar')");
    py_assert!(py, typeobj, "not hasattr(typeobj, 'bar_static')");
}

#[pyclass]
struct RawIdents {
    #[pyo3(get, set)]
    r#type: i64,
}

#[pymethods]
impl RawIdents {
    fn r#fn(&self) {}
}

#[test]
fn test_raw_idents() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<RawIdents>();
    py_assert!(py, typeobj, "not hasattr(typeobj, 'r#fn')");
    py_assert!(py, typeobj, "hasattr(typeobj, 'fn')");
    py_assert!(py, typeobj, "hasattr(typeobj, 'type')");
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
}

#[pyclass]
struct ClassWithObjectField {
    // PyObject has special case for derive_utils::GetPropertyValue,
    // so this test is making sure it compiles!
    #[pyo3(get, set)]
    value: PyObject,
}

#[pymethods]
impl ClassWithObjectField {
    #[new]
    fn new(value: PyObject) -> ClassWithObjectField {
        Self { value }
    }
}

#[test]
fn class_with_object_field() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let ty = py.get_type::<ClassWithObjectField>();
    py_assert!(py, ty, "ty(5).value == 5");
    py_assert!(py, ty, "ty(None).value == None");
}
