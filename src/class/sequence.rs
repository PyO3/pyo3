// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Sequence Interface
//! Trait and support implementation for implementing sequence

use std::os::raw::c_int;

use ffi;
use python::Python;
use err::{PyErr, PyResult};
use objects::{exc, PyObjectRef};
use objectprotocol::ObjectProtocol;
use callback::{PyObjectCallbackConverter, LenResultConverter, BoolCallbackConverter};
use typeob::PyTypeInfo;
use conversion::{IntoPyObject, FromPyObject};


/// Sequece interface
#[allow(unused_variables)]
pub trait PySequenceProtocol<'p>: PyTypeInfo + Sized
{
    fn __len__(&'p self) -> Self::Result
        where Self: PySequenceLenProtocol<'p> { unimplemented!() }

    fn __getitem__(&'p self, key: isize) -> Self::Result
        where Self: PySequenceGetItemProtocol<'p> { unimplemented!() }

    fn __setitem__(&'p mut self, key: isize, value: Self::Value) -> Self::Result
        where Self: PySequenceSetItemProtocol<'p> { unimplemented!() }

    fn __delitem__(&'p mut self, key: isize) -> Self::Result
        where Self: PySequenceDelItemProtocol<'p> { unimplemented!() }

    fn __contains__(&'p self, item: Self::Item) -> Self::Result
        where Self: PySequenceContainsProtocol<'p> { unimplemented!() }

    fn __concat__(&'p self, other: Self::Other) -> Self::Result
        where Self: PySequenceConcatProtocol<'p> { unimplemented!() }

    fn __repeat__(&'p self, count: isize) -> Self::Result
        where Self: PySequenceRepeatProtocol<'p> { unimplemented!() }

    fn __inplace_concat__(&'p mut self, other: Self::Other) -> Self::Result
        where Self: PySequenceInplaceConcatProtocol<'p> { unimplemented!() }

    fn __inplace_repeat__(&'p mut self, count: isize) -> Self::Result
        where Self: PySequenceInplaceRepeatProtocol<'p> { unimplemented!() }
}

// The following are a bunch of marker traits used to detect
// the existance of a slotted method.

pub trait PySequenceLenProtocol<'p>: PySequenceProtocol<'p> {
    type Result: Into<PyResult<usize>>;
}

pub trait PySequenceGetItemProtocol<'p>: PySequenceProtocol<'p> {
    type Success: IntoPyObject;
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
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PySequenceRepeatProtocol<'p>: PySequenceProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PySequenceInplaceConcatProtocol<'p>: PySequenceProtocol<'p> + IntoPyObject {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<Self>>;
}

pub trait PySequenceInplaceRepeatProtocol<'p>: PySequenceProtocol<'p> + IntoPyObject {
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
    #[cfg(Py_3)]
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
            was_sq_slice: ::std::ptr::null_mut(),
            sq_ass_item: f,
            was_sq_ass_slice: ::std::ptr::null_mut(),
            sq_contains: Self::sq_contains(),
            sq_inplace_concat: Self::sq_inplace_concat(),
            sq_inplace_repeat: Self::sq_inplace_repeat(),
        })
    }
    #[cfg(not(Py_3))]
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
            sq_slice: None,
            sq_ass_item: f,
            sq_ass_slice: None,
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

impl<T> PySequenceLenProtocolImpl for T where T: for<'p> PySequenceLenProtocol<'p>
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

impl<T> PySequenceGetItemProtocolImpl for T
    where T: for<'p> PySequenceGetItemProtocol<'p>
{
    #[inline]
    fn sq_item() -> Option<ffi::ssizeargfunc> {
        py_ssizearg_func!(
            PySequenceGetItemProtocol, T::__getitem__, T::Success, PyObjectCallbackConverter)
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

impl<T> PySequenceSetItemProtocolImpl for T
    where T: for<'p> PySequenceSetItemProtocol<'p>
{
    #[inline]
    fn sq_ass_item() -> Option<ffi::ssizeobjargproc> {
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                     key: ffi::Py_ssize_t,
                                     value: *mut ffi::PyObject) -> c_int
            where T: for<'p> PySequenceSetItemProtocol<'p>
        {
            let _pool = ::GILPool::new();
            let py = Python::assume_gil_acquired();
            let slf = py.mut_from_borrowed_ptr::<T>(slf);

            if value.is_null() {
                let e = PyErr::new::<exc::NotImplementedError, _>(
                    format!("Item deletion not supported by {:?}", stringify!(T)));
                e.restore(py);
                -1
            } else {
                let value = py.from_borrowed_ptr::<PyObjectRef>(value);
                let result = match value.extract() {
                    Ok(value) => {
                        slf.__setitem__(key as isize, value).into()
                    },
                    Err(e) => Err(e),
                };
                match result {
                    Ok(_) => 0,
                    Err(e) => {
                        e.restore(py);
                        -1
                    }
                }
            }
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

impl<T> PySequenceDelItemProtocolImpl for T
    where T: for<'p> PySequenceDelItemProtocol<'p>
{
    #[inline]
    default fn sq_del_item() -> Option<ffi::ssizeobjargproc> {
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                     key: ffi::Py_ssize_t,
                                     value: *mut ffi::PyObject) -> c_int
            where T: for<'p> PySequenceDelItemProtocol<'p>
        {
            let _pool = ::GILPool::new();
            let py = Python::assume_gil_acquired();
            let slf = py.mut_from_borrowed_ptr::<T>(slf);

            if value.is_null() {
                let result = slf.__delitem__(key as isize).into();
                match result {
                    Ok(_) => 0,
                    Err(e) => {
                        e.restore(py);
                        -1
                    }
                }
            } else {
                let e = PyErr::new::<exc::NotImplementedError, _>(
                    format!("Item assignment not supported by {:?}", stringify!(T)));
                e.restore(py);
                -1
            }
        }
        Some(wrap::<T>)
    }
}

impl<T> PySequenceDelItemProtocolImpl for T
    where T: for<'p> PySequenceSetItemProtocol<'p> + for<'p> PySequenceDelItemProtocol<'p>
{
    #[inline]
    fn sq_del_item() -> Option<ffi::ssizeobjargproc> {
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                     key: ffi::Py_ssize_t,
                                     value: *mut ffi::PyObject) -> c_int
            where T: for<'p> PySequenceSetItemProtocol<'p> +
               for<'p> PySequenceDelItemProtocol<'p>
        {
            let _pool = ::GILPool::new();
            let py = Python::assume_gil_acquired();
            let slf = py.mut_from_borrowed_ptr::<T>(slf);

            if value.is_null() {
                let result = slf.__delitem__(key as isize).into();
                match result {
                    Ok(_) => 0,
                    Err(e) => {
                        e.restore(py);
                        -1
                    }
                }
            } else {
                let value = py.from_borrowed_ptr::<PyObjectRef>(value);
                let result = match value.extract() {
                    Ok(value) => {
                        slf.__setitem__(key as isize, value).into()
                    },
                    Err(e) => Err(e),
                };
                match result {
                    Ok(_) => 0,
                    Err(e) => {
                        e.restore(py);
                        -1
                    }
                }
            }
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

impl<T> PySequenceContainsProtocolImpl for T
    where T: for<'p> PySequenceContainsProtocol<'p>
{
    #[inline]
    fn sq_contains() -> Option<ffi::objobjproc> {
        py_binary_func!(PySequenceContainsProtocol,
                        T::__contains__, bool, BoolCallbackConverter, c_int)
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

impl<T> PySequenceConcatProtocolImpl for T
    where T: for<'p> PySequenceConcatProtocol<'p>
{
    #[inline]
    fn sq_concat() -> Option<ffi::binaryfunc> {
        py_binary_func!(PySequenceConcatProtocol,
                        T::__concat__, T::Success, PyObjectCallbackConverter)
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

impl<T> PySequenceRepeatProtocolImpl for T where T: for<'p> PySequenceRepeatProtocol<'p>
{
    #[inline]
    fn sq_repeat() -> Option<ffi::ssizeargfunc> {
        py_ssizearg_func!(
            PySequenceRepeatProtocol, T::__repeat__, T::Success, PyObjectCallbackConverter)
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

impl<T> PySequenceInplaceConcatProtocolImpl for T
    where T: for<'p> PySequenceInplaceConcatProtocol<'p>
{
    #[inline]
    fn sq_inplace_concat() -> Option<ffi::binaryfunc> {
        py_binary_func!(PySequenceInplaceConcatProtocol,
                        T::__inplace_concat__, T, PyObjectCallbackConverter)
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

impl<T> PySequenceInplaceRepeatProtocolImpl for T
    where T: for<'p> PySequenceInplaceRepeatProtocol<'p>
{
    #[inline]
    fn sq_inplace_repeat() -> Option<ffi::ssizeargfunc> {
        py_ssizearg_func!(PySequenceInplaceRepeatProtocol,
                          T::__inplace_repeat__, T, PyObjectCallbackConverter)
    }
}
