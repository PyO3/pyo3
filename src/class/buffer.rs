// Copyright (c) 2017-present PyO3 Project and Contributors

//! Represent Python Buffer protocol implementation
//!
//! For more information check [buffer protocol](https://docs.python.org/3/c-api/buffer.html)
//! c-api
use crate::err::PyResult;
use crate::{
    ffi::{self, PyBufferProcs},
    PyCell, PyClass, PyRefMut,
};
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

impl PyBufferProcs {
    pub fn set_getbuffer<T>(&mut self)
    where
        T: for<'p> PyBufferGetBufferProtocol<'p>,
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
                T::bf_getbuffer(slf.try_borrow_mut()?, arg1, arg2).into()
            })
        }
        self.bf_getbuffer = Some(wrap::<T>);
    }
    pub fn set_releasebuffer<T>(&mut self)
    where
        T: for<'p> PyBufferReleaseBufferProtocol<'p>,
    {
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject, arg1: *mut ffi::Py_buffer)
        where
            T: for<'p> PyBufferReleaseBufferProtocol<'p>,
        {
            crate::callback_body!(py, {
                let slf = py.from_borrowed_ptr::<crate::PyCell<T>>(slf);
                T::bf_releasebuffer(slf.try_borrow_mut()?, arg1).into()
            })
        }
        self.bf_releasebuffer = Some(wrap::<T>);
    }
}
