// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Sequence Interface
//! Trait and support implementation for implementing sequence

use std::os::raw::c_int;

use ::Py;
use ffi;
use err::{PyErr, PyResult};
use objects::{exc, PyObject};
use callback::{PyObjectCallbackConverter,
               LenResultConverter, UnitCallbackConverter, BoolCallbackConverter};
use typeob::PyTypeInfo;
use conversion::{ToPyObject, FromPyObject};


/// Sequece interface
#[allow(unused_variables)]
pub trait PySequenceProtocol<'p>: PyTypeInfo + Sized + 'static {
    fn __len__(&self) -> Self::Result
        where Self: PySequenceLenProtocol<'p> { unimplemented!() }

    fn __getitem__(&self, key: isize) -> Self::Result
        where Self: PySequenceGetItemProtocol<'p> { unimplemented!() }

    fn __setitem__(&self, key: isize, value: Self::Value) -> Self::Result
        where Self: PySequenceSetItemProtocol<'p> { unimplemented!() }

    fn __delitem__(&self, key: isize) -> Self::Result
        where Self: PySequenceDelItemProtocol<'p> { unimplemented!() }

    fn __contains__(&self, item: Self::Item) -> Self::Result
        where Self: PySequenceContainsProtocol<'p> { unimplemented!() }

    fn __concat__(&self, other: Self::Other) -> Self::Result
        where Self: PySequenceConcatProtocol<'p> { unimplemented!() }

    fn __repeat__(&self, count: isize) -> Self::Result
        where Self: PySequenceRepeatProtocol<'p> { unimplemented!() }

    fn __inplace_concat__(&self, other: Self::Other) -> Self::Result
        where Self: PySequenceInplaceConcatProtocol<'p> { unimplemented!() }

    fn __inplace_repeat__(&self, count: isize) -> Self::Result
        where Self: PySequenceInplaceRepeatProtocol<'p> { unimplemented!() }
}

// The following are a bunch of marker traits used to detect
// the existance of a slotted method.

pub trait PySequenceLenProtocol<'p>: PySequenceProtocol<'p> {
    type Result: Into<PyResult<usize>>;
}

pub trait PySequenceGetItemProtocol<'p>: PySequenceProtocol<'p> {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PySequenceSetItemProtocol<'p>: PySequenceProtocol<'p> {
    type Value: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}

pub trait PySequenceDelItemProtocol<'p>: PySequenceProtocol<'p> {
    type Result: Into<PyResult<()>>;
}

pub trait PySequenceContainsProtocol<'p>: PySequenceProtocol<'p> {
    type Item: FromPyObject<'p>;
    type Result: Into<PyResult<bool>>;
}

pub trait PySequenceConcatProtocol<'p>: PySequenceProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PySequenceRepeatProtocol<'p>: PySequenceProtocol<'p> {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PySequenceInplaceConcatProtocol<'p>: PySequenceProtocol<'p> + ToPyObject {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<Self>>;
}

pub trait PySequenceInplaceRepeatProtocol<'p>: PySequenceProtocol<'p> + ToPyObject {
    type Result: Into<PyResult<Self>>;
}

#[doc(hidden)]
pub trait PySequenceProtocolImpl {
    fn tp_as_sequence() -> Option<ffi::PySequenceMethods>;
}

impl<T> PySequenceProtocolImpl for T {
    #[inline]
    default fn tp_as_sequence() -> Option<ffi::PySequenceMethods> {
        None
    }
}

impl<'p, T> PySequenceProtocolImpl for T where T: PySequenceProtocol<'p> {
    #[inline]
    fn tp_as_sequence() -> Option<ffi::PySequenceMethods> {
        let f = if let Some(df) = Self::sq_del_item() {
            Some(df)
        } else {
            Self::sq_ass_item()
        };


        Some(ffi::PySequenceMethods {
            sq_length: Self::sq_length(),
            sq_concat: Self::sq_concat(),
            sq_repeat: Self::sq_repeat(),
            sq_item: Self::sq_item(),
            was_sq_slice: 0 as *mut _,
            sq_ass_item: f,
            was_sq_ass_slice: 0 as *mut _,
            sq_contains: Self::sq_contains(),
            sq_inplace_concat: Self::sq_inplace_concat(),
            sq_inplace_repeat: Self::sq_inplace_repeat(),
        })
    }
}

trait PySequenceLenProtocolImpl {
    fn sq_length() -> Option<ffi::lenfunc>;
}

impl<'p, T> PySequenceLenProtocolImpl for T where T: PySequenceProtocol<'p>
{
    #[inline]
    default fn sq_length() -> Option<ffi::lenfunc> {
        None
    }
}

impl<'p, T> PySequenceLenProtocolImpl for T where T: PySequenceLenProtocol<'p>
{
    #[inline]
    fn sq_length() -> Option<ffi::lenfunc> {
        py_len_func!(PySequenceLenProtocol, T::__len__, LenResultConverter)
    }
}

trait PySequenceGetItemProtocolImpl {
    fn sq_item() -> Option<ffi::ssizeargfunc>;
}

impl<'p, T> PySequenceGetItemProtocolImpl for T where T: PySequenceProtocol<'p>
{
    #[inline]
    default fn sq_item() -> Option<ffi::ssizeargfunc> {
        None
    }
}

impl<'p, T> PySequenceGetItemProtocolImpl for T where T: PySequenceGetItemProtocol<'p>
{
    #[inline]
    fn sq_item() -> Option<ffi::ssizeargfunc> {
        py_ssizearg_func!(PySequenceGetItemProtocol, T::__getitem__, PyObjectCallbackConverter)
    }
}

trait PySequenceSetItemProtocolImpl {
    fn sq_ass_item() -> Option<ffi::ssizeobjargproc>;
}

impl<'p, T> PySequenceSetItemProtocolImpl for T where T: PySequenceProtocol<'p>
{
    #[inline]
    default fn sq_ass_item() -> Option<ffi::ssizeobjargproc> {
        None
    }
}

impl<'p, T> PySequenceSetItemProtocolImpl for T where T: PySequenceSetItemProtocol<'p>
{
    #[inline]
    fn sq_ass_item() -> Option<ffi::ssizeobjargproc> {
        unsafe extern "C" fn wrap<'p, T>(slf: *mut ffi::PyObject,
                                         key: ffi::Py_ssize_t,
                                         value: *mut ffi::PyObject) -> c_int
            where T: PySequenceSetItemProtocol<'p>
        {
            const LOCATION: &'static str = "foo.__setitem__()";
            ::callback::handle(LOCATION, UnitCallbackConverter, |py| {
                if value.is_null() {
                    Err(PyErr::new::<exc::NotImplementedError, _>(
                        py, format!("Item deletion not supported by {:?}",
                                    stringify!(T))))
                } else {
                    let value = PyObject::from_borrowed_ptr(py, value);
                    match ::callback::unref(value).extract() {
                        Ok(value) => {
                            let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                            slf.as_ref().__setitem__(key as isize, value).into()
                        },
                        Err(e) => Err(e.into()),
                    }
                }
            })
        }
        Some(wrap::<T>)
    }
}

trait PySequenceDelItemProtocolImpl {
    fn sq_del_item() -> Option<ffi::ssizeobjargproc>;
}
impl<'p, T> PySequenceDelItemProtocolImpl for T where T: PySequenceProtocol<'p>
{
    #[inline]
    default fn sq_del_item() -> Option<ffi::ssizeobjargproc> {
        None
    }
}

impl<'p, T> PySequenceDelItemProtocolImpl for T where T: PySequenceDelItemProtocol<'p>
{
    #[inline]
    default fn sq_del_item() -> Option<ffi::ssizeobjargproc> {
        unsafe extern "C" fn wrap<'p, T>(slf: *mut ffi::PyObject,
                                         key: ffi::Py_ssize_t,
                                         value: *mut ffi::PyObject) -> c_int
            where T: PySequenceDelItemProtocol<'p>
        {
            const LOCATION: &'static str = "T.__detitem__()";
            ::callback::handle(LOCATION, UnitCallbackConverter, |py| {
                if value.is_null() {
                    let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                    slf.__delitem__(key as isize).into()
                } else {
                    Err(PyErr::new::<exc::NotImplementedError, _>(
                        py, format!("Item assignment not supported by {:?}",
                                    stringify!(T))))
                }
            })
        }
        Some(wrap::<T>)
    }
}

impl<'p, T> PySequenceDelItemProtocolImpl for T
    where T: PySequenceSetItemProtocol<'p> + PySequenceDelItemProtocol<'p>
{
    #[inline]
    fn sq_del_item() -> Option<ffi::ssizeobjargproc> {
        unsafe extern "C" fn wrap<'p, T>(slf: *mut ffi::PyObject,
                                         key: ffi::Py_ssize_t,
                                         value: *mut ffi::PyObject) -> c_int
            where T: PySequenceSetItemProtocol<'p> + PySequenceDelItemProtocol<'p>
        {
            const LOCATION: &'static str = "T.__set/del_item__()";

            ::callback::handle(LOCATION, UnitCallbackConverter, |py| {
                if value.is_null() {
                    let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                    slf.__delitem__(key as isize).into()
                } else {
                    let value = ::PyObject::from_borrowed_ptr(py, value);
                    match ::callback::unref(value).extract() {
                        Ok(value) => {
                            let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                            slf.as_ref().__setitem__(key as isize, value).into()
                        },
                        Err(e) => Err(e.into()),
                    }
                }
            })
        }
        Some(wrap::<T>)
    }
}


trait PySequenceContainsProtocolImpl {
    fn sq_contains() -> Option<ffi::objobjproc>;
}

impl<'p, T> PySequenceContainsProtocolImpl for T where T: PySequenceProtocol<'p>
{
    #[inline]
    default fn sq_contains() -> Option<ffi::objobjproc> {
        None
    }
}

impl<'p, T> PySequenceContainsProtocolImpl for T where T: PySequenceContainsProtocol<'p>
{
    #[inline]
    fn sq_contains() -> Option<ffi::objobjproc> {
        unsafe extern "C" fn wrap<'p, T>(slf: *mut ffi::PyObject,
                                         arg: *mut ffi::PyObject) -> c_int
            where T: PySequenceContainsProtocol<'p>
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".__contains__()");
            ::callback::handle(LOCATION, BoolCallbackConverter, |py| {
                let arg = ::PyObject::from_borrowed_ptr(py, arg);
                match ::callback::unref(arg).extract() {
                    Ok(arg) => {
                        let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                        slf.as_ref().__contains__(arg).into()
                    }
                    Err(e) => Err(e.into()),
                }
            })
        }
        Some(wrap::<T>)
    }
}

trait PySequenceConcatProtocolImpl {
    fn sq_concat() -> Option<ffi::binaryfunc>;
}

impl<'p, T> PySequenceConcatProtocolImpl for T where T: PySequenceProtocol<'p>
{
    #[inline]
    default fn sq_concat() -> Option<ffi::binaryfunc> {
        None
    }
}

impl<'p, T> PySequenceConcatProtocolImpl for T where T: PySequenceConcatProtocol<'p>
{
    #[inline]
    fn sq_concat() -> Option<ffi::binaryfunc> {
        py_binary_func!(PySequenceConcatProtocol, T::__concat__, PyObjectCallbackConverter)
    }
}

trait PySequenceRepeatProtocolImpl {
    fn sq_repeat() -> Option<ffi::ssizeargfunc>;
}

impl<'p, T> PySequenceRepeatProtocolImpl for T
    where T: PySequenceProtocol<'p>
{
    #[inline]
    default fn sq_repeat() -> Option<ffi::ssizeargfunc> {
        None
    }
}

impl<'p, T> PySequenceRepeatProtocolImpl for T
    where T: PySequenceRepeatProtocol<'p>
{
    #[inline]
    fn sq_repeat() -> Option<ffi::ssizeargfunc> {
        py_ssizearg_func!(PySequenceRepeatProtocol, T::__repeat__, PyObjectCallbackConverter)
    }
}

trait PySequenceInplaceConcatProtocolImpl {
    fn sq_inplace_concat() -> Option<ffi::binaryfunc>;
}

impl<'p, T> PySequenceInplaceConcatProtocolImpl for T where T: PySequenceProtocol<'p>
{
    #[inline]
    default fn sq_inplace_concat() -> Option<ffi::binaryfunc> {
        None
    }
}

impl<'p, T> PySequenceInplaceConcatProtocolImpl for T
    where T: PySequenceInplaceConcatProtocol<'p>
{
    #[inline]
    fn sq_inplace_concat() -> Option<ffi::binaryfunc> {
        py_binary_func!(PySequenceInplaceConcatProtocol,
                        T::__inplace_concat__, PyObjectCallbackConverter)
    }
}

trait PySequenceInplaceRepeatProtocolImpl {
    fn sq_inplace_repeat() -> Option<ffi::ssizeargfunc>;
}

impl<'p, T> PySequenceInplaceRepeatProtocolImpl for T where T: PySequenceProtocol<'p>
{
    #[inline]
    default fn sq_inplace_repeat() -> Option<ffi::ssizeargfunc> {
        None
    }
}

impl<'p, T> PySequenceInplaceRepeatProtocolImpl for T
    where T: PySequenceInplaceRepeatProtocol<'p>
{
    #[inline]
    fn sq_inplace_repeat() -> Option<ffi::ssizeargfunc> {
        py_ssizearg_func!(PySequenceInplaceRepeatProtocol, T::__inplace_repeat__, PyObjectCallbackConverter)
    }
}
