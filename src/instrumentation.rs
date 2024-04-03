use crate::ffi;
use crate::pyclass::boolean_struct::False;
use crate::types::PyFrame;
use crate::{Bound, PyAny, PyClass, PyObject, PyRefMut, PyResult, Python};
use std::ffi::c_int;

pub trait Event<'py>: Sized {
    fn from_raw(what: c_int, arg: Option<Bound<'py, PyAny>>) -> PyResult<Self>;
}

pub enum ProfileEvent<'py> {
    Call,
    Return(Option<Bound<'py, PyAny>>),
    CCall(Bound<'py, PyAny>),
    CException(Bound<'py, PyAny>),
    CReturn(Bound<'py, PyAny>),
}

impl<'py> Event<'py> for ProfileEvent<'py> {
    fn from_raw(what: c_int, arg: Option<Bound<'py, PyAny>>) -> PyResult<ProfileEvent<'py>> {
        let event = match what {
            ffi::PyTrace_CALL => ProfileEvent::Call,
            ffi::PyTrace_RETURN => ProfileEvent::Return(arg),
            ffi::PyTrace_C_CALL => ProfileEvent::CCall(arg.unwrap()),
            ffi::PyTrace_C_EXCEPTION => ProfileEvent::CException(arg.unwrap()),
            ffi::PyTrace_C_RETURN => ProfileEvent::CReturn(arg.unwrap()),
            _ => unreachable!(),
        };
        Ok(event)
    }
}

pub trait Profiler: PyClass<Frozen = False> {
    fn profile<'py>(
        &mut self,
        frame: Bound<'py, PyFrame>,
        event: ProfileEvent<'py>,
    ) -> PyResult<()>;
}

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

        let event = ProfileEvent::from_raw(what, arg).unwrap();

        match profiler.profile(frame, event) {
            Ok(_) => 0,
            Err(err) => {
                err.restore(py);
                -1
            }
        }
    })
}
