// Copyright (c) 2017-present PyO3 Project and Contributors

//! Basic Python Object customization
//!
//! more information on python async support
//! https://docs.python.org/3/reference/datamodel.html#basic-customization

use std::os::raw::c_int;

use ::Py;
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
pub trait PyObjectProtocol<'a> : Sized + 'static {

    fn __getattr__(&'a self, name: Self::Name)
                       -> Self::Result where Self: PyObjectGetAttrProtocol<'a>
    { unimplemented!() }

    fn __setattr__(&self, py: Python, name: Self::Name, value: Self::Value)
                   -> Self::Result where Self: PyObjectSetAttrProtocol<'a> { unimplemented!() }

    fn __delattr__(&self, py: Python, name: Self::Name)
                   -> Self::Result where Self: PyObjectDelAttrProtocol<'a> { unimplemented!() }

    fn __str__(&self, py: Python)
               -> Self::Result where Self: PyObjectStrProtocol<'a> { unimplemented!() }

    fn __repr__(&self, py: Python)
                -> Self::Result where Self: PyObjectReprProtocol<'a> { unimplemented!() }

    fn __format__(&self, py: Python, format_spec: Self::Format)
                  -> Self::Result where Self: PyObjectFormatProtocol<'a> { unimplemented!() }

    fn __hash__(&self, py: Python)
                -> Self::Result where Self: PyObjectHashProtocol<'a> { unimplemented!() }

    fn __bool__(&self) -> Self::Result where Self: PyObjectBoolProtocol<'a> { unimplemented!() }

    fn __bytes__(&self, py: Python)
                 -> Self::Result where Self: PyObjectBytesProtocol<'a> { unimplemented!() }

    fn __richcmp__(&self, py: Python, other: Self::Other, op: CompareOp)
                   -> Self::Result where Self: PyObjectRichcmpProtocol<'a> { unimplemented!() }
}


pub trait PyObjectGetAttrProtocol<'a>: PyObjectProtocol<'a> {
    type Name: ::FromPyObj<'a> + ::class::typeob::PyTypeObjectInfo;
    type Result: Into<PyResult<()>>;
}

//pub trait PyObjectGetAttrProtocol: PyObjectProtocol + ::BaseObject + ::PyTypeObject {
//    type Name: for<'a> ::FromPyObj<'a>;
//    type Success: ToPyObject;
//    type Result: Into<PyResult<Self::Success>>;
//}
pub trait PyObjectSetAttrProtocol<'a>: PyObjectProtocol<'a> {
    type Name: FromPyObject<'a> + ::class::typeob::PyTypeObjectInfo + ::class::typeob::PyTypeObject + ::PythonObject;
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
        println!("getattr: {:?}", Self::tp_getattro());

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

use python::PyClone;
use callback::CallbackConverter;


impl<'a, T> PyObjectGetAttrProtocolImpl for T
    where T: PyObjectGetAttrProtocol<'a> + ::class::typeob::PyTypeObjectInfo
{
    #[inline]
    fn tp_getattro() -> Option<ffi::binaryfunc> {
        //py_binary_func_!(PyObjectGetAttrProtocol, T::__getattr__, PyObjectCallbackConverter)
        unsafe extern "C" fn wrap<'a, T>(slf: *mut ffi::PyObject,
                                         arg: *mut ffi::PyObject) -> *mut ffi::PyObject
            where T: PyObjectGetAttrProtocol<'a> + ::class::typeob::PyTypeObjectInfo
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            {
                println!("GETATTRO callback");
                let py = Python::assume_gil_acquired();
                //let arg1 = Py::from_borrowed_ptr(py, arg.clone());
                //let name: &Py<T::Name> = {&arg1 as *const _}.as_ref().unwrap();

                //::callback::handle_callback2(LOCATION, PyObjectCallbackConverter, |py| {
                let ret = match Py::<T::Name>::cast_from_borrowed(py, arg) {
                    Ok(arg) => {
                        let name: &Py<T::Name> = {&arg as *const _}.as_ref().unwrap();
                        match name.extr() {
                            Ok(name) => {
                                let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                                let slf1: &Py<T> = {&slf as *const _}.as_ref().unwrap();
                                let res = slf1.as_ref().__getattr__(name).into();
                                res
                            }
                            Err(e) => Err(e.into()),
                        }
                    },
                    Err(e) => Err(e.into()),
                };
                println!("GETATTRO callback 3 {:?}", ret);

                //$crate::PyDrop::release_ref(arg, py);
                //$crate::PyDrop::release_ref(slf, py);
                //res
                match ret {
                    Ok(val) => {
                        PyObjectCallbackConverter::convert(val, py)
                    }
                    Err(e) => {
                        e.restore(py);
                        //PyObjectCallbackConverter::error_value()
                        ::ptr::null_mut()
                    }
                }
            }
        }
        Some(wrap::<T>)
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

/*impl<T> PyObjectSetAttrProtocolImpl for T where T: PyObjectSetAttrProtocol
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
}*/


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

/*impl<T> PyObjectDelAttrProtocolImpl for T where T: PyObjectDelAttrProtocol
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
}*/


/*impl<T> PyObjectDelAttrProtocolImpl for T
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
}*/


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
/*impl<T> PyObjectStrProtocolImpl for T where T: PyObjectStrProtocol
{
    #[inline]
    fn tp_str() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyObjectStrProtocol, T::__str__, PyObjectCallbackConverter)
    }
}*/

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
/*impl<T> PyObjectReprProtocolImpl for T where T: PyObjectReprProtocol
{
    #[inline]
    fn tp_repr() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyObjectReprProtocol, T::__repr__, PyObjectCallbackConverter)
    }
}*/

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
/*impl<T> PyObjectHashProtocolImpl for T where T: PyObjectHashProtocol
{
    #[inline]
    fn tp_hash() -> Option<ffi::hashfunc> {
        py_unary_func_!(PyObjectHashProtocol, T::__hash__, HashConverter, ffi::Py_hash_t)
    }
}*/

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
/*impl<T> PyObjectBoolProtocolImpl for T
    where T: PyObjectBoolProtocol + ::BaseObject<Type=T> + ::PyTypeObject
{
    #[inline]
    fn nb_bool() -> Option<ffi::inquiry> {
        //py_unary_func_2!(PyObjectBoolProtocol, T::__bool__, BoolCallbackConverter, c_int)
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject) -> c_int
            where T: PyObjectBoolProtocol
        {
            const LOCATION: &'static str = concat!(stringify!(T), ".", stringify!($f), "()");
            ::callback::handle_callback(LOCATION, BoolCallbackConverter, |py| {
                let slf: Py<T> = ::pyptr::from_borrowed_ptr(py, slf);
                let ret = slf.__bool__().into();
                //$crate::PyDrop::release_ref(slf, py);
                ret
            })
        }
        Some(wrap::<T>)
    }
}*/

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
/*impl<T> PyObjectRichcmpProtocolImpl for T where T: PyObjectRichcmpProtocol
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
}*/


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
