#![cfg(feature = "macros")]

use pyo3::impl_::pyclass::{PyClassBaseType, PyClassImpl};
use pyo3::prelude::*;
use pyo3::pycell::{
    BorrowChecker, ExtendsMutableAncestor, ImmutableClass, MutableClass, PyClassMutability,
};
use pyo3::PyClass;

#[pyclass(subclass)]
struct MutableBase;

#[pyclass(extends = MutableBase, subclass)]
struct MutableChildOfMutableBase;

#[pyclass(extends = MutableBase, immutable, subclass)]
struct ImmutableChildOfMutableBase;

#[pyclass(extends = MutableChildOfMutableBase)]
struct MutableChildOfMutableChildOfMutableBase;

#[pyclass(extends = ImmutableChildOfMutableBase)]
struct MutableChildOfImmutableChildOfMutableBase;

#[pyclass(extends = MutableChildOfMutableBase, immutable)]
struct ImmutableChildOfMutableChildOfMutableBase;

#[pyclass(extends = ImmutableChildOfMutableBase, immutable)]
struct ImmutableChildOfImmutableChildOfMutableBase;

#[pyclass(immutable, subclass)]
struct ImmutableBase;

#[pyclass(extends = ImmutableBase, subclass)]
struct MutableChildOfImmutableBase;

#[pyclass(extends = ImmutableBase, immutable, subclass)]
struct ImmutableChildOfImmutableBase;

#[pyclass(extends = MutableChildOfImmutableBase)]
struct MutableChildOfMutableChildOfImmutableBase;

#[pyclass(extends = ImmutableChildOfImmutableBase)]
struct MutableChildOfImmutableChildOfImmutableBase;

#[pyclass(extends = MutableChildOfImmutableBase, immutable)]
struct ImmutableChildOfMutableChildOfImmutableBase;

#[pyclass(extends = ImmutableChildOfImmutableBase, immutable)]
struct ImmutableChildOfImmutableChildOfImmutableBase;

fn assert_mutable<T: PyClass<PyClassMutability = MutableClass>>() {}
fn assert_immutable<T: PyClass<PyClassMutability = ImmutableClass>>() {}
fn assert_mutable_with_mutable_ancestor<
    T: PyClass<PyClassMutability = ExtendsMutableAncestor<MutableClass>>,
>()
// These horrible bounds are necessary for Rust 1.48 but not newer versions
where
    <T as PyClassImpl>::BaseType: PyClassImpl<Layout = PyCell<T::BaseType>>,
    <<T as PyClassImpl>::BaseType as PyClassImpl>::PyClassMutability:
        PyClassMutability<Checker = BorrowChecker>,
    <T as PyClassImpl>::BaseType: PyClassBaseType<LayoutAsBase = PyCell<T::BaseType>>,
{
}
fn assert_immutable_with_mutable_ancestor<
    T: PyClass<PyClassMutability = ExtendsMutableAncestor<ImmutableClass>>,
>()
// These horrible bounds are necessary for Rust 1.48 but not newer versions
where
    <T as PyClassImpl>::BaseType: PyClassImpl<Layout = PyCell<T::BaseType>>,
    <<T as PyClassImpl>::BaseType as PyClassImpl>::PyClassMutability:
        PyClassMutability<Checker = BorrowChecker>,
    <T as PyClassImpl>::BaseType: PyClassBaseType<LayoutAsBase = PyCell<T::BaseType>>,
{
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

        let mmm_cell: &PyCell<MutableChildOfMutableChildOfMutableBase> = mmm.as_ref(py);

        let mmm_refmut = mmm_cell.borrow_mut();

        // Cannot take any other mutable or immutable borrows whilst the object is borrowed mutably
        assert!(mmm_cell
            .extract::<PyRef<'_, MutableChildOfMutableChildOfMutableBase>>()
            .is_err());
        assert!(mmm_cell
            .extract::<PyRef<'_, MutableChildOfMutableBase>>()
            .is_err());
        assert!(mmm_cell.extract::<PyRef<'_, MutableBase>>().is_err());
        assert!(mmm_cell
            .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
            .is_err());
        assert!(mmm_cell
            .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
            .is_err());
        assert!(mmm_cell.extract::<PyRefMut<'_, MutableBase>>().is_err());

        // With the borrow dropped, all other borrow attempts will succeed
        drop(mmm_refmut);

        assert!(mmm_cell
            .extract::<PyRef<'_, MutableChildOfMutableChildOfMutableBase>>()
            .is_ok());
        assert!(mmm_cell
            .extract::<PyRef<'_, MutableChildOfMutableBase>>()
            .is_ok());
        assert!(mmm_cell.extract::<PyRef<'_, MutableBase>>().is_ok());
        assert!(mmm_cell
            .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
            .is_ok());
        assert!(mmm_cell
            .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
            .is_ok());
        assert!(mmm_cell.extract::<PyRefMut<'_, MutableBase>>().is_ok());
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

        let mmm_cell: &PyCell<MutableChildOfMutableChildOfMutableBase> = mmm.as_ref(py);

        let mmm_refmut = mmm_cell.borrow();

        // Further immutable borrows are ok
        assert!(mmm_cell
            .extract::<PyRef<'_, MutableChildOfMutableChildOfMutableBase>>()
            .is_ok());
        assert!(mmm_cell
            .extract::<PyRef<'_, MutableChildOfMutableBase>>()
            .is_ok());
        assert!(mmm_cell.extract::<PyRef<'_, MutableBase>>().is_ok());

        // Further mutable borrows are not ok
        assert!(mmm_cell
            .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
            .is_err());
        assert!(mmm_cell
            .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
            .is_err());
        assert!(mmm_cell.extract::<PyRefMut<'_, MutableBase>>().is_err());

        // With the borrow dropped, all mutable borrow attempts will succeed
        drop(mmm_refmut);

        assert!(mmm_cell
            .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
            .is_ok());
        assert!(mmm_cell
            .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
            .is_ok());
        assert!(mmm_cell.extract::<PyRefMut<'_, MutableBase>>().is_ok());
    })
}
