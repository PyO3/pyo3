// Copyright (c) 2017-present PyO3 Project and Contributors

//! Basic Python Object customization
//!
//! more information on python async support
//! https://docs.python.org/3/reference/datamodel.html#basic-customization

use std::os::raw::c_int;

use ::{Py, CompareOp};
use ffi;
use err::{PyErr, PyResult};
use python::{Python};
use objects::{exc, PyObject};
use typeob::PyTypeInfo;
use conversion::{ToPyObject, FromPyObject};
use callback::{PyObjectCallbackConverter, HashConverter, UnitCallbackConverter,
               BoolCallbackConverter};
use class::methods::PyMethodDef;

// classmethod
// staticmethod
// __instancecheck__
// __subclasscheck__


/// Object customization
#[allow(unused_variables)]
pub trait PyObjectProtocol<'a>: PyTypeInfo + Sized + 'static {

    fn __getattr__(&self, name: Self::Name)
                   -> Self::Result where Self: PyObjectGetAttrProtocol<'a> {unimplemented!()}

    fn __setattr__(&self, name: Self::Name, value: Self::Value)
                   -> Self::Result where Self: PyObjectSetAttrProtocol<'a> {unimplemented!()}

    fn __delattr__(&self, name: Self::Name)
                   -> Self::Result where Self: PyObjectDelAttrProtocol<'a> {unimplemented!()}

    fn __str__(&self) -> Self::Result where Self: PyObjectStrProtocol<'a> {unimplemented!()}

    fn __repr__(&self) -> Self::Result where Self: PyObjectReprProtocol<'a> {unimplemented!()}

    fn __format__(&self, format_spec: Self::Format)
                  -> Self::Result where Self: PyObjectFormatProtocol<'a> {unimplemented!()}

    fn __hash__(&self) -> Self::Result where Self: PyObjectHashProtocol<'a> {unimplemented!()}

    fn __bool__(&self) -> Self::Result where Self: PyObjectBoolProtocol<'a> {unimplemented!()}

    fn __bytes__(&self) -> Self::Result where Self: PyObjectBytesProtocol<'a> {unimplemented!()}

    fn __richcmp__(&self, other: Self::Other, op: CompareOp)
                   -> Self::Result where Self: PyObjectRichcmpProtocol<'a> {unimplemented!()}
}


pub trait PyObjectGetAttrProtocol<'a>: PyObjectProtocol<'a> {
    type Name: FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectSetAttrProtocol<'a>: PyObjectProtocol<'a> {
    type Name: FromPyObject<'a>;
    type Value: FromPyObject<'a>;
    type Result: Into<PyResult<()>>;
}
pub trait PyObjectDelAttrProtocol<'a>: PyObjectProtocol<'a> {
    type Name: FromPyObject<'a>;
    type Result: Into<PyResult<()>>;
}
pub trait PyObjectStrProtocol<'a>: PyObjectProtocol<'a> {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectReprProtocol<'a>: PyObjectProtocol<'a> {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectFormatProtocol<'a>: PyObjectProtocol<'a> {
    type Format: FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectHashProtocol<'a>: PyObjectProtocol<'a> {
    type Result: Into<PyResult<usize>>;
}
pub trait PyObjectBoolProtocol<'a>: PyObjectProtocol<'a> {
    type Result: Into<PyResult<bool>>;
}
pub trait PyObjectBytesProtocol<'a>: PyObjectProtocol<'a> {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectRichcmpProtocol<'a>: PyObjectProtocol<'a> {
    type Other: FromPyObject<'a>;
    type Success: ToPyObject;
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

impl<'a, T> PyObjectProtocolImpl for T where T: PyObjectProtocol<'a> {
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
impl<'a, T> PyObjectGetAttrProtocolImpl for T where T: PyObjectProtocol<'a>
{
    #[inline]
    default fn tp_getattro() -> Option<ffi::binaryfunc> {
        None
    }
}
impl<'a, T> PyObjectGetAttrProtocolImpl for T where T: PyObjectGetAttrProtocol<'a>
{
    #[inline]
    fn tp_getattro() -> Option<ffi::binaryfunc> {
        py_binary_func!(PyObjectGetAttrProtocol, T::__getattr__, PyObjectCallbackConverter)
    }
}


trait PyObjectSetAttrProtocolImpl {
    fn tp_setattro() -> Option<ffi::setattrofunc>;
}

impl<'a, T> PyObjectSetAttrProtocolImpl for T where T: PyObjectProtocol<'a>
{
    #[inline]
    default fn tp_setattro() -> Option<ffi::setattrofunc> {
        None
    }
}
impl<'a, T> PyObjectSetAttrProtocolImpl for T where T: PyObjectSetAttrProtocol<'a>
{
    #[inline]
    fn tp_setattro() -> Option<ffi::setattrofunc> {
        unsafe extern "C" fn wrap<'a, T>(slf: *mut ffi::PyObject,
                                         name: *mut ffi::PyObject,
                                         value: *mut ffi::PyObject) -> c_int
            where T: PyObjectSetAttrProtocol<'a>
        {
            const LOCATION: &'static str = "T.__setattr__()";
            ::callback::handle(LOCATION, UnitCallbackConverter, |py| {
                if value.is_null() {
                    return Err(PyErr::new::<exc::NotImplementedError, _>(
                        py, format!("Subscript deletion not supported by {:?}",
                                    stringify!(T))))
                } else {
                    let name = ::PyObject::from_borrowed_ptr(py, name);
                    let value = ::PyObject::from_borrowed_ptr(py, value);

                    match ::callback::unref(name).extract() {
                        Ok(name) => match ::callback::unref(value).extract() {
                            Ok(value) => {
                                let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                                slf.as_ref().__setattr__(name, value).into()
                            },
                            Err(e) => Err(e.into()),
                        },
                        Err(e) => Err(e.into()),
                    }
                }
            })
        }
        Some(wrap::<T>)
    }
}


trait PyObjectDelAttrProtocolImpl {
    fn tp_delattro() -> Option<ffi::setattrofunc>;
}
impl<'a, T> PyObjectDelAttrProtocolImpl for T where T: PyObjectProtocol<'a>
{
    #[inline]
    default fn tp_delattro() -> Option<ffi::setattrofunc> {
        None
    }
}
impl<'a, T> PyObjectDelAttrProtocolImpl for T where T: PyObjectDelAttrProtocol<'a>
{
    #[inline]
    default fn tp_delattro() -> Option<ffi::setattrofunc> {
        unsafe extern "C" fn wrap<'a, T>(slf: *mut ffi::PyObject,
                                         name: *mut ffi::PyObject,
                                         value: *mut ffi::PyObject) -> c_int
            where T: PyObjectDelAttrProtocol<'a>
        {
            const LOCATION: &'static str = "T.__detattr__()";
            ::callback::handle(LOCATION, UnitCallbackConverter, |py| {
                if value.is_null() {
                    let name = ::PyObject::from_borrowed_ptr(py, name);

                    match ::callback::unref(name).extract() {
                        Ok(name) => {
                            let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                            slf.as_ref().__delattr__(name).into()
                        },
                        Err(e) => Err(e.into()),
                    }
                } else {
                    Err(PyErr::new::<exc::NotImplementedError, _>(
                        py, format!("Set attribute not supported by {:?}", stringify!(T))))
                }
            })
        }
        Some(wrap::<T>)
    }
}


impl<'a, T> PyObjectDelAttrProtocolImpl for T
    where T: PyObjectSetAttrProtocol<'a> + PyObjectDelAttrProtocol<'a>
{
    #[inline]
    fn tp_delattro() -> Option<ffi::setattrofunc> {
        unsafe extern "C" fn wrap<'a, T>(slf: *mut ffi::PyObject,
                                         name: *mut ffi::PyObject,
                                         value: *mut ffi::PyObject) -> c_int
            where T: PyObjectSetAttrProtocol<'a> + PyObjectDelAttrProtocol<'a>
        {
            const LOCATION: &'static str = "T.__detattr__()";
            ::callback::handle(LOCATION, UnitCallbackConverter, |py| {
                let name = ::PyObject::from_borrowed_ptr(py, name);
                if value.is_null() {
                    match ::callback::unref(name).extract() {
                        Ok(name) => {
                            let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                            slf.as_ref().__delattr__(name).into()
                        },
                        Err(e) => Err(e.into()),
                    }
                } else {
                    let value = ::PyObject::from_borrowed_ptr(py, value);
                    match ::callback::unref(name).extract() {
                        Ok(name) => match ::callback::unref(value).extract() {
                            Ok(value) => {
                                let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                                slf.as_ref().__setattr__(name, value).into()
                            },
                            Err(e) => Err(e.into()),
                        },
                        Err(e) => Err(e.into()),
                    }
                }
            })
        }
        Some(wrap::<T>)
    }
}


trait PyObjectStrProtocolImpl {
    fn tp_str() -> Option<ffi::unaryfunc>;
}
impl<'a, T> PyObjectStrProtocolImpl for T where T: PyObjectProtocol<'a>
{
    #[inline]
    default fn tp_str() -> Option<ffi::unaryfunc> {
        None
    }
}
impl<'a, T> PyObjectStrProtocolImpl for T where T: PyObjectStrProtocol<'a>
{
    #[inline]
    fn tp_str() -> Option<ffi::unaryfunc> {
        py_unary_func!(PyObjectStrProtocol, T::__str__, PyObjectCallbackConverter)
    }
}

trait PyObjectReprProtocolImpl {
    fn tp_repr() -> Option<ffi::unaryfunc>;
}
impl<'a, T> PyObjectReprProtocolImpl for T where T: PyObjectProtocol<'a>
{
    #[inline]
    default fn tp_repr() -> Option<ffi::unaryfunc> {
        None
    }
}
impl<'a, T> PyObjectReprProtocolImpl for T where T: PyObjectReprProtocol<'a>
{
    #[inline]
    fn tp_repr() -> Option<ffi::unaryfunc> {
        py_unary_func!(PyObjectReprProtocol, T::__repr__, PyObjectCallbackConverter)
    }
}

#[doc(hidden)]
pub trait PyObjectFormatProtocolImpl {
    fn __format__() -> Option<PyMethodDef>;
}
impl<'a, T> PyObjectFormatProtocolImpl for T where T: PyObjectProtocol<'a>
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
impl<'a, T> PyObjectBytesProtocolImpl for T where T: PyObjectProtocol<'a>
{
    #[inline]
    default fn __bytes__() -> Option<PyMethodDef> {
        None
    }
}


trait PyObjectHashProtocolImpl {
    fn tp_hash() -> Option<ffi::hashfunc>;
}
impl<'a, T> PyObjectHashProtocolImpl for T where T: PyObjectProtocol<'a>
{
    #[inline]
    default fn tp_hash() -> Option<ffi::hashfunc> {
        None
    }
}
impl<'a, T> PyObjectHashProtocolImpl for T where T: PyObjectHashProtocol<'a>
{
    #[inline]
    fn tp_hash() -> Option<ffi::hashfunc> {
        py_unary_func!(PyObjectHashProtocol, T::__hash__, HashConverter, ffi::Py_hash_t)
    }
}

trait PyObjectBoolProtocolImpl {
    fn nb_bool() -> Option<ffi::inquiry>;
}
impl<'a, T> PyObjectBoolProtocolImpl for T where T: PyObjectProtocol<'a>
{
    #[inline]
    default fn nb_bool() -> Option<ffi::inquiry> {
        None
    }
}
impl<'a, T> PyObjectBoolProtocolImpl for T where T: PyObjectBoolProtocol<'a>
{
    #[inline]
    fn nb_bool() -> Option<ffi::inquiry> {
        py_unary_func!(PyObjectBoolProtocol, T::__bool__, BoolCallbackConverter, c_int)
    }
}

trait PyObjectRichcmpProtocolImpl {
    fn tp_richcompare() -> Option<ffi::richcmpfunc>;
}
impl<'a, T> PyObjectRichcmpProtocolImpl for T where T: PyObjectProtocol<'a>
{
    #[inline]
    default fn tp_richcompare() -> Option<ffi::richcmpfunc> {
        None
    }
}
impl<'a, T> PyObjectRichcmpProtocolImpl for T where T: PyObjectRichcmpProtocol<'a>
{
    #[inline]
    fn tp_richcompare() -> Option<ffi::richcmpfunc> {
        unsafe extern "C" fn wrap<'a, T>(slf: *mut ffi::PyObject,
                                     arg: *mut ffi::PyObject,
                                     op: c_int) -> *mut ffi::PyObject
            where T: PyObjectRichcmpProtocol<'a>
        {
            const LOCATION: &'static str = concat!(stringify!(T), ".__richcmp__()");
            ::callback::handle(LOCATION, PyObjectCallbackConverter, |py| {
                match extract_op(py, op) {
                    Ok(op) => {
                        let arg = PyObject::from_borrowed_ptr(py, arg);
                        match ::callback::unref(arg).extract() {
                            Ok(arg) => {
                                let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                                slf.as_ref().__richcmp__(arg, op).into()
                            }
                            Err(e) => Err(e.into()),
                        }
                    },
                    Err(e) => Err(e)
                }
            })
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
            py.get_ptype::<exc::ValueError>(),
            Some("tp_richcompare called with invalid comparison operator"
                 .to_object(py).into_pptr())))
    }
}
