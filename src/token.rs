// Copyright (c) 2017-present PyO3 Project and Contributors

use std::rc::Rc;
use std::marker::PhantomData;

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
pub fn with_token<'p, T, F>(py: Python<'p>, f: F) -> Py<'p, T>
    where F: FnOnce(PyToken) -> T,
          T: PyTypeInfo + PyObjectAlloc<Type=T>
{
    let value = f(PyToken(PhantomData));
    if let Ok(ob) = Py::new(py, value) {
        ob
    } else {
        ::err::panic_after_error()
    }
}


pub trait PyObjectWithGilToken<'p> : Sized {
    fn gil(&self) -> Python<'p>;
}

pub trait PyObjectWithToken : Sized {
    fn token<'p>(&'p self) -> Python<'p>;
}
