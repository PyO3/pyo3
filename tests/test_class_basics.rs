#![feature(specialization)]

extern crate pyo3;

use pyo3::prelude::*;

#[macro_use]
mod common;

#[pyclass]
struct EmptyClass {}

#[test]
fn empty_class() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<EmptyClass>();
    // By default, don't allow creating instances from python.
    assert!(typeobj.call(NoArgs, None).is_err());

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
    assert_eq!(
        ty.getattr("__module__")
            .unwrap()
            .extract::<String>()
            .unwrap(),
        "test_module.nested"
    );
}

#[pyclass(variants("SimpleGenericU32<u32>", "SimpleGenericF32<f32>"))]
struct SimpleGeneric<T: 'static> {
    foo: T,
}

#[test]
fn generic_names() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let ty_u32 = py.get_type::<SimpleGeneric<u32>>();
    py_assert!(py, ty_u32, "ty_u32.__name__ == 'SimpleGenericU32'");

    let ty_f32 = py.get_type::<SimpleGeneric<f32>>();
    py_assert!(py, ty_f32, "ty_f32.__name__ == 'SimpleGenericF32'");
}

#[test]
fn generic_type_eq() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let tup = (
        (SimpleGeneric { foo: 1u32 }).into_object(py),
        (SimpleGeneric { foo: 1u32 }).into_object(py),
        (SimpleGeneric { foo: 1f32 }).into_object(py),
        (SimpleGeneric { foo: 1f32 }).into_object(py),
    );

    py_assert!(py, tup, "type(tup[0]) == type(tup[1])");
    py_assert!(py, tup, "type(tup[2]) == type(tup[3])");
    py_assert!(py, tup, "type(tup[0]) != type(tup[2])");
}