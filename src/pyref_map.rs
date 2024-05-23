#![allow(unused_imports, dead_code)]

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::convert::{AsRef, AsMut};

use crate::prelude::*;
use crate::pyclass::PyClass;
use crate::pyclass::boolean_struct::False;


trait OpaquePyRef<'py, T: PyClass>: 'py {}

impl<'py, T: PyClass> OpaquePyRef<'py, T> for PyRef<'py, T> {}
impl<'py, T: PyClass<Frozen=False>> OpaquePyRef<'py, T> for PyRefMut<'py, T> {}


pub struct PyRefMap<'py, T: PyClass, U: 'py> {
    owner: Box<dyn OpaquePyRef<'py, T>>,
    target: *const U,
}

impl<'py, T: PyClass> PyRef<'py, T> {
    pub fn into_map<F, U: 'py>(self, f: F) -> PyRefMap<'py, T, U>
        where F: FnOnce(&T) -> &U
    {
        PyRefMap {target: f(&*self), owner: Box::new(self)}
    }
    
    pub fn try_into_map<F, U: 'py, E>(self, f: F) -> Result<PyRefMap<'py, T, U>, E>
        where F: FnOnce(&T) -> Result<&U, E>
    {
        Ok(PyRefMap {target: f(&*self)?, owner: Box::new(self)})
    }
}

impl<'py, T: PyClass, U: 'py> Deref for PyRefMap<'py, T, U> {
    type Target = U;
    fn deref(&self) -> &U {
        // we own the `PyRef` that is guarding our shared access to `T`
        unsafe { &*self.target }
    }
}


pub struct PyRefMapMut<'py, T: PyClass<Frozen = False>, U: 'py> {
    owner: PyRefMut<'py, T>,
    target: *mut U,
}

impl<'py, T: PyClass<Frozen = False>> PyRefMut<'py, T> {
    
    pub fn into_map<F, U: 'py>(self, f: F) -> PyRefMap<'py, T, U>
        where F: FnOnce(&T) -> &U
    {
        PyRefMap {target: f(&*self), owner: Box::new(self)}
    }
    
    pub fn into_map_mut<F, U: 'py>(mut self, f: F) -> PyRefMapMut<'py, T, U>
        where F: FnOnce(&mut T) -> &mut U
    {
        PyRefMapMut {target: f(&mut *self), owner: self}
    }
}

impl<'py, U: 'py, T: PyClass<Frozen = False>> Deref for PyRefMapMut<'py, T, U> {
    type Target = U;
    fn deref(&self) -> &U {
        // we own the `PyRef` that is guarding our access to `T`
        unsafe { &*self.target }
    }
}

impl<'py, U: 'py, T: PyClass<Frozen = False>> DerefMut for PyRefMapMut<'py, T, U> {
    fn deref_mut(&mut self) -> &mut U {
        // we own the `PyRef` that is guarding our exclusive access to `T`
        unsafe { &mut *self.target }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PyString;
    
    #[pyclass]
    #[pyo3(crate = "crate")]
    pub struct MyClass {
        data: [i32; 100]
    }

    #[test]
    fn pyref_map() -> PyResult<()> {
        Python::with_gil(|py| -> PyResult<()> {
            let bound = Bound::new(py, MyClass{data: [0; 100]})?;
            let data = bound.try_borrow()?.into_map(|c| &c.data);
            assert_eq!(data[0], 0);
            Ok(())
        })
    }

    #[test]
    fn pyrefmut_map() -> PyResult<()> {
        Python::with_gil(|py| -> PyResult<()> {
            let bound = Bound::new(py, MyClass{data: [0; 100]})?;
            let data = bound.try_borrow_mut()?.into_map(|c| &c.data);
            assert_eq!(data[0], 0);
            Ok(())
        })
    }

    #[test]
    fn pyrefmut_map_mut() -> PyResult<()> {
        Python::with_gil(|py| -> PyResult<()> {
            let bound = Bound::new(py, MyClass{data: [0; 100]})?;
            let mut data = bound
                .try_borrow_mut()?
                .into_map_mut(|c| &mut c.data);
            data[0] = 5;
            assert_eq!(data[0], 5);
            Ok(())
        })
    }

    #[test]
    fn pyref_map_unrelated() -> PyResult<()> {
        Python::with_gil(|py| -> PyResult<()> {
            let bound = Bound::new(py, MyClass{data: [0; 100]})?;
            let string = PyString::new_bound(py, "pyo3");
            // there is nothing stopping the user from returning something not
            // borrowing from the pyref, but that shouldn't matter. The borrow 
            // checker still enforces the `'py` lifetime
            let refmap = bound.try_borrow()?.into_map(|_| &string);
            assert_eq!(refmap.to_str()?, "pyo3");
            Ok(())
        })
    }
}
