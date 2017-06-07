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
pub fn init<'p, T, F>(py: Python<'p>, f: F) -> PyResult<T::Target>
    where F: FnOnce(PyToken) -> T,
          T: ToInstancePtr<T> + PyTypeInfo + PyObjectAlloc<Type=T>
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

pub trait ToInstancePtr<T> : Sized {
    type Target: InstancePtr<T> + IntoPyPointer;

    fn to_inst_ptr(&self) -> Self::Target;

    unsafe fn from_owned_ptr(*mut ffi::PyObject) -> Self::Target;

    unsafe fn from_borrowed_ptr(*mut ffi::PyObject) -> Self::Target;

}

pub trait InstancePtr<T> : Sized {

    fn as_ref(&self, py: Python) -> &T;

    fn as_mut(&self, py: Python) -> &mut T;

    fn with<F, R>(&self, f: F) -> R where F: FnOnce(Python, &T) -> R
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        f(py, self.as_ref(py))
    }

    fn with_mut<F, R>(&self, f: F) -> R where F: FnOnce(Python, &mut T) -> R
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        f(py, self.as_mut(py))
    }

    fn into_py<F, R>(self, f: F) -> R
        where Self: IntoPyPointer, F: FnOnce(Python, &T) -> R
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let result = f(py, self.as_ref(py));
        py.release(self);
        result
    }

    fn into_mut_py<F, R>(self, f: F) -> R
        where Self: IntoPyPointer, F: FnOnce(Python, &mut T) -> R
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let result = f(py, self.as_mut(py));
        py.release(self);
        result
    }
}
