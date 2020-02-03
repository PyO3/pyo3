//! Traits and structs for `#[pyclass]`.
use crate::conversion::{AsPyPointer, FromPyPointer, ToPyObject};
use crate::pyclass::PyClass;
use crate::pyclass_init::PyClassInitializer;
use crate::pyclass_slots::{PyClassDict, PyClassWeakRef};
use crate::type_object::{PyObjectLayout, PyObjectSizedLayout};
use crate::types::PyAny;
use crate::{ffi, gil, PyErr, PyObject, PyResult, PyTypeInfo, Python};
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

/// `PyCell` represents the concrete layout of `T: PyClass` when it is converted
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
    pyclass: ManuallyDrop<T>,
    dict: T::Dict,
    weakref: T::WeakRef,
}

impl<T: PyClass> PyCell<T> {
    /// Make new `PyCell` on the Python heap and returns the reference of it.
    pub fn new_ref(py: Python, value: impl Into<PyClassInitializer<T>>) -> PyResult<&Self>
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

    /// Make new `PyCell` on the Python heap and returns the mutable reference of it.
    pub fn new_mut(py: Python, value: impl Into<PyClassInitializer<T>>) -> PyResult<&mut Self>
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

    /// Get the reference of base object.
    pub fn get_super(&self) -> &<T::BaseType as PyTypeInfo>::ConcreteLayout {
        &self.ob_base
    }

    /// Get the mutable reference of base object.
    pub fn get_super_mut(&mut self) -> &mut <T::BaseType as PyTypeInfo>::ConcreteLayout {
        &mut self.ob_base
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
    unsafe fn internal_ref_cast(obj: &PyAny) -> &T {
        let cell = obj.as_ptr() as *const Self;
        &(*cell).pyclass
    }
    unsafe fn internal_mut_cast(obj: &PyAny) -> &mut T {
        let cell = obj.as_ptr() as *const _ as *mut Self;
        &mut (*cell).pyclass
    }
    unsafe fn py_drop(&mut self, py: Python) {
        ManuallyDrop::drop(&mut self.pyclass);
        self.dict.clear_dict(py);
        self.weakref.clear_weakrefs(self.as_ptr(), py);
        self.ob_base.py_drop(py);
    }
    unsafe fn py_init(&mut self, value: T) {
        self.pyclass = ManuallyDrop::new(value);
    }
}

impl<T: PyClass> PyObjectSizedLayout<T> for PyCell<T> {}

impl<T: PyClass> AsPyPointer for PyCell<T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        (self as *const _) as *mut _
    }
}

impl<T: PyClass> std::ops::Deref for PyCell<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.pyclass.deref()
    }
}

impl<T: PyClass> std::ops::DerefMut for PyCell<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.pyclass.deref_mut()
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

unsafe impl<'p, T> FromPyPointer<'p> for &'p PyCell<T>
where
    T: PyClass,
{
    unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<Self> {
        NonNull::new(ptr).map(|p| &*(gil::register_owned(py, p).as_ptr() as *const PyCell<T>))
    }
    unsafe fn from_borrowed_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<Self> {
        NonNull::new(ptr).map(|p| &*(gil::register_borrowed(py, p).as_ptr() as *const PyCell<T>))
    }
}

unsafe impl<'p, T> FromPyPointer<'p> for &'p mut PyCell<T>
where
    T: PyClass,
{
    unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<Self> {
        NonNull::new(ptr)
            .map(|p| &mut *(gil::register_owned(py, p).as_ptr() as *const _ as *mut PyCell<T>))
    }
    unsafe fn from_borrowed_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<Self> {
        NonNull::new(ptr)
            .map(|p| &mut *(gil::register_borrowed(py, p).as_ptr() as *const _ as *mut PyCell<T>))
    }
}
