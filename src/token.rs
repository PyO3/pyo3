// Copyright (c) 2017-present PyO3 Project and Contributors

use std::rc::Rc;
use std::marker::PhantomData;

use err::PyResult;
use pointers::Py;
use python::Python;
use typeob::{PyTypeInfo, PyObjectAlloc};


pub struct PyToken(PhantomData<Rc<()>>);

impl PyToken {
    pub fn token<'p>(&'p self) -> Python<'p> {
        unsafe { Python::assume_gil_acquired() }
    }
}


#[inline]
pub fn with<'p, T, F>(py: Python<'p>, f: F) -> PyResult<Py<'p, T>>
    where F: FnOnce(PyToken) -> T,
          T: PyTypeInfo + PyObjectAlloc<Type=T>
{
    Py::new(py, f(PyToken(PhantomData)))
}


pub trait PyObjectWithGilToken<'p> : Sized {
    fn gil(&self) -> Python<'p>;
}

pub trait PyObjectWithToken : Sized {
    fn token<'p>(&'p self) -> Python<'p>;
}
