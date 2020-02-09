//! Traits and structs for `#[pyclass]`.
use crate::conversion::{AsPyPointer, FromPyPointer, ToPyObject};
use crate::pyclass::PyClass;
use crate::pyclass_init::PyClassInitializer;
use crate::pyclass_slots::{PyClassDict, PyClassWeakRef};
use crate::type_object::{PyDowncastImpl, PyObjectLayout, PyObjectSizedLayout};
use crate::types::PyAny;
use crate::{ffi, gil, PyErr, PyObject, PyResult, PyTypeInfo, Python};
use std::cell::{Cell, UnsafeCell};
use std::fmt;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// Inner type of `PyCell` without dict slots and reference counter.
#[doc(hidden)]
#[repr(C)]
pub struct PyCellBase<T: PyClass> {
    ob_base: <T::BaseType as PyTypeInfo>::ConcreteLayout,
    value: ManuallyDrop<UnsafeCell<T>>,
}

impl<T: PyClass> PyCellBase<T> {
    fn get(&self) -> &T {
        unsafe { &*self.value.get() }
    }
    fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.value.get() }
    }
}

impl<T: PyClass> PyObjectLayout<T> for PyCellBase<T> {
    const IS_NATIVE_TYPE: bool = false;
    fn get_super_or(&mut self) -> Option<&mut <T::BaseType as PyTypeInfo>::ConcreteLayout> {
        Some(&mut self.ob_base)
    }
    unsafe fn py_init(&mut self, value: T) {
        self.value = ManuallyDrop::new(UnsafeCell::new(value));
    }
    unsafe fn py_drop(&mut self, py: Python) {
        ManuallyDrop::drop(&mut self.value);
        self.ob_base.py_drop(py);
    }
}

/// `Pycell` represents the concrete layout of `T: PyClass` when it is converted
/// to a Python class.
///
/// You can use it to test your `#[pyclass]` correctly works.
///
/// ```
/// # use pyo3::prelude::*;
/// # use pyo3::{py_run, PyCell};
/// #[pyclass]
/// struct Book {
///     #[pyo3(get)]
///     name: &'static str,
///     author: &'static str,
/// }
/// let gil = Python::acquire_gil();
/// let py = gil.python();
/// let book = Book {
///     name: "The Man in the High Castle",
///     author: "Philip Kindred Dick",
/// };
/// let book_cell = PyCell::new_ref(py, book).unwrap();
/// py_run!(py, book_cell, "assert book_cell.name[-6:] == 'Castle'");
/// ```
#[repr(C)]
pub struct PyCell<T: PyClass> {
    ob_base: <T::BaseType as PyTypeInfo>::ConcreteLayout,
    value: ManuallyDrop<UnsafeCell<T>>,
    borrow_flag: BorrowFlag,
    dict: T::Dict,
    weakref: T::WeakRef,
}

impl<T: PyClass> PyCell<T> {
    /// Make new `PyCell` on the Python heap and returns the reference of it.
    pub fn new(py: Python, value: impl Into<PyClassInitializer<T>>) -> PyResult<&Self>
    where
        <T::BaseType as PyTypeInfo>::ConcreteLayout:
            crate::type_object::PyObjectSizedLayout<T::BaseType>,
    {
        unsafe {
            let initializer = value.into();
            let self_ = initializer.create_cell(py)?;
            FromPyPointer::from_owned_ptr_or_err(py, self_ as _)
        }
    }

    pub fn borrow(&self) -> PyRef<'_, T> {
        unsafe {
            unimplemented!()
            // if self.borrow.get() == usize::max_value() {
            //     borrow_fail();
            // }
            // self.borrow.set(self.borrow.get() + 1);
            // Ref {
            //     value: &*self.value.get(),
            //     borrow: &self.borrow,
            // }
        }
    }

    pub fn borrow_mut(&self) -> PyRefMut<'_, T> {
        unsafe {
            unimplemented!()
            // if self.borrow.get() != 0 {
            //     borrow_fail();
            // }
            // self.borrow.set(usize::max_value());
            // RefMut {
            //     value: &mut *self.value.get(),
            //     borrow: &self.borrow,
            // }
        }
    }

    pub unsafe fn try_borrow_unguarded(&self) -> Result<&T, PyBorrowError> {
        unimplemented!()
    }

    pub unsafe fn try_borrow_mut_unguarded(&self) -> Result<&mut T, PyBorrowMutError> {
        unimplemented!()
    }

    pub(crate) unsafe fn internal_new(py: Python) -> PyResult<*mut Self>
    where
        <T::BaseType as PyTypeInfo>::ConcreteLayout:
            crate::type_object::PyObjectSizedLayout<T::BaseType>,
    {
        let base = T::alloc(py);
        if base.is_null() {
            return Err(PyErr::fetch(py));
        }
        let self_ = base as *mut Self;
        (*self_).borrow_flag = BorrowFlag::UNUSED;
        (*self_).dict = T::Dict::new();
        (*self_).weakref = T::WeakRef::new();
        Ok(self_)
    }
}

impl<T: PyClass> PyObjectLayout<T> for PyCell<T> {
    const IS_NATIVE_TYPE: bool = false;
    fn get_super_or(&mut self) -> Option<&mut <T::BaseType as PyTypeInfo>::ConcreteLayout> {
        Some(&mut self.ob_base)
    }
    unsafe fn py_init(&mut self, value: T) {
        self.value = ManuallyDrop::new(UnsafeCell::new(value));
    }
    unsafe fn py_drop(&mut self, py: Python) {
        ManuallyDrop::drop(&mut self.value);
        self.dict.clear_dict(py);
        self.weakref.clear_weakrefs(self.as_ptr(), py);
        self.ob_base.py_drop(py);
    }
}

unsafe impl<'py, T: 'py + PyClass> PyDowncastImpl<'py> for PyCell<T> {
    unsafe fn unchecked_downcast(obj: &PyAny) -> &'py Self {
        &*(obj.as_ptr() as *const Self)
    }
    private_impl! {}
}

impl<T: PyClass> PyObjectSizedLayout<T> for PyCell<T> {}

impl<T: PyClass> AsPyPointer for PyCell<T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        (self as *const _) as *mut _
    }
}

impl<T: PyClass> ToPyObject for &PyCell<T> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
    }
}

impl<T: PyClass> ToPyObject for &mut PyCell<T> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
    }
}

unsafe impl<'p, T> FromPyPointer<'p> for PyCell<T>
where
    T: PyClass,
{
    unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<&'p Self> {
        NonNull::new(ptr).map(|p| &*(gil::register_owned(py, p).as_ptr() as *const PyCell<T>))
    }
    unsafe fn from_borrowed_ptr_or_opt(
        py: Python<'p>,
        ptr: *mut ffi::PyObject,
    ) -> Option<&'p Self> {
        NonNull::new(ptr).map(|p| &*(gil::register_borrowed(py, p).as_ptr() as *const PyCell<T>))
    }
}

pub struct PyRef<'p, T: PyClass> {
    value: &'p PyCellBase<T>,
    flag: &'p Cell<BorrowFlag>,
}

impl<'p, T: PyClass> PyRef<'p, T> {
    fn get_super(&'p self) -> &'p T::BaseType {
        unimplemented!()
    }
}

impl<'p, T: PyClass> Deref for PyRef<'p, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value.get()
    }
}

impl<'p, T: PyClass> Drop for PyRef<'p, T> {
    fn drop(&mut self) {
        self.flag.set(self.flag.get());
    }
}

pub struct PyRefMut<'p, T: PyClass> {
    value: &'p mut PyCellBase<T>,
    flag: &'p Cell<BorrowFlag>,
}

impl<'p, T: PyClass> Deref for PyRefMut<'p, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value.get()
    }
}

impl<'p, T: PyClass> DerefMut for PyRefMut<'p, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }
}

impl<'p, T: PyClass> Drop for PyRefMut<'p, T> {
    fn drop(&mut self) {
        self.flag.set(BorrowFlag::UNUSED);
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct BorrowFlag(usize);

impl BorrowFlag {
    const UNUSED: BorrowFlag = BorrowFlag(0);
    const HAS_MUTABLE_BORROW: BorrowFlag = BorrowFlag(usize::max_value());
    fn decrement(self) -> Self {
        Self(self.0 - 1)
    }
}

pub struct PyBorrowError {
    _private: (),
}

impl fmt::Debug for PyBorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PyBorrowError").finish()
    }
}

impl fmt::Display for PyBorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("Already mutably borrowed", f)
    }
}

pub struct PyBorrowMutError {
    _private: (),
}

impl fmt::Debug for PyBorrowMutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PyBorrowMutError").finish()
    }
}

impl fmt::Display for PyBorrowMutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("Already borrowed", f)
    }
}

pyo3_exception!(PyBorrowError, crate::exceptions::Exception);
pyo3_exception!(PyBorrowMutError, crate::exceptions::Exception);
