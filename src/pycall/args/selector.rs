//! This module is responsible, given an unpacked argument, to find the best storage for it.
//! We do that using [autoderef specialization]. We rank each storage implementation,
//! and use the first that can be used.
//!
//! Here are all implementations, ordered by their rank, from the best to the worst:
//!
//!  1. Empty argument set (`()`, `[T; 0]`, `&[T; 0]` and `&mut [T; 0]`).
//!  2. Arrays (`[T; N]`, `&[T; N]` and `&mut [T; N]`).
//!  3. Existing argument slice.
//!  4. `TrustedLen` iterators.
//!  5. `ExactSizeIterator`s.
//!  6. Any `IntoIterator`.
//!  7. `PyTuple`.
//!  8. Python builtin object (sets, lists, etc.).
//!  9. Any Python object.
//! 10. Tuples.
//!
//! They are divided to four groups:
//!
//!  - The empty argument is before everything, since it is cancelled by everything,
//!    and so its effect on performance is zero. That means it is a better match than anything else.
//!    It also enables calling to `PyObject_CallNoArgs()`.
//!  - Stack allocated arrays come next. That includes arrays and tuples. They are preferred to
//!    existing argument slices because they can be added with `PY_VECTORCALL_ARGUMENTS_OFFSET`.
//!  - Existing argument slices come next, since they are better than anything that has to allocate
//!    memory.
//!  - `TrustedLen`, `ExactSizeIterator` and any iterator have to come in this order specifically
//!    since each latter one is a superset of the former, but has a less efficient implementation.
//!  - Likewise for `PyTuple`, Python builtin objects and any Python object. `PyTuple` can be used
//!    as-is for calls, Python builtins are essentially `TrustedLen` iterators, and other Python iterables
//!    are equivalent to any Rust iterator.
//!  - Tuples come last not because they are less efficient (in fact they are equivalent to arrays),
//!    but because it is more convenient to put them in a "catch-all" bound instead of having to
//!    enumerate each tuple type using a macro again. It doesn't matter for performance since
//!    nothing else can match tuples.
//!
//! [autoderef specialization]: https://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html

use std::marker::PhantomData;

pub mod select_traits {
    pub use super::any_iterator::AnyIteratorSelector as _;
    pub use super::any_pyiterable::AnyPyIterableSelector as _;
    pub use super::array::ArraySelector as _;
    pub use super::empty::EmptySelector as _;
    pub use super::exact_size::ExactSizeIteratorSelector as _;
    pub use super::existing::ExistingSelector as _;
    pub use super::python_builtins::PythonBuiltinsSelector as _;
    pub use super::pytuple::PyTupleSelector as _;
    pub use super::trusted_len::TrustedLenSelector as _;
    pub use super::tuple::TupleSelector as _;
}

pub struct ArgsStorageSelector<T>(PhantomData<T>);

impl<T> ArgsStorageSelector<T> {
    /// This is called by the macro like the following:
    ///
    /// ```ignore
    /// ArgsStorageSelector::new(loop {
    ///     break None;
    ///     break Some(value);
    /// })
    /// ```
    ///
    /// This way, the compiler infers the correct type, but no code is actually executed.
    ///
    /// Note that `if false` cannot be used instead, as it can cause borrow checker errors.
    /// The borrow checker understands this construct as unreachable, and so won't complain.
    #[inline(always)]
    pub fn new(_: Option<T>) -> Self {
        Self(PhantomData)
    }
}

mod empty {
    use crate::prelude::IntoPyObject;
    use crate::pycall::args::empty::EmptyArgsStorage;
    use crate::pycall::args::FundamentalStorage;

    use super::ArgsStorageSelector;

    #[diagnostic::on_unimplemented(
        message = "`{Self}` cannot be unpacked in a Python call",
        note = "the following types can be unpacked in a Python call: \
            any iterable Python object, any `IntoIterator` that yields \
            types that can be converted into Python objects"
    )]
    pub trait EmptySelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_args_select(self, value: T) -> Self::Output;
    }

    impl<'py> EmptySelector<'py, ()> for &&&&&&&&&&&ArgsStorageSelector<()> {
        type Output = EmptyArgsStorage;
        #[inline(always)]
        fn __py_unpack_args_select(self, _value: ()) -> EmptyArgsStorage {
            EmptyArgsStorage
        }
    }
    impl<'py, T> EmptySelector<'py, [T; 0]> for &&&&&&&&&&&ArgsStorageSelector<[T; 0]>
    where
        T: IntoPyObject<'py>,
    {
        type Output = EmptyArgsStorage;
        #[inline(always)]
        fn __py_unpack_args_select(self, _value: [T; 0]) -> EmptyArgsStorage {
            EmptyArgsStorage
        }
    }
    impl<'py, 'a, T> EmptySelector<'py, &'a [T; 0]> for &&&&&&&&&&&ArgsStorageSelector<&'a [T; 0]>
    where
        T: IntoPyObject<'py>,
    {
        type Output = EmptyArgsStorage;
        #[inline(always)]
        fn __py_unpack_args_select(self, _value: &'a [T; 0]) -> EmptyArgsStorage {
            EmptyArgsStorage
        }
    }
    impl<'py, 'a, T> EmptySelector<'py, &'a mut [T; 0]>
        for &&&&&&&&&&&ArgsStorageSelector<&'a mut [T; 0]>
    where
        T: IntoPyObject<'py>,
    {
        type Output = EmptyArgsStorage;
        #[inline(always)]
        fn __py_unpack_args_select(self, _value: &'a mut [T; 0]) -> EmptyArgsStorage {
            EmptyArgsStorage
        }
    }
}

mod array {
    use crate::conversion::IntoPyObject;
    use crate::pycall::args::array::ArrayArgsStorage;
    use crate::pycall::args::FundamentalStorage;

    use super::ArgsStorageSelector;

    #[diagnostic::on_unimplemented(
        message = "`{Self}` cannot be unpacked in a Python call",
        note = "the following types can be unpacked in a Python call: \
            any iterable Python object, any `IntoIterator` that yields \
            types that can be converted into Python objects"
    )]
    pub trait ArraySelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_args_select(self, value: T) -> Self::Output;
    }

    impl<'py, T, const N: usize> ArraySelector<'py, [T; N]> for &&&&&&&&&&ArgsStorageSelector<[T; N]>
    where
        T: IntoPyObject<'py>,
    {
        type Output = ArrayArgsStorage<[T; N]>;
        #[inline(always)]
        fn __py_unpack_args_select(self, value: [T; N]) -> ArrayArgsStorage<[T; N]> {
            ArrayArgsStorage(value)
        }
    }

    impl<'a, 'py, T, const N: usize> ArraySelector<'py, &'a [T; N]>
        for &&&&&&&&&&ArgsStorageSelector<&'a [T; N]>
    where
        &'a T: IntoPyObject<'py>,
    {
        type Output = ArrayArgsStorage<&'a [T; N]>;
        #[inline(always)]
        fn __py_unpack_args_select(self, value: &'a [T; N]) -> ArrayArgsStorage<&'a [T; N]> {
            ArrayArgsStorage(value)
        }
    }

    impl<'a, 'py, T, const N: usize> ArraySelector<'py, &'a mut [T; N]>
        for &&&&&&&&&&ArgsStorageSelector<&'a mut [T; N]>
    where
        &'a T: IntoPyObject<'py>,
    {
        type Output = ArrayArgsStorage<&'a mut [T; N]>;
        #[inline(always)]
        fn __py_unpack_args_select(
            self,
            value: &'a mut [T; N],
        ) -> ArrayArgsStorage<&'a mut [T; N]> {
            ArrayArgsStorage(value)
        }
    }
}

mod existing {
    use crate::pycall::args::existing::{ExistingArgListSlice, ExistingArgListSliceTrait};
    use crate::pycall::args::FundamentalStorage;

    use super::ArgsStorageSelector;

    #[diagnostic::on_unimplemented(
        message = "`{Self}` cannot be unpacked in a Python call",
        note = "the following types can be unpacked in a Python call: \
            any iterable Python object, any `IntoIterator` that yields \
            types that can be converted into Python objects"
    )]
    pub trait ExistingSelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_args_select(self, value: T) -> Self::Output;
    }

    impl<'py, T: ExistingArgListSliceTrait> ExistingSelector<'py, T>
        for &&&&&&&&&ArgsStorageSelector<T>
    {
        type Output = ExistingArgListSlice<T>;
        #[inline(always)]
        fn __py_unpack_args_select(self, value: T) -> ExistingArgListSlice<T> {
            ExistingArgListSlice(value)
        }
    }
}

mod trusted_len {
    use crate::conversion::IntoPyObject;
    use crate::pycall::args::vec::{TrustedLenIterator, VecArgsStorage};
    use crate::pycall::args::FundamentalStorage;
    use crate::pycall::trusted_len::TrustedLen;

    use super::ArgsStorageSelector;

    #[diagnostic::on_unimplemented(
        message = "`{Self}` cannot be unpacked in a Python call",
        note = "the following types can be unpacked in a Python call: \
            any iterable Python object, any `IntoIterator` that yields \
            types that can be converted into Python objects"
    )]
    pub trait TrustedLenSelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_args_select(self, value: T) -> Self::Output;
    }

    impl<'py, T, Item> TrustedLenSelector<'py, T> for &&&&&&&&ArgsStorageSelector<T>
    where
        T: IntoIterator,
        T::IntoIter: TrustedLen<Item = Item>,
        Item: IntoPyObject<'py>,
    {
        type Output = VecArgsStorage<TrustedLenIterator<T::IntoIter>>;
        #[inline(always)]
        fn __py_unpack_args_select(
            self,
            value: T,
        ) -> VecArgsStorage<TrustedLenIterator<T::IntoIter>> {
            VecArgsStorage(TrustedLenIterator(value.into_iter()))
        }
    }
}

mod exact_size {
    use crate::conversion::IntoPyObject;
    use crate::pycall::args::vec::{ExactSizeIterator, VecArgsStorage};
    use crate::pycall::args::FundamentalStorage;

    use super::ArgsStorageSelector;

    #[diagnostic::on_unimplemented(
        message = "`{Self}` cannot be unpacked in a Python call",
        note = "the following types can be unpacked in a Python call: \
            any iterable Python object, any `IntoIterator` that yields \
            types that can be converted into Python objects"
    )]
    pub trait ExactSizeIteratorSelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_args_select(self, value: T) -> Self::Output;
    }

    impl<'py, T, Item> ExactSizeIteratorSelector<'py, T> for &&&&&&&ArgsStorageSelector<T>
    where
        T: IntoIterator,
        T::IntoIter: std::iter::ExactSizeIterator<Item = Item>,
        Item: IntoPyObject<'py>,
    {
        type Output = VecArgsStorage<ExactSizeIterator<T::IntoIter>>;
        #[inline(always)]
        fn __py_unpack_args_select(
            self,
            value: T,
        ) -> VecArgsStorage<ExactSizeIterator<T::IntoIter>> {
            VecArgsStorage(ExactSizeIterator::new(value.into_iter()))
        }
    }
}

mod any_iterator {
    use crate::conversion::IntoPyObject;
    use crate::pycall::args::unknown_size::{AnyIteratorArgs, UnsizedArgsStorage};
    use crate::pycall::args::FundamentalStorage;

    use super::ArgsStorageSelector;

    #[diagnostic::on_unimplemented(
        message = "`{Self}` cannot be unpacked in a Python call",
        note = "the following types can be unpacked in a Python call: \
            any iterable Python object, any `IntoIterator` that yields \
            types that can be converted into Python objects"
    )]
    pub trait AnyIteratorSelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_args_select(self, value: T) -> Self::Output;
    }

    impl<'py, T, Item> AnyIteratorSelector<'py, T> for &&&&&&ArgsStorageSelector<T>
    where
        T: IntoIterator<Item = Item>,
        Item: IntoPyObject<'py>,
    {
        type Output = UnsizedArgsStorage<AnyIteratorArgs<T::IntoIter>>;
        #[inline(always)]
        fn __py_unpack_args_select(
            self,
            value: T,
        ) -> UnsizedArgsStorage<AnyIteratorArgs<T::IntoIter>> {
            UnsizedArgsStorage(AnyIteratorArgs(value.into_iter()))
        }
    }
}

mod pytuple {
    use crate::pycall::args::pyobjects::PyTupleArgs;
    use crate::pycall::args::unknown_size::UnsizedArgsStorage;
    use crate::pycall::args::FundamentalStorage;
    use crate::pycall::as_pyobject::AsPyObject;
    use crate::types::PyTuple;

    use super::ArgsStorageSelector;

    #[diagnostic::on_unimplemented(
        message = "`{Self}` cannot be unpacked in a Python call",
        note = "the following types can be unpacked in a Python call: \
            any iterable Python object, any `IntoIterator` that yields \
            types that can be converted into Python objects"
    )]
    pub trait PyTupleSelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_args_select(self, value: T) -> Self::Output;
    }

    impl<'py, T: AsPyObject<'py, PyObject = PyTuple>> PyTupleSelector<'py, T>
        for &&&&&ArgsStorageSelector<T>
    {
        type Output = UnsizedArgsStorage<PyTupleArgs<T>>;
        #[inline(always)]
        fn __py_unpack_args_select(self, value: T) -> UnsizedArgsStorage<PyTupleArgs<T>> {
            UnsizedArgsStorage(PyTupleArgs::new(value))
        }
    }
}

mod python_builtins {
    use crate::pycall::args::pyobjects::{IterableBuiltin, PyBuiltinIterableArgs};
    use crate::pycall::args::unknown_size::UnsizedArgsStorage;
    use crate::pycall::args::FundamentalStorage;
    use crate::pycall::as_pyobject::AsPyObject;

    use super::ArgsStorageSelector;

    #[diagnostic::on_unimplemented(
        message = "`{Self}` cannot be unpacked in a Python call",
        note = "the following types can be unpacked in a Python call: \
            any iterable Python object, any `IntoIterator` that yields \
            types that can be converted into Python objects"
    )]
    pub trait PythonBuiltinsSelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_args_select(self, value: T) -> Self::Output;
    }

    impl<'py, T> PythonBuiltinsSelector<'py, T> for &&&&ArgsStorageSelector<T>
    where
        T: AsPyObject<'py>,
        T::PyObject: IterableBuiltin,
    {
        type Output = UnsizedArgsStorage<PyBuiltinIterableArgs<T>>;
        #[inline(always)]
        fn __py_unpack_args_select(self, value: T) -> UnsizedArgsStorage<PyBuiltinIterableArgs<T>> {
            UnsizedArgsStorage(PyBuiltinIterableArgs::new(value))
        }
    }
}

mod any_pyiterable {
    use crate::pycall::args::pyobjects::AnyPyIterable;
    use crate::pycall::args::unknown_size::UnsizedArgsStorage;
    use crate::pycall::args::FundamentalStorage;
    use crate::pycall::as_pyobject::AsPyObject;

    use super::ArgsStorageSelector;

    #[diagnostic::on_unimplemented(
        message = "`{Self}` cannot be unpacked in a Python call",
        note = "the following types can be unpacked in a Python call: \
            any iterable Python object, any `IntoIterator` that yields \
            types that can be converted into Python objects"
    )]
    pub trait AnyPyIterableSelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_args_select(self, value: T) -> Self::Output;
    }

    impl<'py, T: AsPyObject<'py>> AnyPyIterableSelector<'py, T> for &&&ArgsStorageSelector<T> {
        type Output = UnsizedArgsStorage<AnyPyIterable<T>>;
        #[inline(always)]
        fn __py_unpack_args_select(self, value: T) -> UnsizedArgsStorage<AnyPyIterable<T>> {
            UnsizedArgsStorage(AnyPyIterable::new(value))
        }
    }
}

mod tuple {
    use crate::pycall::args::array::Tuple;
    use crate::pycall::args::{ArrayArgsStorage, FundamentalStorage};
    use crate::pycall::storage::RawStorage;
    use crate::pycall::PPPyObject;

    use super::ArgsStorageSelector;

    #[diagnostic::on_unimplemented(
        message = "`{Self}` cannot be unpacked in a Python call",
        note = "the following types can be unpacked in a Python call: \
            any iterable Python object, any `IntoIterator` that yields \
            types that can be converted into Python objects"
    )]
    pub trait TupleSelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_args_select(self, value: T) -> Self::Output;
    }

    impl<'py, T: Tuple<'py>> TupleSelector<'py, T> for &&ArgsStorageSelector<T>
    where
        T::RawStorage: for<'a> RawStorage<InitParam<'a> = PPPyObject> + 'static,
    {
        type Output = ArrayArgsStorage<T>;
        #[inline(always)]
        fn __py_unpack_args_select(self, value: T) -> ArrayArgsStorage<T> {
            ArrayArgsStorage(value)
        }
    }
}
