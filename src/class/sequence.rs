// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Sequence Interface
//! Trait and support implementation for implementing sequence

use std::os::raw::c_int;

use ::Py;
use ffi;
use err::{PyErr, PyResult};
use objects::exc;
use callback::{PyObjectCallbackConverter,
               LenResultConverter, UnitCallbackConverter, BoolCallbackConverter};
use class::typeob::PyTypeInfo;
use ::{ToPyObject, FromPyObj};


/// Sequece interface
#[allow(unused_variables)]
pub trait PySequenceProtocol<'a>: PyTypeInfo + Sized + 'static {
    fn __len__(&self) -> Self::Result
        where Self: PySequenceLenProtocol<'a> { unimplemented!() }

    fn __getitem__(&self, key: isize) -> Self::Result
        where Self: PySequenceGetItemProtocol<'a> { unimplemented!() }

    fn __setitem__(&self, key: isize, value: Self::Value) -> Self::Result
        where Self: PySequenceSetItemProtocol<'a> { unimplemented!() }

    fn __delitem__(&self, key: isize) -> Self::Result
        where Self: PySequenceDelItemProtocol<'a> { unimplemented!() }

    fn __contains__(&self, item: Self::Item) -> Self::Result
        where Self: PySequenceContainsProtocol<'a> { unimplemented!() }

    fn __concat__(&self, other: Self::Other) -> Self::Result
        where Self: PySequenceConcatProtocol<'a> { unimplemented!() }

    fn __repeat__(&self, count: isize) -> Self::Result
        where Self: PySequenceRepeatProtocol<'a> { unimplemented!() }

    fn __inplace_concat__(&self, other: Self::Other) -> Self::Result
        where Self: PySequenceInplaceConcatProtocol<'a> { unimplemented!() }

    fn __inplace_repeat__(&self, count: isize) -> Self::Result
        where Self: PySequenceInplaceRepeatProtocol<'a> { unimplemented!() }
}

// The following are a bunch of marker traits used to detect
// the existance of a slotted method.

pub trait PySequenceLenProtocol<'a>: PySequenceProtocol<'a> {
    type Result: Into<PyResult<usize>>;
}

pub trait PySequenceGetItemProtocol<'a>: PySequenceProtocol<'a> {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PySequenceSetItemProtocol<'a>: PySequenceProtocol<'a> {
    type Value: FromPyObj<'a>;
    type Result: Into<PyResult<()>>;
}

pub trait PySequenceDelItemProtocol<'a>: PySequenceProtocol<'a> {
    type Result: Into<PyResult<()>>;
}

pub trait PySequenceContainsProtocol<'a>: PySequenceProtocol<'a> {
    type Item: FromPyObj<'a>;
    type Result: Into<PyResult<bool>>;
}

pub trait PySequenceConcatProtocol<'a>: PySequenceProtocol<'a> {
    type Other: FromPyObj<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PySequenceRepeatProtocol<'a>: PySequenceProtocol<'a> {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PySequenceInplaceConcatProtocol<'a>: PySequenceProtocol<'a> + ToPyObject {
    type Other: FromPyObj<'a>;
    type Result: Into<PyResult<Self>>;
}

pub trait PySequenceInplaceRepeatProtocol<'a>: PySequenceProtocol<'a> + ToPyObject {
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

impl<'a, T> PySequenceProtocolImpl for T where T: PySequenceProtocol<'a> {
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

impl<'a, T> PySequenceLenProtocolImpl for T where T: PySequenceProtocol<'a>
{
    #[inline]
    default fn sq_length() -> Option<ffi::lenfunc> {
        None
    }
}

impl<'a, T> PySequenceLenProtocolImpl for T where T: PySequenceLenProtocol<'a>
{
    #[inline]
    fn sq_length() -> Option<ffi::lenfunc> {
        py_len_func2!(PySequenceLenProtocol, T::__len__, LenResultConverter)
    }
}

trait PySequenceGetItemProtocolImpl {
    fn sq_item() -> Option<ffi::ssizeargfunc>;
}

impl<'a, T> PySequenceGetItemProtocolImpl for T where T: PySequenceProtocol<'a>
{
    #[inline]
    default fn sq_item() -> Option<ffi::ssizeargfunc> {
        None
    }
}

impl<'a, T> PySequenceGetItemProtocolImpl for T where T: PySequenceGetItemProtocol<'a>
{
    #[inline]
    fn sq_item() -> Option<ffi::ssizeargfunc> {
        py_ssizearg_func!(PySequenceGetItemProtocol, T::__getitem__, PyObjectCallbackConverter)
    }
}

trait PySequenceSetItemProtocolImpl {
    fn sq_ass_item() -> Option<ffi::ssizeobjargproc>;
}

impl<'a, T> PySequenceSetItemProtocolImpl for T where T: PySequenceProtocol<'a>
{
    #[inline]
    default fn sq_ass_item() -> Option<ffi::ssizeobjargproc> {
        None
    }
}

impl<'a, T> PySequenceSetItemProtocolImpl for T where T: PySequenceSetItemProtocol<'a>
{
    #[inline]
    fn sq_ass_item() -> Option<ffi::ssizeobjargproc> {
        unsafe extern "C" fn wrap<'a, T>(slf: *mut ffi::PyObject,
                                         key: ffi::Py_ssize_t,
                                         value: *mut ffi::PyObject) -> c_int
            where T: PySequenceSetItemProtocol<'a>
        {
            const LOCATION: &'static str = "foo.__setitem__()";
            ::callback::handle_callback2(LOCATION, UnitCallbackConverter, |py| {
                if value.is_null() {
                    Err(PyErr::new::<exc::NotImplementedError, _>(
                        py, format!("Item deletion not supported by {:?}",
                                    stringify!(T))))
                } else {
                    match Py::<T::Value>::cast_from_borrowed(py, value) {
                        Ok(value) => {
                            let value1: &Py<T::Value> = {&value as *const _}.as_ref().unwrap();
                            match value1.extr() {
                                Ok(value) => {
                                    let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                                    slf.as_ref().__setitem__(key as isize, value).into()
                                },
                                Err(e) => Err(e.into()),
                            }
                        },
                        Err(e) => Err(e.into())
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
impl<'a, T> PySequenceDelItemProtocolImpl for T where T: PySequenceProtocol<'a>
{
    #[inline]
    default fn sq_del_item() -> Option<ffi::ssizeobjargproc> {
        None
    }
}

impl<'a, T> PySequenceDelItemProtocolImpl for T where T: PySequenceDelItemProtocol<'a>
{
    #[inline]
    default fn sq_del_item() -> Option<ffi::ssizeobjargproc> {
        unsafe extern "C" fn wrap<'a, T>(slf: *mut ffi::PyObject,
                                         key: ffi::Py_ssize_t,
                                         value: *mut ffi::PyObject) -> c_int
            where T: PySequenceDelItemProtocol<'a>
        {
            const LOCATION: &'static str = "T.__detitem__()";
            ::callback::handle_callback2(LOCATION, UnitCallbackConverter, |py| {
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

impl<'a, T> PySequenceDelItemProtocolImpl for T
    where T: PySequenceSetItemProtocol<'a> + PySequenceDelItemProtocol<'a>
{
    #[inline]
    fn sq_del_item() -> Option<ffi::ssizeobjargproc> {
        unsafe extern "C" fn wrap<'a, T>(slf: *mut ffi::PyObject,
                                     key: ffi::Py_ssize_t,
                                     value: *mut ffi::PyObject) -> c_int
            where T: PySequenceSetItemProtocol<'a> + PySequenceDelItemProtocol<'a>
        {
            const LOCATION: &'static str = "T.__set/del_item__()";

            ::callback::handle_callback2(LOCATION, UnitCallbackConverter, |py| {
                if value.is_null() {
                    let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                    slf.__delitem__(key as isize).into()
                } else {
                    match Py::<T::Value>::cast_from_borrowed(py, value) {
                        Ok(value) => {
                            let value1: &Py<T::Value> = {&value as *const _}.as_ref().unwrap();
                            match value1.extr() {
                                Ok(value) => {
                                    let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                                    slf.as_ref().__setitem__(key as isize, value).into()
                                },
                                Err(e) => Err(e.into()),
                            }
                        },
                        Err(e) => Err(e.into())
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

impl<'a, T> PySequenceContainsProtocolImpl for T where T: PySequenceProtocol<'a>
{
    #[inline]
    default fn sq_contains() -> Option<ffi::objobjproc> {
        None
    }
}

impl<'a, T> PySequenceContainsProtocolImpl for T where T: PySequenceContainsProtocol<'a>
{
    #[inline]
    fn sq_contains() -> Option<ffi::objobjproc> {
        unsafe extern "C" fn wrap<'a, T>(slf: *mut ffi::PyObject,
                                         arg: *mut ffi::PyObject) -> c_int
            where T: PySequenceContainsProtocol<'a>
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".__contains__()");
            ::callback::handle_callback2(LOCATION, BoolCallbackConverter, |py| {
                match Py::<T::Item>::cast_from_borrowed(py, arg) {
                    Ok(arg) => {
                        let item: &Py<T::Item> = {&arg as *const _}.as_ref().unwrap();
                        match item.extr() {
                            Ok(arg) => {
                                let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
                                slf.as_ref().__contains__(arg).into()
                            }
                            Err(e) => Err(e.into()),
                        }
                    },
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

impl<'a, T> PySequenceConcatProtocolImpl for T where T: PySequenceProtocol<'a>
{
    #[inline]
    default fn sq_concat() -> Option<ffi::binaryfunc> {
        None
    }
}

impl<'a, T> PySequenceConcatProtocolImpl for T where T: PySequenceConcatProtocol<'a>
{
    #[inline]
    fn sq_concat() -> Option<ffi::binaryfunc> {
        py_binary_func_2!(PySequenceConcatProtocol,
                          T::__concat__, Other, PyObjectCallbackConverter)
    }
}

trait PySequenceRepeatProtocolImpl {
    fn sq_repeat() -> Option<ffi::ssizeargfunc>;
}

impl<'a, T> PySequenceRepeatProtocolImpl for T
    where T: PySequenceProtocol<'a>
{
    #[inline]
    default fn sq_repeat() -> Option<ffi::ssizeargfunc> {
        None
    }
}

impl<'a, T> PySequenceRepeatProtocolImpl for T
    where T: PySequenceRepeatProtocol<'a>
{
    #[inline]
    fn sq_repeat() -> Option<ffi::ssizeargfunc> {
        py_ssizearg_func!(PySequenceRepeatProtocol, T::__repeat__, PyObjectCallbackConverter)
    }
}

trait PySequenceInplaceConcatProtocolImpl {
    fn sq_inplace_concat() -> Option<ffi::binaryfunc>;
}

impl<'a, T> PySequenceInplaceConcatProtocolImpl for T where T: PySequenceProtocol<'a>
{
    #[inline]
    default fn sq_inplace_concat() -> Option<ffi::binaryfunc> {
        None
    }
}

impl<'a, T> PySequenceInplaceConcatProtocolImpl for T
    where T: PySequenceInplaceConcatProtocol<'a>
{
    #[inline]
    fn sq_inplace_concat() -> Option<ffi::binaryfunc> {
        py_binary_func_2!(PySequenceInplaceConcatProtocol,
                          T::__inplace_concat__, Other, PyObjectCallbackConverter)
    }
}

trait PySequenceInplaceRepeatProtocolImpl {
    fn sq_inplace_repeat() -> Option<ffi::ssizeargfunc>;
}

impl<'a, T> PySequenceInplaceRepeatProtocolImpl for T where T: PySequenceProtocol<'a>
{
    #[inline]
    default fn sq_inplace_repeat() -> Option<ffi::ssizeargfunc> {
        None
    }
}

impl<'a, T> PySequenceInplaceRepeatProtocolImpl for T
    where T: PySequenceInplaceRepeatProtocol<'a>
{
    #[inline]
    fn sq_inplace_repeat() -> Option<ffi::ssizeargfunc> {
        py_ssizearg_func!(PySequenceInplaceRepeatProtocol, T::__inplace_repeat__, PyObjectCallbackConverter)
    }
}
