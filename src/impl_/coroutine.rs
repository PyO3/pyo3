use std::{
    future::Future,
    ops::{Deref, DerefMut},
};

use crate::{
    coroutine::{cancel::ThrowCallback, Coroutine},
    instance::Bound,
    pycell::{
        borrow_checker::PyClassBorrowChecker,
        layout::{PyObjectLayout, TypeObjectStrategy},
    },
    pyclass::boolean_struct::False,
    types::{PyAnyMethods, PyString},
    IntoPyObject, Py, PyAny, PyClass, PyErr, PyResult, Python,
};

use super::pycell::GetBorrowChecker;

pub fn new_coroutine<'py, F, T, E>(
    name: &Bound<'py, PyString>,
    qualname_prefix: Option<&'static str>,
    throw_callback: Option<ThrowCallback>,
    future: F,
) -> Coroutine
where
    F: Future<Output = Result<T, E>> + Send + 'static,
    T: IntoPyObject<'py>,
    E: Into<PyErr>,
{
    Coroutine::new(Some(name.clone()), qualname_prefix, throw_callback, future)
}

pub struct RefGuard<T: PyClass>(Py<T>);

impl<T: PyClass> RefGuard<T> {
    pub fn new(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let bound = obj.downcast::<T>()?;
        // SAFETY: can assume the type object for `T` is initialized because an instance (`obj`) has been created.
        let strategy = unsafe { TypeObjectStrategy::assume_init() };
        let borrow_checker = T::PyClassMutability::borrow_checker(obj.as_raw_ref(), strategy);
        borrow_checker.try_borrow()?;
        Ok(RefGuard(bound.clone().unbind()))
    }
}

impl<T: PyClass> Deref for RefGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: `RefGuard` has been built from `PyRef` and provides the same guarantees
        unsafe {
            PyObjectLayout::get_data::<T>(self.0.as_raw_ref(), TypeObjectStrategy::assume_init())
        }
    }
}

impl<T: PyClass> Drop for RefGuard<T> {
    fn drop(&mut self) {
        Python::with_gil(|py| {
            // SAFETY: `self.0` contains an object that is an instance of `T`
            let borrow_checker =
                unsafe { PyObjectLayout::get_borrow_checker::<T>(py, self.0.as_raw_ref()) };
            borrow_checker.release_borrow();
        })
    }
}

pub struct RefMutGuard<T: PyClass<Frozen = False>>(Py<T>);

impl<T: PyClass<Frozen = False>> RefMutGuard<T> {
    pub fn new(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let bound = obj.downcast::<T>()?;
        // SAFETY: can assume the type object for `T` is initialized because an instance (`obj`) has been created.
        let strategy = unsafe { TypeObjectStrategy::assume_init() };
        let borrow_checker = T::PyClassMutability::borrow_checker(obj.as_raw_ref(), strategy);
        borrow_checker.try_borrow_mut()?;
        Ok(RefMutGuard(bound.clone().unbind()))
    }
}

impl<T: PyClass<Frozen = False>> Deref for RefMutGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: `RefMutGuard` has been built from `PyRefMut` and provides the same guarantees
        unsafe {
            PyObjectLayout::get_data::<T>(self.0.as_raw_ref(), TypeObjectStrategy::assume_init())
        }
    }
}

impl<T: PyClass<Frozen = False>> DerefMut for RefMutGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: `RefMutGuard` has been built from `PyRefMut` and provides the same guarantees
        unsafe {
            &mut *PyObjectLayout::get_data_ptr::<T>(
                self.0.as_ptr(),
                TypeObjectStrategy::assume_init(),
            )
        }
    }
}

impl<T: PyClass<Frozen = False>> Drop for RefMutGuard<T> {
    fn drop(&mut self) {
        Python::with_gil(|py| {
            // SAFETY: `self.0` contains an object that is an instance of `T`
            let borrow_checker =
                unsafe { PyObjectLayout::get_borrow_checker::<T>(py, self.0.as_raw_ref()) };
            borrow_checker.release_borrow_mut();
        })
    }
}
