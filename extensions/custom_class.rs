#![crate_type = "dylib"]

#[macro_use] extern crate cpython;

use cpython::{Python, PyObject, PyResult, PyType};

py_module_initializer!(custom_class, initcustom_class, PyInit_custom_class, |py, m| {
    try!(m.add(py, "__doc__", "Module documentation string"));
    try!(m.add_class::<MyType>(py));
    Ok(())
});

py_class!(class MyType, data: i32, |py| {
    def __new__(_cls: &PyType, arg: i32) -> PyResult<MyType> {
        Ok(MyType::create_instance(py, arg))
    }
    def a(&self) -> PyResult<PyObject> {{
        println!("a() was called with self={:?}", self.data(py));
        Ok(py.None())
    }}
});

