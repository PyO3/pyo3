use crate::object::*;
use crate::rustpython_runtime;
use rustpython_vm::builtins::PyComplex;
use rustpython_vm::AsObject;
use std::ffi::{c_double, c_int};

pub static mut PyComplex_Type: PyTypeObject = PyTypeObject { _opaque: [] };

#[cfg(PyRustPython)]
opaque_struct!(pub PyComplexObject);

#[repr(C)]
pub struct Py_complex {
    pub real: c_double,
    pub imag: c_double,
}

#[inline]
pub unsafe fn PyComplex_Check(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    ptr_to_pyobject_ref_borrowed(op)
        .downcast_ref::<PyComplex>()
        .is_some()
        .into()
}

#[inline]
pub unsafe fn PyComplex_CheckExact(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(op);
    rustpython_runtime::with_vm(|vm| {
        obj.downcast_ref::<PyComplex>()
            .is_some_and(|_| obj.class().is(vm.ctx.types.complex_type))
            .into()
    })
}

#[inline]
pub unsafe fn PyComplex_FromDoubles(real: c_double, imag: c_double) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        match vm.ctx.types.complex_type.as_object().call((real, imag), vm) {
            Ok(complex) => pyobject_ref_to_ptr(complex),
            Err(_) => std::ptr::null_mut(),
        }
    })
}

#[inline]
pub unsafe fn PyComplex_AsCComplex(op: *mut PyObject) -> Py_complex {
    Py_complex {
        real: unsafe { PyComplex_RealAsDouble(op) },
        imag: unsafe { PyComplex_ImagAsDouble(op) },
    }
}

#[inline]
pub unsafe fn PyComplex_RealAsDouble(op: *mut PyObject) -> c_double {
    if op.is_null() {
        return -1.0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(op);
    rustpython_runtime::with_vm(|vm| match obj.try_complex(vm) {
        Ok(Some((value, _))) => value.re,
        _ => -1.0,
    })
}

#[inline]
pub unsafe fn PyComplex_ImagAsDouble(op: *mut PyObject) -> c_double {
    if op.is_null() {
        return -1.0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(op);
    rustpython_runtime::with_vm(|vm| match obj.try_complex(vm) {
        Ok(Some((value, _))) => value.im,
        _ => -1.0,
    })
}
