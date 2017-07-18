// Copyright (c) 2017-present PyO3 Project and Contributors

//! Represent Python Buffer protocol implementation
//!
//! more information on buffer protocol can be found
//! https://docs.python.org/3/c-api/buffer.html

use std::os::raw::c_int;

use ffi;
use err::PyResult;
use typeob::PyTypeInfo;
use python::PyDowncastFrom;
use callback::UnitCallbackConverter;


/// Buffer protocol interface
///
/// more information on buffer protocol can be found
/// https://docs.python.org/3/c-api/buffer.html
#[allow(unused_variables)]
pub trait PyBufferProtocol<'p> : PyTypeInfo + PyDowncastFrom
{
    fn bf_getbuffer(&'p self,
                    view: *mut ffi::Py_buffer, flags: c_int) -> Self::Result
        where Self: PyBufferGetBufferProtocol<'p> { unimplemented!() }

    fn bf_releasebuffer(&'p self, view: *mut ffi::Py_buffer) -> Self::Result
        where Self: PyBufferReleaseBufferProtocol<'p> { unimplemented!() }
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
    default fn tp_as_buffer() -> Option<ffi::PyBufferProcs> { None }
}

impl<'p, T> PyBufferProtocolImpl for T where T: PyBufferProtocol<'p>
{
    #[inline]
    fn tp_as_buffer() -> Option<ffi::PyBufferProcs> {
        Some(ffi::PyBufferProcs{
            bf_getbuffer: Self::cb_bf_getbuffer(),
            bf_releasebuffer: None,
            .. ffi::PyBufferProcs_INIT
        })
    }
}

trait PyBufferGetBufferProtocolImpl {
    fn cb_bf_getbuffer() -> Option<ffi::getbufferproc>;
}

impl<'p, T> PyBufferGetBufferProtocolImpl for T where T: PyBufferProtocol<'p>
{
    #[inline]
    default fn cb_bf_getbuffer() -> Option<ffi::getbufferproc> {
        None
    }
}

impl<T> PyBufferGetBufferProtocolImpl for T
    where T: for<'p> PyBufferGetBufferProtocol<'p>
{
    #[inline]
    fn cb_bf_getbuffer() -> Option<ffi::getbufferproc> {
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                     arg1: *mut ffi::Py_buffer,
                                     arg2: c_int) -> c_int
            where T: for<'p> PyBufferGetBufferProtocol<'p>
        {
            const LOCATION: &'static str = concat!(stringify!(T), ".buffer_get::<PyBufferProtocol>()");
            ::callback::cb_unary::<T, _, _, _>(LOCATION, slf, UnitCallbackConverter, |_, slf| {
                slf.bf_getbuffer(arg1, arg2).into()
            })
        }
        Some(wrap::<T>)
    }
}
