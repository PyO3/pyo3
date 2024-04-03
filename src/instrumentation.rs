//! APIs wrapping the Python interpreter's instrumentation features.
use crate::ffi;
use crate::pyclass::boolean_struct::False;
use crate::types::PyFrame;
use crate::{Bound, PyAny, PyClass, PyObject, PyRefMut, PyResult, Python};
use std::ffi::c_int;

/// Represents a monitoring event used by the profiling API
pub enum ProfileEvent<'py> {
    /// A python function or method was called or a generator was entered.
    Call,
    /// A python function or method returned or a generator yielded. The
    /// contained data is the value returned to the caller or `None` if
    /// caused by an exception.
    Return(Option<Bound<'py, PyAny>>),
    /// A C function is about to be called. The contained data is the
    /// function object being called.
    CCall(Bound<'py, PyAny>),
    /// A C function has raised an exception. The contained data is the
    /// function object being called.
    CException(Bound<'py, PyAny>),
    /// A C function has returned. The contained data is the function
    /// object being called.
    CReturn(Bound<'py, PyAny>),
}

impl<'py> ProfileEvent<'py> {
    fn from_raw(what: c_int, arg: Option<Bound<'py, PyAny>>) -> ProfileEvent<'py> {
        match what {
            ffi::PyTrace_CALL => ProfileEvent::Call,
            ffi::PyTrace_RETURN => ProfileEvent::Return(arg),
            ffi::PyTrace_C_CALL => ProfileEvent::CCall(arg.unwrap()),
            ffi::PyTrace_C_EXCEPTION => ProfileEvent::CException(arg.unwrap()),
            ffi::PyTrace_C_RETURN => ProfileEvent::CReturn(arg.unwrap()),
            _ => unreachable!(),
        }
    }
}

/// Trait for Rust structs that can be used with Python's profiling API.
pub trait Profiler: PyClass<Frozen = False> {
    /// Callback for implementing custom profiling logic.
    fn profile<'py>(
        &mut self,
        frame: Bound<'py, PyFrame>,
        event: ProfileEvent<'py>,
    ) -> PyResult<()>;
}

/// Register a custom Profiler with the Python interpreter.
pub fn register_profiler<P: Profiler>(profiler: Bound<'_, P>) {
    unsafe { ffi::PyEval_SetProfile(Some(profile_callback::<P>), profiler.into_ptr()) };
}

extern "C" fn profile_callback<P>(
    obj: *mut ffi::PyObject,
    frame: *mut ffi::PyFrameObject,
    what: c_int,
    arg: *mut ffi::PyObject,
) -> c_int
where
    P: Profiler,
{
    // Safety:
    //
    // `frame` is an `ffi::PyFrameObject` which can be converted safely to a `PyObject`.
    let frame = frame as *mut ffi::PyObject;
    Python::with_gil(|py| {
        // Safety:
        //
        // `obj` is a reference to our `Profiler` wrapped up in a Python object, so
        // we can safely convert it from an `ffi::PyObject` to a `PyObject`.
        //
        // We borrow the object so we don't break reference counting.
        //
        // https://docs.python.org/3/c-api/init.html#c.Py_tracefunc
        let obj = unsafe { PyObject::from_borrowed_ptr(py, obj) };
        let mut profiler = obj.extract::<PyRefMut<'_, P>>(py).unwrap();

        // Safety:
        //
        // We borrow the object so we don't break reference counting.
        //
        // https://docs.python.org/3/c-api/init.html#c.Py_tracefunc
        let frame = unsafe { PyObject::from_borrowed_ptr(py, frame) };
        let frame = frame.extract(py).unwrap();

        // Safety:
        //
        // `arg` is either a `Py_None` (PyTrace_CALL) or any PyObject (PyTrace_RETURN) or
        // NULL (PyTrace_RETURN).
        //
        // We borrow the object so we don't break reference counting.
        //
        // https://docs.python.org/3/c-api/init.html#c.Py_tracefunc
        let arg = unsafe { Bound::from_borrowed_ptr_or_opt(py, arg) };

        let event = ProfileEvent::from_raw(what, arg);

        match profiler.profile(frame, event) {
            Ok(_) => 0,
            Err(err) => {
                err.restore(py);
                -1
            }
        }
    })
}
