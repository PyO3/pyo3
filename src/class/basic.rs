// Copyright (c) 2017-present PyO3 Project and Contributors

//! Basic Python Object customization
//!
//! more information on python async support
//! https://docs.python.org/3/reference/datamodel.html#basic-customization

use std;
use std::os::raw::c_int;

use ::{Py, CompareOp};
use ffi;
use err::{PyErr, PyResult};
use python::{Python, IntoPyPointer};
use objects::PyObject;
use objects::exc;
use typeob::PyTypeInfo;
use conversion::{FromPyObject, IntoPyObject};
use callback::{PyObjectCallbackConverter, HashConverter, BoolCallbackConverter};
use class::methods::PyMethodDef;

// classmethod
// staticmethod
// __instancecheck__
// __subclasscheck__


/// Object customization
#[allow(unused_variables)]
pub trait PyObjectProtocol<'p>: PyTypeInfo + Sized + 'static {

    fn __getattr__(&'p self, py: Python<'p>, name: Self::Name)
                   -> Self::Result where Self: PyObjectGetAttrProtocol<'p> {unimplemented!()}

    fn __setattr__(&'p mut self, py: Python<'p>, name: Self::Name, value: Self::Value)
                   -> Self::Result where Self: PyObjectSetAttrProtocol<'p> {unimplemented!()}

    fn __delattr__(&'p mut self, py: Python<'p>, name: Self::Name)
                   -> Self::Result where Self: PyObjectDelAttrProtocol<'p> {unimplemented!()}

    fn __str__(&'p self, py: Python<'p>)
               -> Self::Result where Self: PyObjectStrProtocol<'p> {unimplemented!()}

    fn __repr__(&'p self, py: Python<'p>)
                -> Self::Result where Self: PyObjectReprProtocol<'p> {unimplemented!()}

    fn __format__(&'p self, py: Python<'p>, format_spec: Self::Format)
                  -> Self::Result where Self: PyObjectFormatProtocol<'p> {unimplemented!()}

    fn __hash__(&'p self, py: Python<'p>)
                -> Self::Result where Self: PyObjectHashProtocol<'p> {unimplemented!()}

    fn __bool__(&'p self, py: Python<'p>)
                -> Self::Result where Self: PyObjectBoolProtocol<'p> {unimplemented!()}

    fn __bytes__(&'p self, py: Python<'p>)
                 -> Self::Result where Self: PyObjectBytesProtocol<'p> {unimplemented!()}

    fn __richcmp__(&'p self, py: Python<'p>, other: Self::Other, op: CompareOp)
                   -> Self::Result where Self: PyObjectRichcmpProtocol<'p> {unimplemented!()}
}


pub trait PyObjectGetAttrProtocol<'p>: PyObjectProtocol<'p> {
    type Name: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectSetAttrProtocol<'p>: PyObjectProtocol<'p> {
    type Name: FromPyObject<'p>;
    type Value: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyObjectDelAttrProtocol<'p>: PyObjectProtocol<'p> {
    type Name: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyObjectStrProtocol<'p>: PyObjectProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectReprProtocol<'p>: PyObjectProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectFormatProtocol<'p>: PyObjectProtocol<'p> {
    type Format: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectHashProtocol<'p>: PyObjectProtocol<'p> {
    type Result: Into<PyResult<usize>>;
}
pub trait PyObjectBoolProtocol<'p>: PyObjectProtocol<'p> {
    type Result: Into<PyResult<bool>>;
}
pub trait PyObjectBytesProtocol<'p>: PyObjectProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectRichcmpProtocol<'p>: PyObjectProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}


#[doc(hidden)]
pub trait PyObjectProtocolImpl {
    fn methods() -> Vec<PyMethodDef>;
    fn tp_as_object(type_object: &mut ffi::PyTypeObject);
    fn nb_bool_fn() -> Option<ffi::inquiry>;
}

impl<T> PyObjectProtocolImpl for T {
    default fn methods() -> Vec<PyMethodDef> {
        Vec::new()
    }
    default fn tp_as_object(_type_object: &mut ffi::PyTypeObject) {
    }
    default fn nb_bool_fn() -> Option<ffi::inquiry> {
        None
    }
}

impl<'p, T> PyObjectProtocolImpl for T where T: PyObjectProtocol<'p> {
    #[inline]
    fn methods() -> Vec<PyMethodDef> {
        let mut methods = Vec::new();

        if let Some(def) = <Self as PyObjectFormatProtocolImpl>::__format__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyObjectBytesProtocolImpl>::__bytes__() {
            methods.push(def)
        }
        methods
    }
    fn tp_as_object(type_object: &mut ffi::PyTypeObject) {
        type_object.tp_str = Self::tp_str();
        type_object.tp_repr = Self::tp_repr();
        type_object.tp_hash = Self::tp_hash();
        type_object.tp_getattro = Self::tp_getattro();
        type_object.tp_richcompare = Self::tp_richcompare();

        type_object.tp_setattro = if let Some(df) = Self::tp_delattro() {
            Some(df)
        } else {
            Self::tp_setattro()
        };
    }
    fn nb_bool_fn() -> Option<ffi::inquiry> {
        Self::nb_bool()
    }
}

trait PyObjectGetAttrProtocolImpl {
    fn tp_getattro() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyObjectGetAttrProtocolImpl for T where T: PyObjectProtocol<'p>
{
    #[inline]
    default fn tp_getattro() -> Option<ffi::binaryfunc> {
        None
    }
}
impl<T> PyObjectGetAttrProtocolImpl for T where T: for<'p> PyObjectGetAttrProtocol<'p>
{
    #[inline]
    fn tp_getattro() -> Option<ffi::binaryfunc> {
        py_binary_func!(PyObjectGetAttrProtocol,
                        T::__getattr__, T::Success, PyObjectCallbackConverter)
    }
}


trait PyObjectSetAttrProtocolImpl {
    fn tp_setattro() -> Option<ffi::setattrofunc>;
}

impl<'p, T> PyObjectSetAttrProtocolImpl for T where T: PyObjectProtocol<'p>
{
    #[inline]
    default fn tp_setattro() -> Option<ffi::setattrofunc> {
        None
    }
}
impl<T> PyObjectSetAttrProtocolImpl for T where T: for<'p> PyObjectSetAttrProtocol<'p>
{
    #[inline]
    fn tp_setattro() -> Option<ffi::setattrofunc> {
        py_func_set!(PyObjectSetAttrProtocol, T::__setattr__)
    }
}


trait PyObjectDelAttrProtocolImpl {
    fn tp_delattro() -> Option<ffi::setattrofunc>;
}
impl<'p, T> PyObjectDelAttrProtocolImpl for T where T: PyObjectProtocol<'p>
{
    #[inline]
    default fn tp_delattro() -> Option<ffi::setattrofunc> {
        None
    }
}
impl<T> PyObjectDelAttrProtocolImpl for T where T: for<'p> PyObjectDelAttrProtocol<'p>
{
    #[inline]
    default fn tp_delattro() -> Option<ffi::setattrofunc> {
        py_func_del!(PyObjectDelAttrProtocol, T::__delattr__)
    }
}
impl<T> PyObjectDelAttrProtocolImpl for T
    where T: for<'p> PyObjectSetAttrProtocol<'p> + for<'p> PyObjectDelAttrProtocol<'p>
{
    #[inline]
    fn tp_delattro() -> Option<ffi::setattrofunc> {
        py_func_set_del!(PyObjectSetAttrProtocol, PyObjectDelAttrProtocol,
                         T::__setattr__/__delattr__)
    }
}


trait PyObjectStrProtocolImpl {
    fn tp_str() -> Option<ffi::unaryfunc>;
}
impl<'p, T> PyObjectStrProtocolImpl for T where T: PyObjectProtocol<'p>
{
    #[inline]
    default fn tp_str() -> Option<ffi::unaryfunc> {
        None
    }
}
impl<T> PyObjectStrProtocolImpl for T where T: for<'p> PyObjectStrProtocol<'p>
{
    #[inline]
    fn tp_str() -> Option<ffi::unaryfunc> {
        py_unary_func!(PyObjectStrProtocol, T::__str__,
                       <T as PyObjectStrProtocol>::Success, PyObjectCallbackConverter)
    }
}

trait PyObjectReprProtocolImpl {
    fn tp_repr() -> Option<ffi::unaryfunc>;
}
impl<'p, T> PyObjectReprProtocolImpl for T where T: PyObjectProtocol<'p>
{
    #[inline]
    default fn tp_repr() -> Option<ffi::unaryfunc> {
        None
    }
}
impl<T> PyObjectReprProtocolImpl for T where T: for<'p> PyObjectReprProtocol<'p>
{
    #[inline]
    fn tp_repr() -> Option<ffi::unaryfunc> {
        py_unary_func!(PyObjectReprProtocol,
                       T::__repr__, T::Success, PyObjectCallbackConverter)
    }
}

#[doc(hidden)]
pub trait PyObjectFormatProtocolImpl {
    fn __format__() -> Option<PyMethodDef>;
}
impl<'p, T> PyObjectFormatProtocolImpl for T where T: PyObjectProtocol<'p>
{
    #[inline]
    default fn __format__() -> Option<PyMethodDef> {
        None
    }
}

#[doc(hidden)]
pub trait PyObjectBytesProtocolImpl {
    fn __bytes__() -> Option<PyMethodDef>;
}
impl<'p, T> PyObjectBytesProtocolImpl for T where T: PyObjectProtocol<'p>
{
    #[inline]
    default fn __bytes__() -> Option<PyMethodDef> {
        None
    }
}


trait PyObjectHashProtocolImpl {
    fn tp_hash() -> Option<ffi::hashfunc>;
}
impl<'p, T> PyObjectHashProtocolImpl for T where T: PyObjectProtocol<'p>
{
    #[inline]
    default fn tp_hash() -> Option<ffi::hashfunc> {
        None
    }
}
impl<T> PyObjectHashProtocolImpl for T where T: for<'p> PyObjectHashProtocol<'p>
{
    #[inline]
    fn tp_hash() -> Option<ffi::hashfunc> {
        py_unary_func!(PyObjectHashProtocol,
                       T::__hash__, usize, HashConverter, ffi::Py_hash_t)
    }
}

trait PyObjectBoolProtocolImpl {
    fn nb_bool() -> Option<ffi::inquiry>;
}
impl<'p, T> PyObjectBoolProtocolImpl for T where T: PyObjectProtocol<'p>
{
    #[inline]
    default fn nb_bool() -> Option<ffi::inquiry> {
        None
    }
}
impl<T> PyObjectBoolProtocolImpl for T where T: for<'p> PyObjectBoolProtocol<'p>
{
    #[inline]
    fn nb_bool() -> Option<ffi::inquiry> {
        py_unary_func!(PyObjectBoolProtocol, T::__bool__, bool, BoolCallbackConverter, c_int)
    }
}

trait PyObjectRichcmpProtocolImpl {
    fn tp_richcompare() -> Option<ffi::richcmpfunc>;
}
impl<'p, T> PyObjectRichcmpProtocolImpl for T where T: PyObjectProtocol<'p>
{
    #[inline]
    default fn tp_richcompare() -> Option<ffi::richcmpfunc> {
        None
    }
}
impl<T> PyObjectRichcmpProtocolImpl for T where T: for<'p> PyObjectRichcmpProtocol<'p>
{
    #[inline]
    fn tp_richcompare() -> Option<ffi::richcmpfunc> {
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                     arg: *mut ffi::PyObject,
                                     op: c_int) -> *mut ffi::PyObject
            where T: for<'p> PyObjectRichcmpProtocol<'p>
        {
            const LOCATION: &'static str = concat!(stringify!(T), ".__richcmp__()");

            let guard = ::callback::AbortOnDrop(LOCATION);
            let ret = std::panic::catch_unwind(|| {
                let py = Python::assume_gil_acquired();
                let slf = Py::<T>::from_borrowed_ptr(py, slf);
                let arg = PyObject::from_borrowed_ptr(py, arg);

                let res = match extract_op(py, op) {
                    Ok(op) => {
                        match arg.extract(py) {
                            Ok(arg) => {
                                slf.__richcmp__(py, arg, op).into()
                            }
                            Err(e) => Err(e.into()),
                        }
                    },
                    Err(e) => Err(e)
                };
                match res {
                    Ok(val) => {
                        val.into_object(py).into_ptr()
                    }
                    Err(e) => {
                        e.restore(py);
                        std::ptr::null_mut()
                    }
                }
            });

            let ret = match ret {
                Ok(r) => r,
                Err(ref err) => {
                    ::callback::handle_panic(Python::assume_gil_acquired(), err);
                    std::ptr::null_mut()
                }
            };
            std::mem::forget(guard);
            ret
        }
        Some(wrap::<T>)
    }
}


fn extract_op(py: Python, op: c_int) -> PyResult<CompareOp> {
    match op {
        ffi::Py_LT => Ok(CompareOp::Lt),
        ffi::Py_LE => Ok(CompareOp::Le),
        ffi::Py_EQ => Ok(CompareOp::Eq),
        ffi::Py_NE => Ok(CompareOp::Ne),
        ffi::Py_GT => Ok(CompareOp::Gt),
        ffi::Py_GE => Ok(CompareOp::Ge),
        _ => Err(PyErr::new_lazy_init(
            py.get_type::<exc::ValueError>(),
            Some("tp_richcompare called with invalid comparison operator".into_object(py))))
    }
}
