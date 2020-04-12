// Copyright (c) 2017-present PyO3 Project and Contributors

//! Represent Python Buffer protocol implementation
//!
//! For more information check [buffer protocol](https://docs.python.org/3/c-api/buffer.html)
//! c-api
use crate::err::PyResult;
use crate::gil::GILPool;
use crate::{callback, ffi, run_callback, PyCell, PyClass, PyRefMut};
use std::os::raw::c_int;

/// Buffer protocol interface
///
/// For more information check [buffer protocol](https://docs.python.org/3/c-api/buffer.html)
/// c-api
#[allow(unused_variables)]
pub trait PyBufferProtocol<'p>: PyClass {
    fn bf_getbuffer(slf: PyRefMut<Self>, view: *mut ffi::Py_buffer, flags: c_int) -> Self::Result
    where
        Self: PyBufferGetBufferProtocol<'p>,
    {
        unimplemented!()
    }

    fn bf_releasebuffer(slf: PyRefMut<Self>, view: *mut ffi::Py_buffer) -> Self::Result
    where
        Self: PyBufferReleaseBufferProtocol<'p>,
    {
        unimplemented!()
    }
}

pub trait PyBufferGetBufferProtocol<'p>: PyBufferProtocol<'p> {
    type Result: Into<PyResult<()>>;
}

pub trait PyBufferReleaseBufferProtocol<'p>: PyBufferProtocol<'p> {
    type Result: Into<PyResult<()>>;
}

#[doc(hidden)]
pub trait PyBufferProtocolImpl {
    fn tp_as_buffer() -> Option<ffi::PyBufferProcs>;
}

impl<T> PyBufferProtocolImpl for T {
    default fn tp_as_buffer() -> Option<ffi::PyBufferProcs> {
        None
    }
}

impl<'p, T> PyBufferProtocolImpl for T
where
    T: PyBufferProtocol<'p>,
{
    #[inline]
    #[allow(clippy::needless_update)] // For python 2 it's not useless
    fn tp_as_buffer() -> Option<ffi::PyBufferProcs> {
        Some(ffi::PyBufferProcs {
            bf_getbuffer: Self::cb_bf_getbuffer(),
            bf_releasebuffer: Self::cb_bf_releasebuffer(),
            ..ffi::PyBufferProcs_INIT
        })
    }
}

trait PyBufferGetBufferProtocolImpl {
    fn cb_bf_getbuffer() -> Option<ffi::getbufferproc>;
}

impl<'p, T> PyBufferGetBufferProtocolImpl for T
where
    T: PyBufferProtocol<'p>,
{
    default fn cb_bf_getbuffer() -> Option<ffi::getbufferproc> {
        None
    }
}

impl<T> PyBufferGetBufferProtocolImpl for T
where
    T: for<'p> PyBufferGetBufferProtocol<'p>,
{
    #[inline]
    fn cb_bf_getbuffer() -> Option<ffi::getbufferproc> {
        unsafe extern "C" fn wrap<T>(
            slf: *mut ffi::PyObject,
            arg1: *mut ffi::Py_buffer,
            arg2: c_int,
        ) -> c_int
        where
            T: for<'p> PyBufferGetBufferProtocol<'p>,
        {
            let pool = GILPool::new();
            let py = pool.python();
            run_callback(py, || {
                let slf = py.from_borrowed_ptr::<PyCell<T>>(slf);
                let result = T::bf_getbuffer(slf.try_borrow_mut()?, arg1, arg2).into();
                callback::convert(py, result)
            })
        }
        Some(wrap::<T>)
    }
}

trait PyBufferReleaseBufferProtocolImpl {
    fn cb_bf_releasebuffer() -> Option<ffi::releasebufferproc>;
}

impl<'p, T> PyBufferReleaseBufferProtocolImpl for T
where
    T: PyBufferProtocol<'p>,
{
    default fn cb_bf_releasebuffer() -> Option<ffi::releasebufferproc> {
        None
    }
}

impl<T> PyBufferReleaseBufferProtocolImpl for T
where
    T: for<'p> PyBufferReleaseBufferProtocol<'p>,
{
    #[inline]
    fn cb_bf_releasebuffer() -> Option<ffi::releasebufferproc> {
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject, arg1: *mut ffi::Py_buffer)
        where
            T: for<'p> PyBufferReleaseBufferProtocol<'p>,
        {
            let pool = GILPool::new();
            let py = pool.python();
            run_callback(py, || {
                let slf = py.from_borrowed_ptr::<crate::PyCell<T>>(slf);
                let result = T::bf_releasebuffer(slf.try_borrow_mut()?, arg1).into();
                crate::callback::convert(py, result)
            })
        }
        Some(wrap::<T>)
    }
}
