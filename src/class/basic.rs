// Copyright (c) 2017-present PyO3 Project and Contributors

//! Basic Python Object customization
//!
//! more information on python async support
//! https://docs.python.org/3/reference/datamodel.html#basic-customization

use std::os::raw::c_int;

use ::CompareOp;
use ffi;
use err::{PyErr, PyResult};
use python::{Python, PythonObject, PyDrop};
use objects::{exc, PyObject};
use conversion::{ToPyObject, FromPyObject};
use callback::{handle_callback, PyObjectCallbackConverter,
               HashConverter, UnitCallbackConverter, BoolCallbackConverter};
use class::methods::PyMethodDef;

// classmethod
// staticmethod
// __instancecheck__
// __subclasscheck__


/// Object customization
#[allow(unused_variables)]
pub trait PyObjectProtocol: PythonObject {

    fn __getattr__(&self, py: Python, name: Self::Name)
                   -> Self::Result where Self: PyObjectGetAttrProtocol { unimplemented!() }

    fn __setattr__(&self, py: Python, name: Self::Name, value: Self::Value)
                   -> Self::Result where Self: PyObjectSetAttrProtocol { unimplemented!() }

    fn __delattr__(&self, py: Python, name: Self::Name)
                   -> Self::Result where Self: PyObjectDelAttrProtocol { unimplemented!() }

    fn __str__(&self, py: Python)
               -> Self::Result where Self: PyObjectStrProtocol { unimplemented!() }

    fn __repr__(&self, py: Python)
                -> Self::Result where Self: PyObjectReprProtocol { unimplemented!() }

    fn __format__(&self, py: Python, format_spec: Self::Format)
                  -> Self::Result where Self: PyObjectFormatProtocol { unimplemented!() }

    fn __hash__(&self, py: Python)
                -> Self::Result where Self: PyObjectHashProtocol { unimplemented!() }

    fn __bool__(&self, py: Python)
                -> Self::Result where Self: PyObjectBoolProtocol { unimplemented!() }

    fn __bytes__(&self, py: Python)
                 -> Self::Result where Self: PyObjectBytesProtocol { unimplemented!() }

    fn __richcmp__(&self, py: Python, other: Self::Other, op: CompareOp)
                   -> Self::Result where Self: PyObjectRichcmpProtocol { unimplemented!() }
}


pub trait PyObjectGetAttrProtocol: PyObjectProtocol {
    type Name: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectSetAttrProtocol: PyObjectProtocol {
    type Name: for<'a> FromPyObject<'a>;
    type Value: for<'a> FromPyObject<'a>;
    type Result: Into<PyResult<()>>;
}
pub trait PyObjectDelAttrProtocol: PyObjectProtocol {
    type Name: for<'a> FromPyObject<'a>;
    type Result: Into<PyResult<()>>;
}
pub trait PyObjectStrProtocol: PyObjectProtocol {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectReprProtocol: PyObjectProtocol {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectFormatProtocol: PyObjectProtocol {
    type Format: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectHashProtocol: PyObjectProtocol {
    type Result: Into<PyResult<usize>>;
}
pub trait PyObjectBoolProtocol: PyObjectProtocol {
    type Result: Into<PyResult<bool>>;
}
pub trait PyObjectBytesProtocol: PyObjectProtocol {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyObjectRichcmpProtocol: PyObjectProtocol {
    type Other: for<'a> FromPyObject<'a>;
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

impl<T> PyObjectProtocolImpl for T where T: PyObjectProtocol {
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

impl<T> PyObjectGetAttrProtocolImpl for T
    where T: PyObjectProtocol
{
    #[inline]
    default fn tp_getattro() -> Option<ffi::binaryfunc> {
        None
    }
}

impl<T> PyObjectGetAttrProtocolImpl for T where T: PyObjectGetAttrProtocol
{
    #[inline]
    fn tp_getattro() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyObjectGetAttrProtocol, T::__getattr__, PyObjectCallbackConverter)
    }
}


trait PyObjectSetAttrProtocolImpl {
    fn tp_setattro() -> Option<ffi::setattrofunc>;
}

impl<T> PyObjectSetAttrProtocolImpl for T where T: PyObjectProtocol
{
    #[inline]
    default fn tp_setattro() -> Option<ffi::setattrofunc> {
        None
    }
}

impl<T> PyObjectSetAttrProtocolImpl for T where T: PyObjectSetAttrProtocol
{
    #[inline]
    fn tp_setattro() -> Option<ffi::setattrofunc> {
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                     name: *mut ffi::PyObject,
                                     value: *mut ffi::PyObject) -> c_int
            where T: PyObjectSetAttrProtocol
        {
            const LOCATION: &'static str = "T.__setattr__()";
            ::callback::handle_callback(LOCATION, UnitCallbackConverter, |py| {
                let slf = PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let name = PyObject::from_borrowed_ptr(py, name);

                let ret = match name.extract(py) {
                    Ok(key) =>
                        if value.is_null() {
                            Err(PyErr::new::<exc::NotImplementedError, _>(
                                py, format!("Subscript deletion not supported by {:?}",
                                            stringify!(T))))
                        } else {
                            let value = PyObject::from_borrowed_ptr(py, value);
                            let ret = match value.extract(py) {
                                Ok(value) => slf.__setattr__(py, key, value).into(),
                                Err(e) => Err(e),
                            };
                            PyDrop::release_ref(value, py);
                            ret
                        },
                    Err(e) => Err(e),
                };

                PyDrop::release_ref(name, py);
                PyDrop::release_ref(slf, py);
                ret
            })
        }
        Some(wrap::<T>)
    }
}


trait PyObjectDelAttrProtocolImpl {
    fn tp_delattro() -> Option<ffi::setattrofunc>;
}

impl<T> PyObjectDelAttrProtocolImpl for T where T: PyObjectProtocol
{
    #[inline]
    default fn tp_delattro() -> Option<ffi::setattrofunc> {
        None
    }
}

impl<T> PyObjectDelAttrProtocolImpl for T where T: PyObjectDelAttrProtocol
{
    #[inline]
    default fn tp_delattro() -> Option<ffi::setattrofunc> {
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                     name: *mut ffi::PyObject,
                                     value: *mut ffi::PyObject) -> c_int
            where T: PyObjectDelAttrProtocol
        {
            const LOCATION: &'static str = "T.__detattr__()";
            ::callback::handle_callback(LOCATION, UnitCallbackConverter, |py| {
                let slf = PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let name = PyObject::from_borrowed_ptr(py, name);

                let ret = match name.extract(py) {
                    Ok(name) =>
                        if value.is_null() {
                            slf.__delattr__(py, name).into()
                        } else {
                            Err(PyErr::new::<exc::NotImplementedError, _>(
                                py, format!("Set attribute not supported by {:?}",
                                            stringify!(T))))
                        },
                    Err(e) => Err(e),
                };

                PyDrop::release_ref(name, py);
                PyDrop::release_ref(slf, py);
                ret
            })
        }
        Some(wrap::<T>)
    }
}


impl<T> PyObjectDelAttrProtocolImpl for T
    where T: PyObjectSetAttrProtocol + PyObjectDelAttrProtocol
{
    #[inline]
    fn tp_delattro() -> Option<ffi::setattrofunc> {
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                     name: *mut ffi::PyObject,
                                     value: *mut ffi::PyObject) -> c_int
            where T: PyObjectSetAttrProtocol + PyObjectDelAttrProtocol
        {
            const LOCATION: &'static str = "T.__detattr__()";
            ::callback::handle_callback(LOCATION, UnitCallbackConverter, |py| {
                let slf = PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let name = PyObject::from_borrowed_ptr(py, name);

                let ret = if value.is_null() {
                    match name.extract(py) {
                        Ok(key) => slf.__delattr__(py, key).into(),
                        Err(e) => Err(e)
                    }
                } else {
                    match name.extract(py) {
                        Ok(name) => {
                            let value = PyObject::from_borrowed_ptr(py, value);
                            let ret = match value.extract(py) {
                                Ok(value) => slf.__setattr__(py, name, value).into(),
                                Err(e) => Err(e),
                            };
                            PyDrop::release_ref(value, py);
                            ret
                        },
                        Err(e) => Err(e),
                    }
                };

                PyDrop::release_ref(name, py);
                PyDrop::release_ref(slf, py);
                ret
            })
        }
        Some(wrap::<T>)
    }
}


trait PyObjectStrProtocolImpl {
    fn tp_str() -> Option<ffi::unaryfunc>;
}
impl<T> PyObjectStrProtocolImpl for T where T: PyObjectProtocol
{
    #[inline]
    default fn tp_str() -> Option<ffi::unaryfunc> {
        None
    }
}
impl<T> PyObjectStrProtocolImpl for T where T: PyObjectStrProtocol
{
    #[inline]
    fn tp_str() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyObjectStrProtocol, T::__str__, PyObjectCallbackConverter)
    }
}

trait PyObjectReprProtocolImpl {
    fn tp_repr() -> Option<ffi::unaryfunc>;
}
impl<T> PyObjectReprProtocolImpl for T where T: PyObjectProtocol
{
    #[inline]
    default fn tp_repr() -> Option<ffi::unaryfunc> {
        None
    }
}
impl<T> PyObjectReprProtocolImpl for T where T: PyObjectReprProtocol
{
    #[inline]
    fn tp_repr() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyObjectReprProtocol, T::__repr__, PyObjectCallbackConverter)
    }
}

#[doc(hidden)]
pub trait PyObjectFormatProtocolImpl {
    fn __format__() -> Option<PyMethodDef>;
}
impl<T> PyObjectFormatProtocolImpl for T where T: PyObjectProtocol
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
impl<T> PyObjectBytesProtocolImpl for T where T: PyObjectProtocol
{
    #[inline]
    default fn __bytes__() -> Option<PyMethodDef> {
        None
    }
}


trait PyObjectHashProtocolImpl {
    fn tp_hash() -> Option<ffi::hashfunc>;
}
impl<T> PyObjectHashProtocolImpl for T where T: PyObjectProtocol
{
    #[inline]
    default fn tp_hash() -> Option<ffi::hashfunc> {
        None
    }
}
impl<T> PyObjectHashProtocolImpl for T where T: PyObjectHashProtocol
{
    #[inline]
    fn tp_hash() -> Option<ffi::hashfunc> {
        py_unary_func_!(PyObjectHashProtocol, T::__hash__, HashConverter, ffi::Py_hash_t)
    }
}

trait PyObjectBoolProtocolImpl {
    fn nb_bool() -> Option<ffi::inquiry>;
}
impl<T> PyObjectBoolProtocolImpl for T where T: PyObjectProtocol
{
    #[inline]
    default fn nb_bool() -> Option<ffi::inquiry> {
        None
    }
}
impl<T> PyObjectBoolProtocolImpl for T where T: PyObjectBoolProtocol
{
    #[inline]
    fn nb_bool() -> Option<ffi::inquiry> {
        py_unary_func_!(PyObjectBoolProtocol, T::__bool__, BoolCallbackConverter, c_int)
    }
}

trait PyObjectRichcmpProtocolImpl {
    fn tp_richcompare() -> Option<ffi::richcmpfunc>;
}
impl<T> PyObjectRichcmpProtocolImpl for T where T: PyObjectProtocol
{
    #[inline]
    default fn tp_richcompare() -> Option<ffi::richcmpfunc> {
        None
    }
}
impl<T> PyObjectRichcmpProtocolImpl for T where T: PyObjectRichcmpProtocol
{
    #[inline]
    fn tp_richcompare() -> Option<ffi::richcmpfunc> {
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                     arg: *mut ffi::PyObject,
                                     op: c_int) -> *mut ffi::PyObject
            where T: PyObjectRichcmpProtocol
        {
            const LOCATION: &'static str = concat!(stringify!(T), ".__richcmp__()");
            handle_callback(LOCATION, PyObjectCallbackConverter, |py| {
                let slf = PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let arg = PyObject::from_borrowed_ptr(py, arg);

                let ret = match arg.extract(py) {
                    Ok(arg) => match extract_op(py, op) {
                        Ok(op) => slf.__richcmp__(py, arg, op).into(),
                        Err(e) => Err(e)
                    },
                    Err(e) => Err(e)
                };
                PyDrop::release_ref(arg, py);
                PyDrop::release_ref(slf, py);
                ret
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
            py.get_type::<exc::ValueError>(),
            Some("tp_richcompare called with invalid comparison operator"
                 .to_py_object(py).into_object())))
    }
}
