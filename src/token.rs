// Copyright (c) 2017-present PyO3 Project and Contributors

use std::rc::Rc;
use std::marker::PhantomData;

use ffi;
use err::PyResult;
use python::{Python, IntoPyPointer};
use typeob::{PyTypeInfo, PyObjectAlloc};


pub struct PyToken(PhantomData<Rc<()>>);

impl PyToken {
    pub fn token<'p>(&'p self) -> Python<'p> {
        unsafe { Python::assume_gil_acquired() }
    }
}

/// Create new python object and move T instance under python management
#[inline]
pub fn with<'p, T, F>(py: Python<'p>, f: F) -> PyResult<T::ParkTarget>
    where F: FnOnce(PyToken) -> T,
          T: Park<T> + PyTypeInfo + PyObjectAlloc<Type=T>
{
    let ob = f(PyToken(PhantomData));

    let ob = unsafe {
        let ob = try!(<T as PyObjectAlloc>::alloc(py, ob));
        T::from_owned_ptr(ob)
    };
    Ok(ob)
}

pub trait PyObjectWithToken : Sized {
    fn token<'p>(&'p self) -> Python<'p>;
}


pub trait Park<T> : Sized {
    type ParkTarget: PythonPtr<T> + IntoPyPointer;

    fn park(&self) -> Self::ParkTarget;

    unsafe fn from_owned_ptr(*mut ffi::PyObject) -> Self::ParkTarget;

    unsafe fn from_borrowed_ptr(*mut ffi::PyObject) -> Self::ParkTarget;

}

pub trait PythonPtr<T> : Sized {

    fn as_ref(&self, py: Python) -> &T;

    fn as_mut(&self, py: Python) -> &mut T;

}
