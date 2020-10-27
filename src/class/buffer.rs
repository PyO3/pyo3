// Copyright (c) 2017-present PyO3 Project and Contributors

//! Represent Python Buffer protocol implementation
//!
//! For more information check [buffer protocol](https://docs.python.org/3/c-api/buffer.html)
//! c-api
use crate::callback::IntoPyCallbackOutput;
use crate::{ffi, PyCell, PyClass, PyRefMut};
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
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyBufferReleaseBufferProtocol<'p>: PyBufferProtocol<'p> {
    type Result: IntoPyCallbackOutput<()>;
}

/// Extension trait for proc-macro backend.
#[doc(hidden)]
pub trait PyBufferSlots {
    fn get_getbuffer() -> ffi::getbufferproc
    where
        Self: for<'p> PyBufferGetBufferProtocol<'p>,
    {
        unsafe extern "C" fn wrap<T>(
            slf: *mut ffi::PyObject,
            arg1: *mut ffi::Py_buffer,
            arg2: c_int,
        ) -> c_int
        where
            T: for<'p> PyBufferGetBufferProtocol<'p>,
        {
            crate::callback_body!(py, {
                let slf = py.from_borrowed_ptr::<PyCell<T>>(slf);
                T::bf_getbuffer(slf.try_borrow_mut()?, arg1, arg2).convert(py)
            })
        }

        wrap::<Self>
    }

    fn get_releasebuffer() -> ffi::releasebufferproc
    where
        Self: for<'p> PyBufferReleaseBufferProtocol<'p>,
    {
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject, arg1: *mut ffi::Py_buffer)
        where
            T: for<'p> PyBufferReleaseBufferProtocol<'p>,
        {
            crate::callback_body!(py, {
                let slf = py.from_borrowed_ptr::<crate::PyCell<T>>(slf);
                T::bf_releasebuffer(slf.try_borrow_mut()?, arg1).convert(py)
            })
        }

        wrap::<Self>
    }
}

impl<'p, T> PyBufferSlots for T where T: PyBufferProtocol<'p> {}
