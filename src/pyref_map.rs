
use std::ptr::NonNull;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::pyclass::PyClass;
use crate::pycell::{PyRef, PyRefMut};
use crate::pyclass::boolean_struct::{True, False, private::Boolean};


/// Represents a `PyRef` or `PyRefMut` with an opaque pyclass type.
trait OpaquePyRef<'py>: 'py {}

impl<'py, T: PyClass> OpaquePyRef<'py> for PyRef<'py, T> {}
impl<'py, T: PyClass<Frozen=False>> OpaquePyRef<'py> for PyRefMut<'py, T> {}


/// Base wrapper type for a [`PyRef<'py, T>`] or [`PyRefMut<'py, T>`] that 
/// dereferences to data of type `U` that is nested within a pyclass `T` 
/// instead of `T` itself. 
/// 
/// See the type aliases [`PyRefMap<'py, U>`] and [`PyRefMapMut<'py, U>`] 
/// for more information.
pub struct PyRefMapBase<'py, U: 'py, Mut: Boolean> {
    // Either `PyRef` or `PyRefMut` guarding the opaque pyclass from which 
    // the target is borrowed. 
    owner: Box<dyn OpaquePyRef<'py>>,
    // Pointer to the `Deref` target which is (probably) borrowed from `owner`.
    // This pointer is derived from either `&U` or `&mut U` as  indicated by 
    // the `Mut` parameter.
    target: NonNull<U>,
    // Marks whether mutable methods are supported. If `Mut` is `True` then 
    // `owner` is a `PyRefMut` and `target` was derived from `&mut U`, so 
    // the pointer may be mutably dereferenced safely; if `False`, then 
    // `owner` may be either `PyRef` or `PyRefMut` and `target` was derived 
    // from `&U` so mutable dereferencing is forbidden.
    _mut: PhantomData<Mut>
}

/// A wrapper type for an _immutable_ reference to data of type `U` that is 
/// nested within a [`PyRef<'py, T>`] or [`PyRefMut<'py, T>`].
pub type PyRefMap<'py, U> = PyRefMapBase<'py, U, False>;

/// A wrapper type for a _mutable_ reference to data of type `U` that is 
/// nested within a [`PyRefMut<'py, T>`].
pub type PyRefMapMut<'py, U> = PyRefMapBase<'py, U, True>;


impl<'py, T: PyClass> PyRefMap<'py, T> {
    
    /// Construct a no-op `PyRefMap` that dereferences to the same 
    /// value as the given [`PyRef`] or [`PyRefMut`].
    pub fn new<R>(owner: R) -> PyRefMap<'py, T> 
        where R: OpaquePyRef<'py> + Deref<Target=T>,
    {
        let target = NonNull::from(&*owner);
        PyRefMap {target, owner: Box::new(owner), _mut: PhantomData}
    }
}

impl<'py, T: PyClass<Frozen = False>> PyRefMapMut<'py, T> {
    
    /// Construct a no-op `PyRefMapMut` that dereferences to the same 
    /// value as the given [`PyRefMut`].
    pub fn new(mut owner: PyRefMut<'py, T>) -> PyRefMapMut<'py, T> {
        let target = NonNull::from(&mut *owner);
        PyRefMapMut {target, owner: Box::new(owner), _mut: PhantomData}
    }
}

impl<'py, U: 'py, Mut: Boolean> PyRefMapBase<'py, U, Mut> {
    
    /// Applies the given function to the wrapped reference and wrap the 
    /// return value in a new `PyRefMap`.
    pub fn map<F, V>(mut self, f: F) -> PyRefMap<'py, V> 
        where F: FnOnce(&U) -> &V
    {
        let target = NonNull::from(f(&*self));
        PyRefMap {target, owner: self.owner, _mut: PhantomData}
    }
}

impl<'py, U: 'py> PyRefMapMut<'py, U> {
    
    /// Applies the given function to the wrapped mutable reference and 
    /// wrap the return value in a new `PyRefMapMut`.
    pub fn map_mut<F, V>(mut self, f: F) -> PyRefMapMut<'py, V> 
        where F: FnOnce(&mut U) -> &mut V
    {
        let target = NonNull::from(f(&mut *self));
        PyRefMapMut {target, owner: self.owner, _mut: PhantomData}
    }
}

// either flavor can safely implement `Deref`
impl<'py, U: 'py, Mut: Boolean> Deref for PyRefMapBase<'py, U, Mut> {
    type Target = U;
    fn deref(&self) -> &U {
        // we own the `PyRef` or `PyRefMut` that is guarding our access to `T`
        unsafe { self.target.as_ref() }
    }
}

// only the `Mut=True` flavor can safely implement `DerefMut`
impl<'py, U: 'py> DerefMut for PyRefMapMut<'py, U> {
    fn deref_mut(&mut self) -> &mut U {
        // we own the `PyRefMut` that is guarding our exclusive access to `T`
        unsafe { self.target.as_mut() }
    }
}

impl<'py, T: PyClass> PyRef<'py, T> {
    pub fn into_map<F, U: 'py>(self, f: F) -> PyRefMap<'py, U>
        where F: FnOnce(&T) -> &U
    {
        PyRefMap::new(self).map(f)
    }
}

impl<'py, T: PyClass<Frozen = False>> PyRefMut<'py, T> {
    
    pub fn into_map<F, U: 'py>(self, f: F) -> PyRefMap<'py, U>
        where F: FnOnce(&T) -> &U
    {
        PyRefMap::new(self).map(f)
    }
    
    pub fn into_map_mut<F, U: 'py>(self, f: F) -> PyRefMapMut<'py, U>
        where F: FnOnce(&mut T) -> &mut U
    {
        PyRefMapMut::new(self).map_mut(f)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
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
