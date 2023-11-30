use std::{
    future::Future,
    mem,
    ops::{Deref, DerefMut},
};

use crate::{
    coroutine::{cancel::ThrowCallback, Coroutine},
    pyclass::boolean_struct::False,
    types::PyString,
    IntoPy, Py, PyAny, PyCell, PyClass, PyErr, PyObject, PyResult, Python,
};

pub fn new_coroutine<F, T, E>(
    name: &PyString,
    qualname_prefix: Option<&'static str>,
    throw_callback: Option<ThrowCallback>,
    future: F,
) -> Coroutine
where
    F: Future<Output = Result<T, E>> + Send + 'static,
    T: IntoPy<PyObject>,
    E: Into<PyErr>,
{
    Coroutine::new(Some(name.into()), qualname_prefix, throw_callback, future)
}

fn get_ptr<T: PyClass>(obj: &Py<T>) -> *mut T {
    // SAFETY: Py<T> can be casted as *const PyCell<T>
    unsafe { &*(obj.as_ptr() as *const PyCell<T>) }.get_ptr()
}

pub struct RefGuard<T: PyClass>(Py<T>);

impl<T: PyClass> RefGuard<T> {
    pub fn new(obj: &PyAny) -> PyResult<Self> {
        let owned: Py<T> = obj.extract()?;
        mem::forget(owned.try_borrow(obj.py())?);
        Ok(RefGuard(owned))
    }
}

impl<T: PyClass> Deref for RefGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: `RefGuard` has been built from `PyRef` and provides the same guarantees
        unsafe { &*get_ptr(&self.0) }
    }
}

impl<T: PyClass> Drop for RefGuard<T> {
    fn drop(&mut self) {
        Python::with_gil(|gil| self.0.as_ref(gil).release_ref())
    }
}

pub struct RefMutGuard<T: PyClass<Frozen = False>>(Py<T>);

impl<T: PyClass<Frozen = False>> RefMutGuard<T> {
    pub fn new(obj: &PyAny) -> PyResult<Self> {
        let owned: Py<T> = obj.extract()?;
        mem::forget(owned.try_borrow_mut(obj.py())?);
        Ok(RefMutGuard(owned))
    }
}

impl<T: PyClass<Frozen = False>> Deref for RefMutGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: `RefMutGuard` has been built from `PyRefMut` and provides the same guarantees
        unsafe { &*get_ptr(&self.0) }
    }
}

impl<T: PyClass<Frozen = False>> DerefMut for RefMutGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: `RefMutGuard` has been built from `PyRefMut` and provides the same guarantees
        unsafe { &mut *get_ptr(&self.0) }
    }
}

impl<T: PyClass<Frozen = False>> Drop for RefMutGuard<T> {
    fn drop(&mut self) {
        Python::with_gil(|gil| self.0.as_ref(gil).release_mut())
    }
}
