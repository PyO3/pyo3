// Copyright (c) 2017-present PyO3 Project and Contributors

//! Represent Python Buffer protocol implementation
//!
//! more information on buffer protocol can be found
//! https://docs.python.org/3/c-api/buffer.html

use std::os::raw::c_int;

use ffi;
use err::PyResult;
use objects::PyObject;
use typeob::PyTypeInfo;
use callback::{handle, UnitCallbackConverter};
use class::NO_METHODS;


/// Buffer protocol interface
pub trait PyBufferProtocol<'p> : PyTypeInfo {

    fn bf_getbuffer(&self, view: *mut ffi::Py_buffer, flags: c_int) -> PyResult<()>;

    fn bf_releasebuffer(&self, view: *mut ffi::Py_buffer) -> PyResult<()>;
}

#[doc(hidden)]
pub trait PyBufferProtocolImpl {
    fn methods() -> &'static [&'static str];
}

impl<T> PyBufferProtocolImpl for T {
    default fn methods() -> &'static [&'static str] {
        NO_METHODS
    }
}

impl<'p, T> PyBufferProtocol<'p> for T where T: PyTypeInfo {

    default fn bf_getbuffer(&self, _view: *mut ffi::Py_buffer, _flags: c_int) -> PyResult<()> {
        Ok(())
    }
    default fn bf_releasebuffer(&self, _view: *mut ffi::Py_buffer) -> PyResult<()> {
        Ok(())
    }
}


impl ffi::PyBufferProcs {

    /// Construct PyBufferProcs struct for PyTypeObject.tp_as_buffer
    pub fn new<'p, T>() -> Option<ffi::PyBufferProcs>
        where T: PyBufferProtocol<'p> + PyBufferProtocolImpl
    {
        let methods = T::methods();
        if methods.is_empty() {
            return None
        }

        let mut buf_procs: ffi::PyBufferProcs = ffi::PyBufferProcs_INIT;

        for name in methods {
            match name {
                &"bf_getbuffer" => {
                    buf_procs.bf_getbuffer = {
                        unsafe extern "C" fn wrap<'p, T>(slf: *mut ffi::PyObject,
                                                     arg1: *mut ffi::Py_buffer,
                                                     arg2: c_int) -> c_int
                            where T: PyBufferProtocol<'p>
                        {
                            const LOCATION: &'static str = concat!(stringify!(T), ".buffer_get::<PyBufferProtocol>()");
                            handle(LOCATION, UnitCallbackConverter, |py| {
                                let slf = PyObject::from_borrowed_ptr(py, slf);
                                slf.bf_getbuffer(arg1, arg2)
                            })
                        }
                        Some(wrap::<T>)
                    }
                },
                _ => ()
            }
        }

        Some(buf_procs)
    }
}
