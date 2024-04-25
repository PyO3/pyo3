use std::ops::{Deref, DerefMut};

use crate::{
    instance::Bound, pycell::impl_::PyClassBorrowChecker, pyclass::boolean_struct::False,
    types::PyAnyMethods, Py, PyAny, PyClass, PyResult, Python,
};

fn get_ptr<T: PyClass>(obj: &Py<T>) -> *mut T {
    obj.get_class_object().get_ptr()
}

pub struct RefGuard<T: PyClass>(Py<T>);

impl<T: PyClass> RefGuard<T> {
    pub fn new(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let bound = obj.downcast::<T>()?;
        bound.get_class_object().borrow_checker().try_borrow()?;
        Ok(RefGuard(bound.clone().unbind()))
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
        Python::with_gil(|gil| {
            self.0
                .bind(gil)
                .get_class_object()
                .borrow_checker()
                .release_borrow()
        })
    }
}

pub struct RefMutGuard<T: PyClass<Frozen = False>>(Py<T>);

impl<T: PyClass<Frozen = False>> RefMutGuard<T> {
    pub fn new(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let bound = obj.downcast::<T>()?;
        bound.get_class_object().borrow_checker().try_borrow_mut()?;
        Ok(RefMutGuard(bound.clone().unbind()))
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
        Python::with_gil(|gil| {
            self.0
                .bind(gil)
                .get_class_object()
                .borrow_checker()
                .release_borrow_mut()
        })
    }
}
