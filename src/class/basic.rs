// Copyright (c) 2017-present PyO3 Project and Contributors

//! Basic Python Object customization
//!
//! Check [python c-api information](https://docs.python.org/3/reference/datamodel.html#basic-customization)
//! for more information.
//!
//! Parts of the documentation are copied from the respective methods from the
//! [typeobj docs](https://docs.python.org/3/c-api/typeobj.html)

use crate::callback::{BoolCallbackConverter, HashConverter, PyObjectCallbackConverter};
use crate::class::methods::PyMethodDef;
use crate::err::{PyErr, PyResult};
use crate::exceptions;
use crate::ffi;
use crate::objectprotocol::ObjectProtocol;
use crate::type_object::PyTypeInfo;
use crate::types::PyAny;
use crate::IntoPyPointer;
use crate::Python;
use crate::{FromPyObject, IntoPyObject};
use std::os::raw::c_int;
use std::ptr;

/// Operators for the __richcmp__ method
#[derive(Debug)]
pub enum CompareOp {
    Lt = ffi::Py_LT as isize,
    Le = ffi::Py_LE as isize,
    Eq = ffi::Py_EQ as isize,
    Ne = ffi::Py_NE as isize,
    Gt = ffi::Py_GT as isize,
    Ge = ffi::Py_GE as isize,
}

/// Basic python class customization
#[allow(unused_variables)]
pub trait PyObjectProtocol<'p>: PyTypeInfo {
    fn __getattr__(&'p self, name: Self::Name) -> Self::Result
    where
        Self: PyObjectGetAttrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __setattr__(&'p mut self, name: Self::Name, value: Self::Value) -> Self::Result
    where
        Self: PyObjectSetAttrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __delattr__(&'p mut self, name: Self::Name) -> Self::Result
    where
        Self: PyObjectDelAttrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __str__(&'p self) -> Self::Result
    where
        Self: PyObjectStrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __repr__(&'p self) -> Self::Result
    where
        Self: PyObjectReprProtocol<'p>,
    {
        unimplemented!()
    }

    fn __format__(&'p self, format_spec: Self::Format) -> Self::Result
    where
        Self: PyObjectFormatProtocol<'p>,
    {
        unimplemented!()
    }

    fn __hash__(&'p self) -> Self::Result
    where
        Self: PyObjectHashProtocol<'p>,
    {
        unimplemented!()
    }

    fn __bool__(&'p self) -> Self::Result
    where
        Self: PyObjectBoolProtocol<'p>,
    {
        unimplemented!()
    }

    fn __bytes__(&'p self) -> Self::Result
    where
        Self: PyObjectBytesProtocol<'p>,
    {
        unimplemented!()
    }

    fn __richcmp__(&'p self, other: Self::Other, op: CompareOp) -> Self::Result
    where
        Self: PyObjectRichcmpProtocol<'p>,
    {
        unimplemented!()
    }
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
    type Result: Into<PyResult<isize>>;
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
    fn methods() -> Vec<PyMethodDef> {
        Vec::new()
    }
    fn tp_as_object(_type_object: &mut ffi::PyTypeObject) {}
    fn nb_bool_fn() -> Option<ffi::inquiry> {
        None
    }
}

impl<T> PyObjectProtocolImpl for T {}

impl<'p, T> PyObjectProtocolImpl for T
where
    T: PyObjectProtocol<'p>,
{
    fn methods() -> Vec<PyMethodDef> {
        let mut methods = Vec::new();

        if let Some(def) = <Self as FormatProtocolImpl>::__format__() {
            methods.push(def)
        }
        if let Some(def) = <Self as BytesProtocolImpl>::__bytes__() {
            methods.push(def)
        }
        if let Some(def) = <Self as UnicodeProtocolImpl>::__unicode__() {
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
        type_object.tp_setattro = tp_setattro_impl::tp_setattro::<Self>();
    }
    fn nb_bool_fn() -> Option<ffi::inquiry> {
        Self::nb_bool()
    }
}

trait GetAttrProtocolImpl {
    fn tp_getattro() -> Option<ffi::binaryfunc> {
        None
    }
}

impl<'p, T> GetAttrProtocolImpl for T where T: PyObjectProtocol<'p> {}

impl<T> GetAttrProtocolImpl for T
where
    T: for<'p> PyObjectGetAttrProtocol<'p>,
{
    fn tp_getattro() -> Option<ffi::binaryfunc> {
        py_binary_func!(
            PyObjectGetAttrProtocol,
            T::__getattr__,
            T::Success,
            PyObjectCallbackConverter
        )
    }
}

/// An object may support setting attributes (by implementing PyObjectSetAttrProtocol)
/// and may support deleting attributes (by implementing PyObjectDelAttrProtocol)
/// and we need to generate a single extern c function that supports only setting, only deleting
/// or both, and return None in case none of the two is supported.
mod tp_setattro_impl {
    use super::*;

    /// setattrofunc PyTypeObject.tp_setattro
    ///
    /// An optional pointer to the function for setting and deleting attributes.
    ///
    /// The signature is the same as for PyObject_SetAttr(), but setting v to NULL to delete an
    /// attribute must be supported. It is usually convenient to set this field to
    /// PyObject_GenericSetAttr(), which implements the normal way of setting object attributes.
    pub(super) fn tp_setattro<'p, T: PyObjectProtocol<'p>>() -> Option<ffi::setattrofunc> {
        if let Some(set_del) = T::set_del_attr() {
            Some(set_del)
        } else if let Some(set) = T::set_attr() {
            Some(set)
        } else if let Some(del) = T::del_attr() {
            Some(del)
        } else {
            None
        }
    }

    trait SetAttr {
        fn set_attr() -> Option<ffi::setattrofunc> {
            None
        }
    }

    impl<'p, T: PyObjectProtocol<'p>> SetAttr for T {}

    impl<T> SetAttr for T
    where
        T: for<'p> PyObjectSetAttrProtocol<'p>,
    {
        fn set_attr() -> Option<ffi::setattrofunc> {
            py_func_set!(PyObjectSetAttrProtocol, T, __setattr__)
        }
    }

    trait DelAttr {
        fn del_attr() -> Option<ffi::setattrofunc> {
            None
        }
    }

    impl<'p, T> DelAttr for T where T: PyObjectProtocol<'p> {}

    impl<T> DelAttr for T
    where
        T: for<'p> PyObjectDelAttrProtocol<'p>,
    {
        fn del_attr() -> Option<ffi::setattrofunc> {
            py_func_del!(PyObjectDelAttrProtocol, T, __delattr__)
        }
    }

    trait SetDelAttr {
        fn set_del_attr() -> Option<ffi::setattrofunc> {
            None
        }
    }

    impl<'p, T> SetDelAttr for T where T: PyObjectProtocol<'p> {}

    impl<T> SetDelAttr for T
    where
        T: for<'p> PyObjectSetAttrProtocol<'p> + for<'p> PyObjectDelAttrProtocol<'p>,
    {
        fn set_del_attr() -> Option<ffi::setattrofunc> {
            py_func_set_del!(
                PyObjectSetAttrProtocol,
                PyObjectDelAttrProtocol,
                T,
                __setattr__,
                __delattr__
            )
        }
    }
}

trait StrProtocolImpl {
    fn tp_str() -> Option<ffi::unaryfunc> {
        None
    }
}
impl<'p, T> StrProtocolImpl for T where T: PyObjectProtocol<'p> {}
impl<T> StrProtocolImpl for T
where
    T: for<'p> PyObjectStrProtocol<'p>,
{
    fn tp_str() -> Option<ffi::unaryfunc> {
        py_unary_func!(
            PyObjectStrProtocol,
            T::__str__,
            <T as PyObjectStrProtocol>::Success,
            PyObjectCallbackConverter
        )
    }
}

trait ReprProtocolImpl {
    fn tp_repr() -> Option<ffi::unaryfunc> {
        None
    }
}
impl<'p, T> ReprProtocolImpl for T where T: PyObjectProtocol<'p> {}
impl<T> ReprProtocolImpl for T
where
    T: for<'p> PyObjectReprProtocol<'p>,
{
    fn tp_repr() -> Option<ffi::unaryfunc> {
        py_unary_func!(
            PyObjectReprProtocol,
            T::__repr__,
            T::Success,
            PyObjectCallbackConverter
        )
    }
}

#[doc(hidden)]
pub trait FormatProtocolImpl {
    fn __format__() -> Option<PyMethodDef> {
        None
    }
}
impl<'p, T> FormatProtocolImpl for T where T: PyObjectProtocol<'p> {}

#[doc(hidden)]
pub trait BytesProtocolImpl {
    fn __bytes__() -> Option<PyMethodDef> {
        None
    }
}
impl<'p, T> BytesProtocolImpl for T where T: PyObjectProtocol<'p> {}

#[doc(hidden)]
pub trait UnicodeProtocolImpl {
    fn __unicode__() -> Option<PyMethodDef> {
        None
    }
}
impl<'p, T> UnicodeProtocolImpl for T where T: PyObjectProtocol<'p> {}

trait HashProtocolImpl {
    fn tp_hash() -> Option<ffi::hashfunc> {
        None
    }
}
impl<'p, T> HashProtocolImpl for T where T: PyObjectProtocol<'p> {}
impl<T> HashProtocolImpl for T
where
    T: for<'p> PyObjectHashProtocol<'p>,
{
    fn tp_hash() -> Option<ffi::hashfunc> {
        py_unary_func!(
            PyObjectHashProtocol,
            T::__hash__,
            isize,
            HashConverter,
            ffi::Py_hash_t
        )
    }
}

trait BoolProtocolImpl {
    fn nb_bool() -> Option<ffi::inquiry> {
        None
    }
}
impl<'p, T> BoolProtocolImpl for T where T: PyObjectProtocol<'p> {}
impl<T> BoolProtocolImpl for T
where
    T: for<'p> PyObjectBoolProtocol<'p>,
{
    fn nb_bool() -> Option<ffi::inquiry> {
        py_unary_func!(
            PyObjectBoolProtocol,
            T::__bool__,
            bool,
            BoolCallbackConverter,
            c_int
        )
    }
}

trait RichcmpProtocolImpl {
    fn tp_richcompare() -> Option<ffi::richcmpfunc> {
        None
    }
}
impl<'p, T> RichcmpProtocolImpl for T where T: PyObjectProtocol<'p> {}
impl<T> RichcmpProtocolImpl for T
where
    T: for<'p> PyObjectRichcmpProtocol<'p>,
{
    fn tp_richcompare() -> Option<ffi::richcmpfunc> {
        unsafe extern "C" fn wrap<T>(
            slf: *mut ffi::PyObject,
            arg: *mut ffi::PyObject,
            op: c_int,
        ) -> *mut ffi::PyObject
        where
            T: for<'p> PyObjectRichcmpProtocol<'p>,
        {
            let _pool = crate::GILPool::new();
            let py = Python::assume_gil_acquired();
            let slf = py.from_borrowed_ptr::<T>(slf);
            let arg = py.from_borrowed_ptr::<PyAny>(arg);

            let res = match extract_op(op) {
                Ok(op) => match arg.extract() {
                    Ok(arg) => slf.__richcmp__(arg, op).into(),
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            };
            match res {
                Ok(val) => val.into_object(py).into_ptr(),
                Err(e) => {
                    e.restore(py);
                    ptr::null_mut()
                }
            }
        }
        Some(wrap::<T>)
    }
}

fn extract_op(op: c_int) -> PyResult<CompareOp> {
    match op {
        ffi::Py_LT => Ok(CompareOp::Lt),
        ffi::Py_LE => Ok(CompareOp::Le),
        ffi::Py_EQ => Ok(CompareOp::Eq),
        ffi::Py_NE => Ok(CompareOp::Ne),
        ffi::Py_GT => Ok(CompareOp::Gt),
        ffi::Py_GE => Ok(CompareOp::Ge),
        _ => Err(PyErr::new::<exceptions::ValueError, _>(
            "tp_richcompare called with invalid comparison operator",
        )),
    }
}
