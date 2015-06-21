use std::ptr;
use std::cell::RefCell;
use python::Python;
use pythonrun::{GILProtected};
use objects::{PyObject, PyType};
use ffi;
use super::PyRustType;
use super::typebuilder::PyRustTypeBuilder;

/*
struct MethodDescriptor<'p> {
    ty: PyType<'p>,
    name: PyObject<'p>
    // d_method
}

static METHOD_DESCRIPTOR: GILProtected<RefCell<Option<SendablePyObject>>> = GILProtected::new(RefCell::new(None));

fn get_method_descriptor_type<'p>(py: Python<'p>) -> PyRustType<'p, MethodDescriptor<'p>> {
    METHOD_DESCRIPTOR.get(py);
}

*/

