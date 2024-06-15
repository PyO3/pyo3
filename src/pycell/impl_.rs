#![allow(missing_docs)]
//! Crate-private implementation of PyClassObject

use std::cell::{Cell, UnsafeCell};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use crate::impl_::pyclass::{
    PyClassBaseType, PyClassDict, PyClassImpl, PyClassThreadChecker, PyClassWeakRef,
};
use crate::type_object::{get_tp_free, PyLayout, PySizedLayout};
use crate::{ffi, PyClass, PyTypeInfo, Python};

use super::{PyBorrowError, PyBorrowMutError};

pub trait PyClassMutability {
    // The storage for this inheritance layer. Only the first mutable class in
    // an inheritance hierarchy needs to store the borrow flag.
    type Storage: PyClassBorrowChecker;
    // The borrow flag needed to implement this class' mutability. Empty until
    // the first mutable class, at which point it is BorrowChecker and will be
    // for all subclasses.
    type Checker: PyClassBorrowChecker;
    type ImmutableChild: PyClassMutability;
    type MutableChild: PyClassMutability;
}

pub struct ImmutableClass(());
pub struct MutableClass(());
pub struct ExtendsMutableAncestor<M: PyClassMutability>(PhantomData<M>);

impl PyClassMutability for ImmutableClass {
    type Storage = EmptySlot;
    type Checker = EmptySlot;
    type ImmutableChild = ImmutableClass;
    type MutableChild = MutableClass;
}

impl PyClassMutability for MutableClass {
    type Storage = BorrowChecker;
    type Checker = BorrowChecker;
    type ImmutableChild = ExtendsMutableAncestor<ImmutableClass>;
    type MutableChild = ExtendsMutableAncestor<MutableClass>;
}

impl<M: PyClassMutability> PyClassMutability for ExtendsMutableAncestor<M> {
    type Storage = EmptySlot;
    type Checker = BorrowChecker;
    type ImmutableChild = ExtendsMutableAncestor<ImmutableClass>;
    type MutableChild = ExtendsMutableAncestor<MutableClass>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct BorrowFlag(usize);

impl BorrowFlag {
    pub(crate) const UNUSED: BorrowFlag = BorrowFlag(0);
    const HAS_MUTABLE_BORROW: BorrowFlag = BorrowFlag(usize::MAX);
    const fn increment(self) -> Self {
        Self(self.0 + 1)
    }
    const fn decrement(self) -> Self {
        Self(self.0 - 1)
    }
}

pub struct EmptySlot(());
pub struct BorrowChecker(Cell<BorrowFlag>);

pub trait PyClassBorrowChecker {
    /// Initial value for self
    fn new() -> Self;

    /// Increments immutable borrow count, if possible
    fn try_borrow(&self) -> Result<(), PyBorrowError>;

    #[cfg(feature = "gil-refs")]
    fn try_borrow_unguarded(&self) -> Result<(), PyBorrowError>;

    /// Decrements immutable borrow count
    fn release_borrow(&self);
    /// Increments mutable borrow count, if possible
    fn try_borrow_mut(&self) -> Result<(), PyBorrowMutError>;
    /// Decremements mutable borrow count
    fn release_borrow_mut(&self);
}

impl PyClassBorrowChecker for EmptySlot {
    #[inline]
    fn new() -> Self {
        EmptySlot(())
    }

    #[inline]
    fn try_borrow(&self) -> Result<(), PyBorrowError> {
        Ok(())
    }

    #[inline]
    #[cfg(feature = "gil-refs")]
    fn try_borrow_unguarded(&self) -> Result<(), PyBorrowError> {
        Ok(())
    }

    #[inline]
    fn release_borrow(&self) {}

    #[inline]
    fn try_borrow_mut(&self) -> Result<(), PyBorrowMutError> {
        unreachable!()
    }

    #[inline]
    fn release_borrow_mut(&self) {
        unreachable!()
    }
}

impl PyClassBorrowChecker for BorrowChecker {
    #[inline]
    fn new() -> Self {
        Self(Cell::new(BorrowFlag::UNUSED))
    }

    fn try_borrow(&self) -> Result<(), PyBorrowError> {
        let flag = self.0.get();
        if flag != BorrowFlag::HAS_MUTABLE_BORROW {
            self.0.set(flag.increment());
            Ok(())
        } else {
            Err(PyBorrowError { _private: () })
        }
    }

    #[cfg(feature = "gil-refs")]
    fn try_borrow_unguarded(&self) -> Result<(), PyBorrowError> {
        let flag = self.0.get();
        if flag != BorrowFlag::HAS_MUTABLE_BORROW {
            Ok(())
        } else {
            Err(PyBorrowError { _private: () })
        }
    }

    fn release_borrow(&self) {
        let flag = self.0.get();
        self.0.set(flag.decrement())
    }

    fn try_borrow_mut(&self) -> Result<(), PyBorrowMutError> {
        let flag = self.0.get();
        if flag == BorrowFlag::UNUSED {
            self.0.set(BorrowFlag::HAS_MUTABLE_BORROW);
            Ok(())
        } else {
            Err(PyBorrowMutError { _private: () })
        }
    }

    fn release_borrow_mut(&self) {
        self.0.set(BorrowFlag::UNUSED)
    }
}

pub trait GetBorrowChecker<T: PyClassImpl> {
    fn borrow_checker(
        class_object: &PyClassObject<T>,
    ) -> &<T::PyClassMutability as PyClassMutability>::Checker;
}

impl<T: PyClassImpl<PyClassMutability = Self>> GetBorrowChecker<T> for MutableClass {
    fn borrow_checker(class_object: &PyClassObject<T>) -> &BorrowChecker {
        &class_object.contents.borrow_checker
    }
}

impl<T: PyClassImpl<PyClassMutability = Self>> GetBorrowChecker<T> for ImmutableClass {
    fn borrow_checker(class_object: &PyClassObject<T>) -> &EmptySlot {
        &class_object.contents.borrow_checker
    }
}

impl<T: PyClassImpl<PyClassMutability = Self>, M: PyClassMutability> GetBorrowChecker<T>
    for ExtendsMutableAncestor<M>
where
    T::BaseType: PyClassImpl + PyClassBaseType<LayoutAsBase = PyClassObject<T::BaseType>>,
    <T::BaseType as PyClassImpl>::PyClassMutability: PyClassMutability<Checker = BorrowChecker>,
{
    fn borrow_checker(class_object: &PyClassObject<T>) -> &BorrowChecker {
        <<T::BaseType as PyClassImpl>::PyClassMutability as GetBorrowChecker<T::BaseType>>::borrow_checker(&class_object.ob_base)
    }
}

/// Base layout of PyClassObject.
#[doc(hidden)]
#[repr(C)]
pub struct PyClassObjectBase<T> {
    ob_base: T,
}

unsafe impl<T, U> PyLayout<T> for PyClassObjectBase<U> where U: PySizedLayout<T> {}

#[doc(hidden)]
pub trait PyClassObjectLayout<T>: PyLayout<T> {
    fn ensure_threadsafe(&self);
    fn check_threadsafe(&self) -> Result<(), PyBorrowError>;
    /// Implementation of tp_dealloc.
    /// # Safety
    /// - slf must be a valid pointer to an instance of a T or a subclass.
    /// - slf must not be used after this call (as it will be freed).
    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject);
}

impl<T, U> PyClassObjectLayout<T> for PyClassObjectBase<U>
where
    U: PySizedLayout<T>,
    T: PyTypeInfo,
{
    fn ensure_threadsafe(&self) {}
    fn check_threadsafe(&self) -> Result<(), PyBorrowError> {
        Ok(())
    }
    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject) {
        let type_obj = T::type_object_raw(py);
        // For `#[pyclass]` types which inherit from PyAny, we can just call tp_free
        if type_obj == std::ptr::addr_of_mut!(ffi::PyBaseObject_Type) {
            return get_tp_free(ffi::Py_TYPE(slf))(slf.cast());
        }

        // More complex native types (e.g. `extends=PyDict`) require calling the base's dealloc.
        #[cfg(not(Py_LIMITED_API))]
        {
            if let Some(dealloc) = (*type_obj).tp_dealloc {
                // Before CPython 3.11 BaseException_dealloc would use Py_GC_UNTRACK which
                // assumes the exception is currently GC tracked, so we have to re-track
                // before calling the dealloc so that it can safely call Py_GC_UNTRACK.
                #[cfg(not(any(Py_3_11, PyPy)))]
                if ffi::PyType_FastSubclass(type_obj, ffi::Py_TPFLAGS_BASE_EXC_SUBCLASS) == 1 {
                    ffi::PyObject_GC_Track(slf.cast());
                }
                dealloc(slf);
            } else {
                get_tp_free(ffi::Py_TYPE(slf))(slf.cast());
            }
        }

        #[cfg(Py_LIMITED_API)]
        unreachable!("subclassing native types is not possible with the `abi3` feature");
    }
}

/// The layout of a PyClass as a Python object
#[repr(C)]
pub struct PyClassObject<T: PyClassImpl> {
    pub(crate) ob_base: <T::BaseType as PyClassBaseType>::LayoutAsBase,
    pub(crate) contents: PyClassObjectContents<T>,
}

#[repr(C)]
pub(crate) struct PyClassObjectContents<T: PyClassImpl> {
    pub(crate) value: ManuallyDrop<UnsafeCell<T>>,
    pub(crate) borrow_checker: <T::PyClassMutability as PyClassMutability>::Storage,
    pub(crate) thread_checker: T::ThreadChecker,
    pub(crate) dict: T::Dict,
    pub(crate) weakref: T::WeakRef,
}

impl<T: PyClassImpl> PyClassObject<T> {
    pub(crate) fn get_ptr(&self) -> *mut T {
        self.contents.value.get()
    }

    /// Gets the offset of the dictionary from the start of the struct in bytes.
    pub(crate) fn dict_offset() -> ffi::Py_ssize_t {
        use memoffset::offset_of;

        let offset =
            offset_of!(PyClassObject<T>, contents) + offset_of!(PyClassObjectContents<T>, dict);

        // Py_ssize_t may not be equal to isize on all platforms
        #[allow(clippy::useless_conversion)]
        offset.try_into().expect("offset should fit in Py_ssize_t")
    }

    /// Gets the offset of the weakref list from the start of the struct in bytes.
    pub(crate) fn weaklist_offset() -> ffi::Py_ssize_t {
        use memoffset::offset_of;

        let offset =
            offset_of!(PyClassObject<T>, contents) + offset_of!(PyClassObjectContents<T>, weakref);

        // Py_ssize_t may not be equal to isize on all platforms
        #[allow(clippy::useless_conversion)]
        offset.try_into().expect("offset should fit in Py_ssize_t")
    }
}

impl<T: PyClassImpl> PyClassObject<T> {
    pub(crate) fn borrow_checker(&self) -> &<T::PyClassMutability as PyClassMutability>::Checker {
        T::PyClassMutability::borrow_checker(self)
    }
}

unsafe impl<T: PyClassImpl> PyLayout<T> for PyClassObject<T> {}
impl<T: PyClass> PySizedLayout<T> for PyClassObject<T> {}

impl<T: PyClassImpl> PyClassObjectLayout<T> for PyClassObject<T>
where
    <T::BaseType as PyClassBaseType>::LayoutAsBase: PyClassObjectLayout<T::BaseType>,
{
    fn ensure_threadsafe(&self) {
        self.contents.thread_checker.ensure();
        self.ob_base.ensure_threadsafe();
    }
    fn check_threadsafe(&self) -> Result<(), PyBorrowError> {
        if !self.contents.thread_checker.check() {
            return Err(PyBorrowError { _private: () });
        }
        self.ob_base.check_threadsafe()
    }
    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject) {
        // Safety: Python only calls tp_dealloc when no references to the object remain.
        let class_object = &mut *(slf.cast::<PyClassObject<T>>());
        if class_object.contents.thread_checker.can_drop(py) {
            ManuallyDrop::drop(&mut class_object.contents.value);
        }
        class_object.contents.dict.clear_dict(py);
        class_object.contents.weakref.clear_weakrefs(slf, py);
        <T::BaseType as PyClassBaseType>::LayoutAsBase::tp_dealloc(py, slf)
    }
}

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {
    use super::*;

    use crate::prelude::*;
    use crate::pyclass::boolean_struct::{False, True};

    #[pyclass(crate = "crate", subclass)]
    struct MutableBase;

    #[pyclass(crate = "crate", extends = MutableBase, subclass)]
    struct MutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = MutableBase, frozen, subclass)]
    struct ImmutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = MutableChildOfMutableBase)]
    struct MutableChildOfMutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = ImmutableChildOfMutableBase)]
    struct MutableChildOfImmutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = MutableChildOfMutableBase, frozen)]
    struct ImmutableChildOfMutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = ImmutableChildOfMutableBase, frozen)]
    struct ImmutableChildOfImmutableChildOfMutableBase;

    #[pyclass(crate = "crate", frozen, subclass)]
    struct ImmutableBase;

    #[pyclass(crate = "crate", extends = ImmutableBase, subclass)]
    struct MutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = ImmutableBase, frozen, subclass)]
    struct ImmutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = MutableChildOfImmutableBase)]
    struct MutableChildOfMutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = ImmutableChildOfImmutableBase)]
    struct MutableChildOfImmutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = MutableChildOfImmutableBase, frozen)]
    struct ImmutableChildOfMutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = ImmutableChildOfImmutableBase, frozen)]
    struct ImmutableChildOfImmutableChildOfImmutableBase;

    fn assert_mutable<T: PyClass<Frozen = False, PyClassMutability = MutableClass>>() {}
    fn assert_immutable<T: PyClass<Frozen = True, PyClassMutability = ImmutableClass>>() {}
    fn assert_mutable_with_mutable_ancestor<
        T: PyClass<Frozen = False, PyClassMutability = ExtendsMutableAncestor<MutableClass>>,
    >() {
    }
    fn assert_immutable_with_mutable_ancestor<
        T: PyClass<Frozen = True, PyClassMutability = ExtendsMutableAncestor<ImmutableClass>>,
    >() {
    }

    #[test]
    fn test_inherited_mutability() {
        // mutable base
        assert_mutable::<MutableBase>();

        // children of mutable base have a mutable ancestor
        assert_mutable_with_mutable_ancestor::<MutableChildOfMutableBase>();
        assert_immutable_with_mutable_ancestor::<ImmutableChildOfMutableBase>();

        // grandchildren of mutable base have a mutable ancestor
        assert_mutable_with_mutable_ancestor::<MutableChildOfMutableChildOfMutableBase>();
        assert_mutable_with_mutable_ancestor::<MutableChildOfImmutableChildOfMutableBase>();
        assert_immutable_with_mutable_ancestor::<ImmutableChildOfMutableChildOfMutableBase>();
        assert_immutable_with_mutable_ancestor::<ImmutableChildOfImmutableChildOfMutableBase>();

        // immutable base and children
        assert_immutable::<ImmutableBase>();
        assert_immutable::<ImmutableChildOfImmutableBase>();
        assert_immutable::<ImmutableChildOfImmutableChildOfImmutableBase>();

        // mutable children of immutable at any level are simply mutable
        assert_mutable::<MutableChildOfImmutableBase>();
        assert_mutable::<MutableChildOfImmutableChildOfImmutableBase>();

        // children of the mutable child display this property
        assert_mutable_with_mutable_ancestor::<MutableChildOfMutableChildOfImmutableBase>();
        assert_immutable_with_mutable_ancestor::<ImmutableChildOfMutableChildOfImmutableBase>();
    }

    #[test]
    fn test_mutable_borrow_prevents_further_borrows() {
        Python::with_gil(|py| {
            let mmm = Py::new(
                py,
                PyClassInitializer::from(MutableBase)
                    .add_subclass(MutableChildOfMutableBase)
                    .add_subclass(MutableChildOfMutableChildOfMutableBase),
            )
            .unwrap();

            let mmm_bound: &Bound<'_, MutableChildOfMutableChildOfMutableBase> = mmm.bind(py);

            let mmm_refmut = mmm_bound.borrow_mut();

            // Cannot take any other mutable or immutable borrows whilst the object is borrowed mutably
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound.extract::<PyRef<'_, MutableBase>>().is_err());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound.extract::<PyRefMut<'_, MutableBase>>().is_err());

            // With the borrow dropped, all other borrow attempts will succeed
            drop(mmm_refmut);

            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound.extract::<PyRef<'_, MutableBase>>().is_ok());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound.extract::<PyRefMut<'_, MutableBase>>().is_ok());
        })
    }

    #[test]
    fn test_immutable_borrows_prevent_mutable_borrows() {
        Python::with_gil(|py| {
            let mmm = Py::new(
                py,
                PyClassInitializer::from(MutableBase)
                    .add_subclass(MutableChildOfMutableBase)
                    .add_subclass(MutableChildOfMutableChildOfMutableBase),
            )
            .unwrap();

            let mmm_bound: &Bound<'_, MutableChildOfMutableChildOfMutableBase> = mmm.bind(py);

            let mmm_refmut = mmm_bound.borrow();

            // Further immutable borrows are ok
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound.extract::<PyRef<'_, MutableBase>>().is_ok());

            // Further mutable borrows are not ok
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound.extract::<PyRefMut<'_, MutableBase>>().is_err());

            // With the borrow dropped, all mutable borrow attempts will succeed
            drop(mmm_refmut);

            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound.extract::<PyRefMut<'_, MutableBase>>().is_ok());
        })
    }
}
