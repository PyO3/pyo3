#![allow(missing_docs)]
//! Crate-private implementation of PyClassObject

use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::impl_::pyclass::PyClassImpl;
use crate::{ffi, PyTypeInfo};

use super::layout::AssumeInitializedTypeProvider;
use super::{PyBorrowError, PyBorrowMutError, PyObjectLayout};

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

#[derive(Debug)]
struct BorrowFlag(AtomicUsize);

impl BorrowFlag {
    pub(crate) const UNUSED: usize = 0;
    const HAS_MUTABLE_BORROW: usize = usize::MAX;
    fn increment(&self) -> Result<(), PyBorrowError> {
        let mut value = self.0.load(Ordering::Relaxed);
        loop {
            if value == BorrowFlag::HAS_MUTABLE_BORROW {
                return Err(PyBorrowError { _private: () });
            }
            match self.0.compare_exchange(
                // only increment if the value hasn't changed since the
                // last atomic load
                value,
                value + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(..) => {
                    // value has been successfully incremented, we need an acquire fence
                    // so that data this borrow flag protects can be read safely in this thread
                    std::sync::atomic::fence(Ordering::Acquire);
                    break Ok(());
                }
                Err(changed_value) => {
                    // value changed under us, need to try again
                    value = changed_value;
                }
            }
        }
    }
    fn decrement(&self) {
        // impossible to get into a bad state from here so relaxed
        // ordering is fine, the decrement only needs to eventually
        // be visible
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

pub struct EmptySlot(());
pub struct BorrowChecker(BorrowFlag);

pub trait PyClassBorrowChecker {
    /// Initial value for self
    fn new() -> Self;
    /// Increments immutable borrow count, if possible
    fn try_borrow(&self) -> Result<(), PyBorrowError>;
    /// Decrements immutable borrow count
    fn release_borrow(&self);
    /// Increments mutable borrow count, if possible
    fn try_borrow_mut(&self) -> Result<(), PyBorrowMutError>;
    /// Decrements mutable borrow count
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
        Self(BorrowFlag(AtomicUsize::new(BorrowFlag::UNUSED)))
    }

    fn try_borrow(&self) -> Result<(), PyBorrowError> {
        self.0.increment()
    }

    fn release_borrow(&self) {
        self.0.decrement();
    }

    fn try_borrow_mut(&self) -> Result<(), PyBorrowMutError> {
        let flag = &self.0;
        match flag.0.compare_exchange(
            // only allowed to transition to mutable borrow if the reference is
            // currently unused
            BorrowFlag::UNUSED,
            BorrowFlag::HAS_MUTABLE_BORROW,
            // On success, reading the flag and updating its state are an atomic
            // operation
            Ordering::AcqRel,
            // It doesn't matter precisely when the failure gets turned
            // into an error
            Ordering::Relaxed,
        ) {
            Ok(..) => Ok(()),
            Err(..) => Err(PyBorrowMutError { _private: () }),
        }
    }

    fn release_borrow_mut(&self) {
        self.0 .0.store(BorrowFlag::UNUSED, Ordering::Release)
    }
}

pub trait GetBorrowChecker<T: PyClassImpl> {
    fn borrow_checker(obj: &ffi::PyObject)
        -> &<T::PyClassMutability as PyClassMutability>::Checker;
}

impl<T: PyClassImpl<PyClassMutability = Self> + PyTypeInfo> GetBorrowChecker<T> for MutableClass {
    fn borrow_checker(obj: &ffi::PyObject) -> &BorrowChecker {
        let type_provider = unsafe { AssumeInitializedTypeProvider::new() };
        let contents = PyObjectLayout::get_contents::<T, _>(obj, type_provider);
        &contents.borrow_checker
    }
}

impl<T: PyClassImpl<PyClassMutability = Self> + PyTypeInfo> GetBorrowChecker<T> for ImmutableClass {
    fn borrow_checker(obj: &ffi::PyObject) -> &EmptySlot {
        let type_provider = unsafe { AssumeInitializedTypeProvider::new() };
        let contents = PyObjectLayout::get_contents::<T, _>(obj, type_provider);
        &contents.borrow_checker
    }
}

impl<T, M> GetBorrowChecker<T> for ExtendsMutableAncestor<M>
where
    T: PyClassImpl<PyClassMutability = Self>,
    M: PyClassMutability,
    T::BaseType: PyClassImpl,
    <T::BaseType as PyClassImpl>::PyClassMutability: PyClassMutability<Checker = BorrowChecker>,
{
    fn borrow_checker(obj: &ffi::PyObject) -> &BorrowChecker {
        // the same PyObject pointer can be re-interpreted as the base/parent type
        <<T::BaseType as PyClassImpl>::PyClassMutability as GetBorrowChecker<T::BaseType>>::borrow_checker(obj)
    }
}

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {
    use super::*;

    use crate::pyclass::boolean_struct::{False, True};
    use crate::{prelude::*, PyClass};

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

    #[pyclass(crate = "crate", subclass)]
    struct BaseWithData(#[allow(unused)] u64);

    #[pyclass(crate = "crate", extends = BaseWithData)]
    struct ChildWithData(#[allow(unused)] u64);

    #[pyclass(crate = "crate", extends = BaseWithData)]
    struct ChildWithoutData;

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

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_thread_safety() {
        #[crate::pyclass(crate = "crate")]
        struct MyClass {
            x: u64,
        }

        Python::with_gil(|py| {
            let inst = Py::new(py, MyClass { x: 0 }).unwrap();

            let total_modifications = py.allow_threads(|| {
                std::thread::scope(|s| {
                    // Spawn a bunch of threads all racing to write to
                    // the same instance of `MyClass`.
                    let threads = (0..10)
                        .map(|_| {
                            s.spawn(|| {
                                Python::with_gil(|py| {
                                    // Each thread records its own view of how many writes it made
                                    let mut local_modifications = 0;
                                    for _ in 0..100 {
                                        if let Ok(mut i) = inst.try_borrow_mut(py) {
                                            i.x += 1;
                                            local_modifications += 1;
                                        }
                                    }
                                    local_modifications
                                })
                            })
                        })
                        .collect::<Vec<_>>();

                    // Sum up the total number of writes made by all threads
                    threads.into_iter().map(|t| t.join().unwrap()).sum::<u64>()
                })
            });

            // If the implementation is free of data races, the total number of writes
            // should match the final value of `x`.
            assert_eq!(total_modifications, inst.borrow(py).x);
        });
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_thread_safety_2() {
        use std::cell::UnsafeCell;

        struct SyncUnsafeCell<T>(UnsafeCell<T>);
        unsafe impl<T> Sync for SyncUnsafeCell<T> {}

        impl<T> SyncUnsafeCell<T> {
            fn get(&self) -> *mut T {
                self.0.get()
            }
        }

        let data = SyncUnsafeCell(UnsafeCell::new(0));
        let data2 = SyncUnsafeCell(UnsafeCell::new(0));
        let borrow_checker = BorrowChecker(BorrowFlag(AtomicUsize::new(BorrowFlag::UNUSED)));

        std::thread::scope(|s| {
            s.spawn(|| {
                for _ in 0..1_000_000 {
                    if borrow_checker.try_borrow_mut().is_ok() {
                        // thread 1 writes to both values during the mutable borrow
                        unsafe { *data.get() += 1 };
                        unsafe { *data2.get() += 1 };
                        borrow_checker.release_borrow_mut();
                    }
                }
            });

            s.spawn(|| {
                for _ in 0..1_000_000 {
                    if borrow_checker.try_borrow().is_ok() {
                        // if the borrow checker is working correctly, it should be impossible
                        // for thread 2 to observe a difference in the two values
                        assert_eq!(unsafe { *data.get() }, unsafe { *data2.get() });
                        borrow_checker.release_borrow();
                    }
                }
            });
        });
    }
}
