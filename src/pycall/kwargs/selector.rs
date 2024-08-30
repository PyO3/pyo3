//! This module is responsible, given an unpacked argument, to find the best storage for it.
//! We do that using [autoderef specialization]. We rank each storage implementation,
//! and use the first that can be used.
//!
//! Here are all implementations, ordered by their rank, from the best to the worst:
//!
//!  1. Empty argument set (`()`, `[T; 0]`, `&[T; 0]` and `&mut [T; 0]`).
//!  2. Arrays (`[T; N]`, `&[T; N]` and `&mut [T; N]`).
//!  3. `TrustedLen` iterators.
//!  4. `ExactSizeIterator`s.
//!  5. Any `IntoIterator`.
//!  6. `PyDict`.
//!  7. Any Python object.
//!  8. Tuples.
//!
//! They are divided to four groups:
//!
//!  - The empty argument is before everything, since it is cancelled by everything,
//!    and so its effect on performance is zero. That means it is a better match than anything else.
//!    It also enables calling with NULL kwargs.
//!  - Stack allocated arrays come next. That includes arrays and tuples.
//!  - `TrustedLen`, `ExactSizeIterator` and any iterator have to come in this order specifically
//!    since each latter one is a superset of the former, but has a less efficient implementation.
//!  - Likewise for `PyDict` and any Python object. `PyDict` can be used as-is for calls and is `TrustedLen`,
//!    while other Python mappings are equivalent to any Rust iterator.
//!  - Tuples come last not because they are less efficient (in fact they are equivalent to arrays),
//!    but because it is more convenient to put them in a "catch-all" bound instead of having to
//!    enumerate each tuple type using a macro again. It doesn't matter for performance since
//!    nothing else can match tuples.
//!
//! [autoderef specialization]: https://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html

use std::marker::PhantomData;

pub mod select_traits {
    pub use super::any_iterator::AnyIteratorSelector as _;
    pub use super::any_pymapping::AnyPyMappingSelector as _;
    pub use super::array::ArraySelector as _;
    pub use super::empty::EmptySelector as _;
    pub use super::exact_size::ExactSizeIteratorSelector as _;
    pub use super::pydict::PyDictSelector as _;
    pub use super::trusted_len::TrustedLenSelector as _;
    pub use super::tuple::TupleSelector as _;
}

pub struct KwargsStorageSelector<T>(PhantomData<T>);

impl<T> KwargsStorageSelector<T> {
    /// This is called by the macro like the following:
    ///
    /// ```ignore
    /// KwargsStorageSelector::new(loop {
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
    use crate::conversion::IntoPyObject;
    use crate::pycall::kwargs::concat::FundamentalStorage;
    use crate::pycall::kwargs::empty::EmptyKwargsStorage;
    use crate::types::PyString;

    use super::KwargsStorageSelector;

    pub trait EmptySelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_kwargs_select(self, value: T) -> Self::Output;
    }

    impl<'py> EmptySelector<'py, ()> for &&&&&&&&&&&KwargsStorageSelector<()> {
        type Output = EmptyKwargsStorage;
        #[inline(always)]
        fn __py_unpack_kwargs_select(self, _value: ()) -> EmptyKwargsStorage {
            EmptyKwargsStorage
        }
    }
    impl<'py, K, V> EmptySelector<'py, [(K, V); 0]> for &&&&&&&&&&&KwargsStorageSelector<[(K, V); 0]>
    where
        K: IntoPyObject<'py, Target = PyString>,
        V: IntoPyObject<'py>,
    {
        type Output = EmptyKwargsStorage;
        #[inline(always)]
        fn __py_unpack_kwargs_select(self, _value: [(K, V); 0]) -> EmptyKwargsStorage {
            EmptyKwargsStorage
        }
    }
    impl<'py, 'a, K, V> EmptySelector<'py, &'a [(K, V); 0]>
        for &&&&&&&&&&&KwargsStorageSelector<&'a [(K, V); 0]>
    where
        &'a K: IntoPyObject<'py, Target = PyString>,
        &'a V: IntoPyObject<'py>,
    {
        type Output = EmptyKwargsStorage;
        #[inline(always)]
        fn __py_unpack_kwargs_select(self, _value: &'a [(K, V); 0]) -> EmptyKwargsStorage {
            EmptyKwargsStorage
        }
    }
    impl<'py, 'a, K, V> EmptySelector<'py, &'a mut [(K, V); 0]>
        for &&&&&&&&&&&KwargsStorageSelector<&'a mut [(K, V); 0]>
    where
        &'a K: IntoPyObject<'py, Target = PyString>,
        &'a V: IntoPyObject<'py>,
    {
        type Output = EmptyKwargsStorage;
        #[inline(always)]
        fn __py_unpack_kwargs_select(self, _value: &'a mut [(K, V); 0]) -> EmptyKwargsStorage {
            EmptyKwargsStorage
        }
    }
}

mod array {
    use crate::conversion::IntoPyObject;
    use crate::pycall::kwargs::array::ArrayKwargsStorage;
    use crate::pycall::kwargs::concat::FundamentalStorage;
    use crate::types::PyString;

    use super::KwargsStorageSelector;

    pub trait ArraySelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_kwargs_select(self, value: T) -> Self::Output;
    }

    impl<'py, K, V, const N: usize> ArraySelector<'py, [(K, V); N]>
        for &&&&&&&&&&KwargsStorageSelector<[(K, V); N]>
    where
        K: IntoPyObject<'py, Target = PyString>,
        V: IntoPyObject<'py>,
    {
        type Output = ArrayKwargsStorage<[(K, V); N]>;
        #[inline(always)]
        fn __py_unpack_kwargs_select(self, value: [(K, V); N]) -> ArrayKwargsStorage<[(K, V); N]> {
            ArrayKwargsStorage(value)
        }
    }

    impl<'a, 'py, K, V, const N: usize> ArraySelector<'py, &'a [(K, V); N]>
        for &&&&&&&&&&KwargsStorageSelector<&'a [(K, V); N]>
    where
        &'a K: IntoPyObject<'py, Target = PyString>,
        &'a V: IntoPyObject<'py>,
    {
        type Output = ArrayKwargsStorage<&'a [(K, V); N]>;
        #[inline(always)]
        fn __py_unpack_kwargs_select(
            self,
            value: &'a [(K, V); N],
        ) -> ArrayKwargsStorage<&'a [(K, V); N]> {
            ArrayKwargsStorage(value)
        }
    }

    impl<'a, 'py, K, V, const N: usize> ArraySelector<'py, &'a mut [(K, V); N]>
        for &&&&&&&&&&KwargsStorageSelector<&'a mut [(K, V); N]>
    where
        &'a K: IntoPyObject<'py, Target = PyString>,
        &'a V: IntoPyObject<'py>,
    {
        type Output = ArrayKwargsStorage<&'a mut [(K, V); N]>;
        #[inline(always)]
        fn __py_unpack_kwargs_select(
            self,
            value: &'a mut [(K, V); N],
        ) -> ArrayKwargsStorage<&'a mut [(K, V); N]> {
            ArrayKwargsStorage(value)
        }
    }
}

mod trusted_len {
    use crate::conversion::IntoPyObject;
    use crate::pycall::kwargs::concat::FundamentalStorage;
    use crate::pycall::kwargs::vec::{TrustedLenIterator, VecKwargsStorage};
    use crate::pycall::trusted_len::TrustedLen;
    use crate::types::PyString;

    use super::KwargsStorageSelector;

    pub trait TrustedLenSelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_kwargs_select(self, value: T) -> Self::Output;
    }

    impl<'py, T, K, V> TrustedLenSelector<'py, T> for &&&&&&&&KwargsStorageSelector<T>
    where
        T: IntoIterator,
        T::IntoIter: TrustedLen<Item = (K, V)>,
        K: IntoPyObject<'py, Target = PyString>,
        V: IntoPyObject<'py>,
    {
        type Output = VecKwargsStorage<TrustedLenIterator<T::IntoIter>>;
        #[inline(always)]
        fn __py_unpack_kwargs_select(
            self,
            value: T,
        ) -> VecKwargsStorage<TrustedLenIterator<T::IntoIter>> {
            VecKwargsStorage(TrustedLenIterator(value.into_iter()))
        }
    }
}

mod exact_size {
    use crate::conversion::IntoPyObject;
    use crate::pycall::kwargs::concat::FundamentalStorage;
    use crate::pycall::kwargs::vec::{ExactSizeIterator, VecKwargsStorage};
    use crate::types::PyString;

    use super::KwargsStorageSelector;

    pub trait ExactSizeIteratorSelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_kwargs_select(self, value: T) -> Self::Output;
    }

    impl<'py, T, K, V> ExactSizeIteratorSelector<'py, T> for &&&&&&&KwargsStorageSelector<T>
    where
        T: IntoIterator,
        T::IntoIter: std::iter::ExactSizeIterator<Item = (K, V)>,
        K: IntoPyObject<'py, Target = PyString>,
        V: IntoPyObject<'py>,
    {
        type Output = VecKwargsStorage<ExactSizeIterator<T::IntoIter>>;
        #[inline(always)]
        fn __py_unpack_kwargs_select(
            self,
            value: T,
        ) -> VecKwargsStorage<ExactSizeIterator<T::IntoIter>> {
            VecKwargsStorage(ExactSizeIterator::new(value.into_iter()))
        }
    }
}

mod any_iterator {
    use crate::conversion::IntoPyObject;
    use crate::pycall::kwargs::concat::FundamentalStorage;
    use crate::pycall::kwargs::unknown_size::{AnyIteratorKwargs, UnsizedKwargsStorage};
    use crate::types::PyString;

    use super::KwargsStorageSelector;

    pub trait AnyIteratorSelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_kwargs_select(self, value: T) -> Self::Output;
    }

    impl<'py, T, K, V> AnyIteratorSelector<'py, T> for &&&&&&KwargsStorageSelector<T>
    where
        T: IntoIterator<Item = (K, V)>,
        K: IntoPyObject<'py, Target = PyString>,
        V: IntoPyObject<'py>,
    {
        type Output = UnsizedKwargsStorage<AnyIteratorKwargs<T::IntoIter>>;
        #[inline(always)]
        fn __py_unpack_kwargs_select(
            self,
            value: T,
        ) -> UnsizedKwargsStorage<AnyIteratorKwargs<T::IntoIter>> {
            UnsizedKwargsStorage(AnyIteratorKwargs(value.into_iter()))
        }
    }
}

mod pydict {
    use crate::pycall::as_pyobject::AsPyObject;
    use crate::pycall::kwargs::concat::FundamentalStorage;
    use crate::pycall::kwargs::pyobjects::PyDictKwargsStorage;
    use crate::pycall::kwargs::unknown_size::UnsizedKwargsStorage;
    use crate::types::PyDict;

    use super::KwargsStorageSelector;

    pub trait PyDictSelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_kwargs_select(self, value: T) -> Self::Output;
    }

    impl<'py, T: AsPyObject<'py, PyObject = PyDict>> PyDictSelector<'py, T>
        for &&&&&KwargsStorageSelector<T>
    {
        type Output = UnsizedKwargsStorage<PyDictKwargsStorage<T>>;
        #[inline(always)]
        fn __py_unpack_kwargs_select(
            self,
            value: T,
        ) -> UnsizedKwargsStorage<PyDictKwargsStorage<T>> {
            UnsizedKwargsStorage(PyDictKwargsStorage::new(value))
        }
    }
}

mod any_pymapping {
    use crate::pycall::as_pyobject::AsPyObject;
    use crate::pycall::kwargs::concat::FundamentalStorage;
    use crate::pycall::kwargs::pyobjects::AnyPyMapping;
    use crate::pycall::kwargs::unknown_size::UnsizedKwargsStorage;

    use super::KwargsStorageSelector;

    pub trait AnyPyMappingSelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_kwargs_select(self, value: T) -> Self::Output;
    }

    impl<'py, T: AsPyObject<'py>> AnyPyMappingSelector<'py, T> for &&&KwargsStorageSelector<T> {
        type Output = UnsizedKwargsStorage<AnyPyMapping<T>>;
        #[inline(always)]
        fn __py_unpack_kwargs_select(self, value: T) -> UnsizedKwargsStorage<AnyPyMapping<T>> {
            UnsizedKwargsStorage(AnyPyMapping(value))
        }
    }
}

mod tuple {
    use crate::pycall::kwargs::array::{ArrayKwargsStorage, Tuple};
    use crate::pycall::kwargs::concat::FundamentalStorage;

    use super::KwargsStorageSelector;

    pub trait TupleSelector<'py, T> {
        type Output: FundamentalStorage<'py>;
        fn __py_unpack_kwargs_select(self, value: T) -> Self::Output;
    }

    impl<'py, T: Tuple<'py>> TupleSelector<'py, T> for &&KwargsStorageSelector<T> {
        type Output = ArrayKwargsStorage<T>;
        #[inline(always)]
        fn __py_unpack_kwargs_select(self, value: T) -> ArrayKwargsStorage<T> {
            ArrayKwargsStorage(value)
        }
    }
}
