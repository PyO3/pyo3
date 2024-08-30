use crate::pycall::as_pyobject::AsPyObject;
use crate::pycall::storage::{UnsizedInitParam, UnsizedStorage};
use crate::pycall::PPPyObject;
use crate::types::{
    PyAnyMethods, PyByteArray, PyBytes, PyDict, PyDictItems, PyDictKeys, PyDictValues, PyFrozenSet,
    PyList, PySet, PyTuple,
};
use crate::{ffi, Borrowed, Bound, BoundObject, PyAny, PyResult, PyTypeInfo, Python};

use super::helpers::{write_iter_to_tuple, write_raw_storage_to_tuple, DropManyGuard};
use super::unknown_size::UnsizedGuard;
use super::{ArgumentsOffsetFlag, ResolveArgs};

pub struct PyTupleArgs<T> {
    value: T,
    is_not_tuple_subclass: bool,
    len: usize,
}

impl<'py, T: AsPyObject<'py, PyObject = PyTuple>> PyTupleArgs<T> {
    #[inline(always)]
    pub fn new(value: T) -> Self {
        let value_borrowed = value.as_borrowed(unsafe { Python::assume_gil_acquired() });
        let is_not_tuple_subclass = value_borrowed.is_exact_instance_of::<PyTuple>();
        let len = value_borrowed.len().unwrap_or(0);
        Self {
            value,
            is_not_tuple_subclass,
            len,
        }
    }
}

impl<'py, T: AsPyObject<'py, PyObject = PyTuple>> ResolveArgs<'py> for PyTupleArgs<T> {
    type RawStorage = UnsizedStorage;
    type Guard = (Self, UnsizedGuard<Bound<'py, PyAny>>);
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: UnsizedInitParam<'_>,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        if self.is_not_tuple_subclass {
            storage.reserve(self.len);
            unsafe {
                // The buffer might've been invalidated.
                base_storage.cast_mut().write(storage.as_mut_ptr());
            }
            DropManyGuard::from_iter(
                py,
                unsafe { storage.as_mut_ptr().add(storage.len()) },
                base_storage,
                self.value.as_borrowed(py).iter_borrowed(),
            )?;
            Ok((self, UnsizedGuard::empty(base_storage)))
        } else {
            let guard = UnsizedGuard::from_iter(
                storage,
                base_storage,
                self.len,
                self.value.as_borrowed(py).as_any().iter()?,
            )?;
            Ok((self, guard))
        }
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        debug_assert!(
            self.is_not_tuple_subclass,
            "cannot write an unsized type directly into tuple",
        );
        write_iter_to_tuple(
            tuple,
            self.value.as_borrowed(tuple.py()).iter_borrowed(),
            index,
        )
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        tuple: Borrowed<'_, 'py, PyTuple>,
        guard: Self::Guard,
        raw_storage: &mut PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) {
        if guard.0.is_not_tuple_subclass {
            write_raw_storage_to_tuple::<Borrowed<'_, '_, PyAny>, _>(
                tuple,
                raw_storage,
                index,
                guard.0.len,
            );
        } else {
            write_raw_storage_to_tuple::<Bound<'_, PyAny>, _>(
                tuple,
                raw_storage,
                index,
                guard.0.len,
            );
        }
        std::mem::forget(guard.1);
    }
    #[inline(always)]
    fn as_pytuple(&self, py: Python<'py>) -> Option<Borrowed<'_, 'py, PyTuple>> {
        if self.is_not_tuple_subclass {
            Some(self.value.as_borrowed(py))
        } else {
            None
        }
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        self.is_not_tuple_subclass
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = false;
    const IS_ONE: bool = false;
    const USE_STACK_FOR_SMALL_LEN: bool = true;
}

pub struct PyBuiltinIterableArgs<T> {
    value: T,
    is_not_builtin_subclass: bool,
    len: usize,
}

pub trait IterableBuiltin: PyTypeInfo {}
impl IterableBuiltin for PyByteArray {}
impl IterableBuiltin for PyBytes {}
impl IterableBuiltin for PyDict {}
impl IterableBuiltin for PyDictKeys {}
impl IterableBuiltin for PyDictValues {}
impl IterableBuiltin for PyDictItems {}
impl IterableBuiltin for PySet {}
impl IterableBuiltin for PyFrozenSet {}
impl IterableBuiltin for PyList {}

impl<'py, T> PyBuiltinIterableArgs<T>
where
    T: AsPyObject<'py>,
    T::PyObject: IterableBuiltin,
{
    #[inline(always)]
    pub fn new(value: T) -> Self {
        let value_borrowed = value
            .as_borrowed(unsafe { Python::assume_gil_acquired() })
            .into_any();
        let is_not_builtin_subclass = value_borrowed.is_exact_instance_of::<T::PyObject>();
        let len = value_borrowed.len().unwrap_or(0);
        Self {
            value,
            is_not_builtin_subclass,
            len,
        }
    }
}

impl<'py, Target, T> ResolveArgs<'py> for PyBuiltinIterableArgs<T>
where
    T: AsPyObject<'py, PyObject = Target>,
    Target: IterableBuiltin,
{
    type RawStorage = UnsizedStorage;
    type Guard = (T, UnsizedGuard<Bound<'py, PyAny>>);
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: UnsizedInitParam<'_>,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        let len = self.len;
        if self.is_not_builtin_subclass {
            storage.reserve(len);
            unsafe {
                // The buffer might've been invalidated.
                base_storage.cast_mut().write(storage.as_mut_ptr());
            }
            let start = unsafe { storage.as_mut_ptr().add(storage.len()) };
            DropManyGuard::from_iter(
                py,
                start,
                base_storage,
                self.value.as_borrowed(py).as_any().iter(),
            )?;
            Ok((
                self.value,
                UnsizedGuard::from_range(base_storage, start, len),
            ))
        } else {
            let guard = UnsizedGuard::from_iter(
                storage,
                base_storage,
                len,
                self.value.as_borrowed(py).as_any().iter()?,
            )?;
            Ok((self.value, guard))
        }
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        debug_assert!(
            self.is_not_builtin_subclass,
            "cannot write an unsized type directly into tuple",
        );
        write_iter_to_tuple(
            tuple,
            self.value.as_borrowed(tuple.py()).as_any().iter(),
            index,
        )
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        tuple: Borrowed<'_, 'py, PyTuple>,
        guard: Self::Guard,
        raw_storage: &mut PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) {
        write_raw_storage_to_tuple::<Bound<'_, PyAny>, _>(tuple, raw_storage, index, guard.1.len());
        std::mem::forget(guard.1);
    }
    #[inline(always)]
    fn as_pytuple(&self, _py: Python<'py>) -> Option<Borrowed<'_, 'py, PyTuple>> {
        None
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        self.is_not_builtin_subclass
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = false;
    const IS_ONE: bool = false;
    const USE_STACK_FOR_SMALL_LEN: bool = true;
}

pub struct AnyPyIterable<T> {
    value: T,
    len: usize,
}

impl<'py, T: AsPyObject<'py>> AnyPyIterable<T> {
    #[inline(always)]
    pub fn new(value: T) -> Self {
        let value_borrowed = value
            .as_borrowed(unsafe { Python::assume_gil_acquired() })
            .into_any();
        let len = value_borrowed.len().unwrap_or(0);
        Self { value, len }
    }
}

impl<'py, T: AsPyObject<'py>> ResolveArgs<'py> for AnyPyIterable<T> {
    type RawStorage = UnsizedStorage;
    type Guard = (T, UnsizedGuard<Bound<'py, PyAny>>);
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: UnsizedInitParam<'_>,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        let guard = UnsizedGuard::from_iter(
            storage,
            base_storage,
            self.len,
            self.value.as_borrowed(py).as_any().iter()?,
        )?;
        Ok((self.value, guard))
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        _tuple: Borrowed<'_, 'py, PyTuple>,
        _index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        panic!("cannot write an unsized type directly into tuple")
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        tuple: Borrowed<'_, 'py, PyTuple>,
        guard: Self::Guard,
        raw_storage: &mut PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) {
        write_raw_storage_to_tuple::<Bound<'_, PyAny>, _>(tuple, raw_storage, index, guard.1.len());
        std::mem::forget(guard.1);
    }
    #[inline(always)]
    fn as_pytuple(&self, py: Python<'py>) -> Option<Borrowed<'_, 'py, PyTuple>> {
        let value = self.value.as_borrowed(py).into_any();
        if value.is_exact_instance_of::<PyTuple>() {
            // FIXME: There is no downcast method for `Borrowed`.
            unsafe {
                Some(std::mem::transmute::<
                    Borrowed<'_, 'py, PyAny>,
                    Borrowed<'_, 'py, PyTuple>,
                >(value))
            }
        } else {
            None
        }
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        false
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = false;
    const IS_ONE: bool = false;
    const USE_STACK_FOR_SMALL_LEN: bool = false;
}
