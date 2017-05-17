// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Number Interface
//! Trait and support implementation for implementing number protocol

use std::os::raw::c_int;

use ffi;
use err::PyResult;
use python::{Python, PythonObject};
use objects::PyObject;
use callback::{BoolConverter, PyObjectCallbackConverter};
use class::{NO_METHODS, NO_PY_METHODS};
use class::basic::{PyObjectProtocol, PyObjectProtocolImpl};

/// Number interface
pub trait PyNumberProtocol {

    fn __add__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __sub__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __mul__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __matmul__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __truediv__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __floordiv__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __mod__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __divmod__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __pow__(&self, py: Python, other: &PyObject, modulo: &PyObject) -> PyResult<PyObject>;
    fn __lshift__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __rshift__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __and__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __xor__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __or__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;

    fn __radd__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __rsub__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __rmul__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __rmatmul__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __rtruediv__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __rfloordiv__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __rmod__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __rdivmod__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __rpow__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __rlshift__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __rrshift__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __rand__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __rxor__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __ror__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;

    fn __iadd__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __isub__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __imul__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __imatmul__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __itruediv__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __ifloordiv__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __imod__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __ipow__(&self, py: Python, other: &PyObject, modulo: &PyObject) -> PyResult<PyObject>;
    fn __ilshift__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __irshift__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __iand__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __ixor__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;
    fn __ior__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;

    // Unary arithmetic
    fn __neg__(&self, py: Python) -> PyResult<PyObject>;
    fn __pos__(&self, py: Python) -> PyResult<PyObject>;
    fn __abs__(&self, py: Python) -> PyResult<PyObject>;
    fn __invert__(&self, py: Python) -> PyResult<PyObject>;
    fn __complex__(&self, py: Python) -> PyResult<PyObject>;
    fn __int__(&self, py: Python) -> PyResult<PyObject>;
    fn __float__(&self, py: Python) -> PyResult<PyObject>;
    fn __round__(&self, py: Python) -> PyResult<PyObject>;
    fn __index__(&self, py: Python) -> PyResult<PyObject>;
}

impl<T> PyNumberProtocol for T where T: PythonObject {
    default fn __add__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __sub__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __mul__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __matmul__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __truediv__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __floordiv__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __mod__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __divmod__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __pow__(&self, py: Python, _: &PyObject, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __lshift__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __rshift__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __and__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __xor__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __or__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }

    default fn __radd__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __rsub__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __rmul__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __rmatmul__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __rtruediv__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __rfloordiv__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __rmod__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __rdivmod__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __rpow__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __rlshift__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __rrshift__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __rand__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __rxor__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __ror__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }

    default fn __iadd__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __isub__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __imul__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __imatmul__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __itruediv__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __ifloordiv__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __imod__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __ipow__(&self, py: Python, _: &PyObject, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __ilshift__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __irshift__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __iand__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __ixor__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __ior__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __neg__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __pos__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __abs__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __invert__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __complex__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __int__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __float__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __round__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
    default fn __index__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.NotImplemented())
    }
}

#[doc(hidden)]
pub trait PyNumberProtocolImpl {
    fn methods() -> &'static [&'static str];
    fn py_methods() -> &'static [::class::PyMethodDefType];
}

impl<T> PyNumberProtocolImpl for T {
    default fn methods() -> &'static [&'static str] {
        NO_METHODS
    }
    default fn py_methods() -> &'static [::class::PyMethodDefType] {
        NO_PY_METHODS
    }
}

impl ffi::PyNumberMethods {

    /// Construct PyNumberMethods struct for PyTypeObject.tp_as_number
    pub fn new<T>() -> Option<ffi::PyNumberMethods>
        where T: PyNumberProtocol + PyNumberProtocolImpl
                 + PyObjectProtocol + PyObjectProtocolImpl
                 + PythonObject

    {
        let objm = <T as PyObjectProtocolImpl>::methods();
        let methods = <T as PyNumberProtocolImpl>::methods();
        if methods.is_empty() && ! objm.contains(&"__bool__") {
            return None
        }

        let mut meth: ffi::PyNumberMethods = ffi::PyNumberMethods_INIT;

        for name in methods {
            match name {
                &"__add__" => {
                    meth.nb_add = py_binary_func!(
                        PyNumberProtocol, T::__add__, PyObjectCallbackConverter);
                },
                &"__sub__" => {
                    meth.nb_subtract = py_binary_func!(
                        PyNumberProtocol, T::__sub__, PyObjectCallbackConverter);
                },
                &"__mul__" => {
                    meth.nb_multiply = py_binary_func!(
                        PyNumberProtocol, T::__mul__, PyObjectCallbackConverter);
                },
                &"__matmul__" => {
                    meth.nb_matrix_multiply = py_binary_func!(
                        PyNumberProtocol, T::__matmul__, PyObjectCallbackConverter);
                },
                &"__truediv__" => {
                    meth.nb_true_divide = py_binary_func!(
                        PyNumberProtocol, T::__truediv__, PyObjectCallbackConverter);
                },
                &"__floordiv__" => {
                    meth.nb_floor_divide = py_binary_func!(
                        PyNumberProtocol, T::__floordiv__, PyObjectCallbackConverter);
                },
                &"__mod__" => {
                    meth.nb_remainder = py_binary_func!(
                        PyNumberProtocol, T::__mod__, PyObjectCallbackConverter);
                },
                &"__divmod__" => {
                    meth.nb_divmod = py_binary_func!(
                        PyNumberProtocol, T::__divmod__, PyObjectCallbackConverter);
                },
                &"__pow__" => {
                    meth.nb_power = py_ternary_func!(
                        PyNumberProtocol, T::__pow__, PyObjectCallbackConverter);
                },
                &"__lshift__" => {
                    meth.nb_lshift = py_binary_func!(
                        PyNumberProtocol, T::__lshift__, PyObjectCallbackConverter);
                },
                &"__rshift__" => {
                    meth.nb_rshift = py_binary_func!(
                        PyNumberProtocol, T::__rshift__, PyObjectCallbackConverter);
                },
                &"__and__" => {
                    meth.nb_and = py_binary_func!(
                        PyNumberProtocol, T::__and__, PyObjectCallbackConverter);
                },
                &"__xor__" => {
                    meth.nb_xor = py_binary_func!(
                        PyNumberProtocol, T::__xor__, PyObjectCallbackConverter);
                },
                &"__or__" => {
                    meth.nb_or = py_binary_func!(
                        PyNumberProtocol, T::__or__, PyObjectCallbackConverter);
                },
                &"__iadd__" => {
                    meth.nb_inplace_add = py_binary_func!(
                        PyNumberProtocol, T::__iadd__, PyObjectCallbackConverter);
                },
                &"__isub__" => {
                    meth.nb_inplace_subtract = py_binary_func!(
                        PyNumberProtocol, T::__isub__, PyObjectCallbackConverter);
                },
                &"__imul__" => {
                    meth.nb_inplace_multiply = py_binary_func!(
                        PyNumberProtocol, T::__imul__, PyObjectCallbackConverter);
                },
                &"__imatmul__" => {
                    meth.nb_inplace_matrix_multiply = py_binary_func!(
                        PyNumberProtocol, T::__imatmul__, PyObjectCallbackConverter);
                },
                &"__itruediv__" => {
                    meth.nb_inplace_true_divide = py_binary_func!(
                        PyNumberProtocol, T::__itruediv__, PyObjectCallbackConverter);
                },
                &"__ifloordiv__" => {
                    meth.nb_inplace_floor_divide = py_binary_func!(
                        PyNumberProtocol, T::__ifloordiv__, PyObjectCallbackConverter);
                },
                &"__imod__" => {
                    meth.nb_inplace_remainder = py_binary_func!(
                        PyNumberProtocol, T::__imod__, PyObjectCallbackConverter);
                },
                &"__ipow__" => {
                    meth.nb_inplace_power = py_ternary_func!(
                        PyNumberProtocol, T::__ipow__, PyObjectCallbackConverter);
                },
                &"__ilshift__" => {
                    meth.nb_inplace_lshift = py_binary_func!(
                        PyNumberProtocol, T::__ilshift__, PyObjectCallbackConverter);
                },
                &"__irshift__" => {
                    meth.nb_inplace_rshift = py_binary_func!(
                        PyNumberProtocol, T::__irshift__, PyObjectCallbackConverter);
                },
                &"__iand__" => {
                    meth.nb_inplace_and = py_binary_func!(
                        PyNumberProtocol, T::__iand__, PyObjectCallbackConverter);
                },
                &"__ixor__" => {
                    meth.nb_inplace_xor = py_binary_func!(
                        PyNumberProtocol, T::__ixor__, PyObjectCallbackConverter);
                },
                &"__ior__" => {
                    meth.nb_inplace_or = py_binary_func!(
                        PyNumberProtocol, T::__ior__, PyObjectCallbackConverter);
                },
                &"__neg__" => {
                    meth.nb_negative = py_unary_func!(
                        PyNumberProtocol, T::__neg__, PyObjectCallbackConverter);
                },
                &"__pos__" => {
                    meth.nb_positive = py_unary_func!(
                        PyNumberProtocol, T::__pos__, PyObjectCallbackConverter);

                },
                &"__abs__" => {
                    meth.nb_absolute = py_unary_func!(
                        PyNumberProtocol, T::__abs__, PyObjectCallbackConverter);

                },
                &"__invert__" => {
                    meth.nb_invert = py_unary_func!(
                        PyNumberProtocol, T::__invert__, PyObjectCallbackConverter);

                },
                &"__int__" => {
                    meth.nb_int = py_unary_func!(
                        PyNumberProtocol, T::__int__, PyObjectCallbackConverter);

                },
                &"__float__" => {
                    meth.nb_float = py_unary_func!(
                        PyNumberProtocol, T::__float__, PyObjectCallbackConverter);

                },
                &"__index__" => {
                    meth.nb_index = py_unary_func!(
                        PyNumberProtocol, T::__index__, PyObjectCallbackConverter);
                },
                _ => (),
            }
        }

        if objm.contains(&"__bool__") {
            meth.nb_bool = py_unary_func!(
                PyObjectProtocol, T::__bool__, BoolConverter, c_int);
        }

        Some(meth)
    }
}
