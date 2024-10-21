use crate::types::PyTuple;
use crate::{ffi, Borrowed, PyResult, Python};

use super::args::{self, ArgumentsOffsetFlag, ConcatStorages, ResolveArgs};
use super::kwargs::{self, ExistingNames, ResolveKwargs};
use super::PPPyObject;

pub struct KwargsArgsAdapter<'a, 'py, Kwargs> {
    pub(super) kwargs: Kwargs,
    pub(super) kwargs_tuple: Borrowed<'a, 'py, PyTuple>,
}

impl<'py, Kwargs: ResolveKwargs<'py>> ResolveArgs<'py> for KwargsArgsAdapter<'_, 'py, Kwargs> {
    type RawStorage = Kwargs::RawStorage;
    type Guard = Kwargs::Guard;
    fn init(
        self,
        _py: Python<'py>,
        storage: PPPyObject,
        _base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        let len = self.kwargs.len();
        self.kwargs.init(
            storage,
            self.kwargs_tuple,
            &mut 0,
            &mut ExistingNames::new(len),
        )
    }
    fn len(&self) -> usize {
        self.kwargs.len()
    }
    fn write_to_tuple(
        self,
        _tuple: Borrowed<'_, 'py, PyTuple>,
        _index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        unreachable!("kwargs-args adapters are only used for vectorcall")
    }
    fn write_initialized_to_tuple(
        _tuple: Borrowed<'_, 'py, PyTuple>,
        _guard: Self::Guard,
        _raw_storage: &mut PPPyObject,
        _index: &mut ffi::Py_ssize_t,
    ) {
        unreachable!("kwargs-args adapters are only used for vectorcall")
    }
    fn has_known_size(&self) -> bool {
        self.kwargs.has_known_size()
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = Kwargs::IS_EMPTY;
    const IS_ONE: bool = false;
    const USE_STACK_FOR_SMALL_LEN: bool = true;
}

pub struct KwargsArgsNoNamesAdapter<Kwargs>(Kwargs);

impl<'py, Kwargs: ResolveKwargs<'py>> ResolveArgs<'py> for KwargsArgsNoNamesAdapter<Kwargs> {
    type RawStorage = Kwargs::RawStorage;
    type Guard = Kwargs::Guard;
    fn init(
        self,
        py: Python<'py>,
        storage: PPPyObject,
        _base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        self.0.init_no_names(py, storage)
    }
    fn len(&self) -> usize {
        self.0.len()
    }
    fn write_to_tuple(
        self,
        _tuple: Borrowed<'_, 'py, PyTuple>,
        _index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        unreachable!("kwargs-args adapters are only used for vectorcall")
    }
    fn write_initialized_to_tuple(
        _tuple: Borrowed<'_, 'py, PyTuple>,
        _guard: Self::Guard,
        _raw_storage: &mut PPPyObject,
        _index: &mut ffi::Py_ssize_t,
    ) {
        unreachable!("kwargs-args adapters are only used for vectorcall")
    }
    fn has_known_size(&self) -> bool {
        self.0.has_known_size()
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = Kwargs::IS_EMPTY;
    const IS_ONE: bool = false;
    const USE_STACK_FOR_SMALL_LEN: bool = true;
}

// We have 5 fundamental args storages (`AppendEmptyArgForVectorcall` isn't counted because
// it is always converted to `ArrayArgsStorage` before combined), and 5 fundamental kwargs
// storages. That means 5*5 = 25 combinations.

pub trait CombineArgsKwargs<'a, 'py, Kwargs>
where
    Self: args::FundamentalStorage<'py>,
    Kwargs: kwargs::FundamentalStorage<'py>,
{
    type Output: args::FundamentalStorage<'py>;
    fn combine(self, kwargs: KwargsArgsAdapter<'a, 'py, Kwargs>) -> Self::Output;
    type OutputNoNames: args::FundamentalStorage<'py>;
    fn combine_no_names(self, kwargs: Kwargs) -> Self::OutputNoNames;
}

macro_rules! define_combine {
    (
        $(
            $args:ident + (( $($py:lifetime)? ) $($b:ident)?) $kwargs:ident = $result:ident
        )+
    ) => {
        $(
            impl<'a, 'py, A, $($b)?> CombineArgsKwargs<'a, 'py, kwargs::$kwargs<$($py,)? $($b)?>> for args::$args<A>
            where
                args::$args<A>: args::ResolveArgs<'py>,
                kwargs::$kwargs<$($py,)? $($b)?>: kwargs::FundamentalStorage<'py>,
                args::$result<ConcatStorages<args::$args<A>, args::$args<KwargsArgsAdapter<'a, 'py, kwargs::$kwargs<$($py,)? $($b)?>>>>>: args::ResolveArgs<'py>,
                args::$result<ConcatStorages<args::$args<A>, args::$args<KwargsArgsNoNamesAdapter<kwargs::$kwargs<$($py,)? $($b)?>>>>>: args::ResolveArgs<'py>,
            {
                type Output = args::$result<ConcatStorages<args::$args<A>, args::$args<KwargsArgsAdapter<'a, 'py, kwargs::$kwargs<$($py,)? $($b)?>>>>>;
                #[inline(always)]
                fn combine(self, kwargs: KwargsArgsAdapter<'a, 'py, kwargs::$kwargs<$($py,)? $($b)?>>) -> Self::Output {
                    args::$result(ConcatStorages(self, args::$args(kwargs)))
                }
                type OutputNoNames = args::$result<ConcatStorages<args::$args<A>, args::$args<KwargsArgsNoNamesAdapter<kwargs::$kwargs<$($py,)? $($b)?>>>>>;
                #[inline(always)]
                fn combine_no_names(self, kwargs: kwargs::$kwargs<$($py,)? $($b)?>) -> Self::OutputNoNames {
                    args::$result(ConcatStorages(self, args::$args(KwargsArgsNoNamesAdapter(kwargs))))
                }
            }
        )+
    };
}
define_combine!(
    ArrayArgsStorage + (() B) ArrayKwargsStorage = ArrayArgsStorage
    VecArgsStorage + (() B) ArrayKwargsStorage = VecArgsStorage
    ArrayArgsStorage + (() B) VecKwargsStorage = VecArgsStorage
    ArrayArgsStorage + (('py) B) KnownKwargsStorage = ArrayArgsStorage
    VecArgsStorage + (('py) B) KnownKwargsStorage = VecArgsStorage
    VecArgsStorage + (() B) VecKwargsStorage = VecArgsStorage
    // The following will never be used, since we check for empty kwargs and pass NULL.
    // But they need to be here to please the compiler.
    ArrayArgsStorage + (()) EmptyKwargsStorage = ArrayArgsStorage
    VecArgsStorage + (()) EmptyKwargsStorage = ArrayArgsStorage
    // The following will never really be unsized used, since we check for unknown size kwargs
    // and use normal (tuple and dict) calling convention. But they can be dynamically sized
    // if `PyDict`.
    ArrayArgsStorage + (() B) UnsizedKwargsStorage = VecArgsStorage
    VecArgsStorage + (() B) UnsizedKwargsStorage = VecArgsStorage
);

macro_rules! define_combine_empty_args {
    (
        $(
            ( $($py:lifetime)? ) $kwargs:ident = $result:ident
        )+
    ) => {
        $(
            impl<'a, 'py, T> CombineArgsKwargs<'a, 'py, kwargs::$kwargs<$($py,)? T>> for args::EmptyArgsStorage
            where
                kwargs::$kwargs<$($py,)? T>: kwargs::FundamentalStorage<'py>,
                args::$result<KwargsArgsAdapter<'a, 'py, kwargs::$kwargs<$($py,)? T>>>: args::ResolveArgs<'py>,
                args::$result<KwargsArgsNoNamesAdapter<kwargs::$kwargs<$($py,)? T>>>: args::ResolveArgs<'py>,
            {
                type Output = args::$result<KwargsArgsAdapter<'a, 'py, kwargs::$kwargs<$($py,)? T>>>;
                #[inline(always)]
                fn combine(self, kwargs: KwargsArgsAdapter<'a, 'py, kwargs::$kwargs<$($py,)? T>>) -> Self::Output {
                    args::$result(kwargs)
                }
                type OutputNoNames = args::$result<KwargsArgsNoNamesAdapter<kwargs::$kwargs<$($py,)? T>>>;
                #[inline(always)]
                fn combine_no_names(self, kwargs: kwargs::$kwargs<$($py,)? T>) -> Self::OutputNoNames {
                    args::$result(KwargsArgsNoNamesAdapter(kwargs))
                }
            }
        )+
    };
}
define_combine_empty_args!(
    () ArrayKwargsStorage = ArrayArgsStorage
    ('py) KnownKwargsStorage = ArrayArgsStorage
    () VecKwargsStorage = VecArgsStorage
    // The following will never be used, since we check for unknown size kwargs and use normal
    // (tuple and dict) calling convention. But it needs to be here to please the compiler.
    () UnsizedKwargsStorage = VecArgsStorage
);
// The following will never be used, since we check for empty kwargs and pass NULL.
// But it needs to be here to please the compiler.
impl<'a, 'py> CombineArgsKwargs<'a, 'py, kwargs::EmptyKwargsStorage> for args::EmptyArgsStorage {
    type Output = args::EmptyArgsStorage;
    #[inline(always)]
    fn combine(
        self,
        _kwargs: KwargsArgsAdapter<'a, 'py, kwargs::EmptyKwargsStorage>,
    ) -> Self::Output {
        args::EmptyArgsStorage
    }
    type OutputNoNames = args::EmptyArgsStorage;
    #[inline(always)]
    fn combine_no_names(self, _kwargs: kwargs::EmptyKwargsStorage) -> Self::OutputNoNames {
        args::EmptyArgsStorage
    }
}

macro_rules! define_combine_sized_to_unsized {
    (
        $(
            $args:ident + (( $($py:lifetime)? ) $($b:ident)?) $kwargs:ident
        )+
    ) => {
        $(
            impl<'a, 'py, A, $($b)?> CombineArgsKwargs<'a, 'py, kwargs::$kwargs<$($py,)? $($b)?>> for args::$args<A>
            where
                args::$args<A>: args::ResolveArgs<'py>,
                kwargs::$kwargs<$($py,)? $($b)?>: kwargs::FundamentalStorage<'py>,
                args::UnsizedArgsStorage<ConcatStorages<
                    args::$args<A>,
                    args::UnsizedArgsStorage<args::SizedToUnsizedStorage<KwargsArgsAdapter<'a, 'py, kwargs::$kwargs<$($py,)? $($b)?>>>>,
                >>: args::ResolveArgs<'py>,
                args::UnsizedArgsStorage<ConcatStorages<
                    args::$args<A>,
                    args::UnsizedArgsStorage<args::SizedToUnsizedStorage<KwargsArgsNoNamesAdapter<kwargs::$kwargs<$($py,)? $($b)?>>>>,
                >>: args::ResolveArgs<'py>,
            {
                type Output = args::UnsizedArgsStorage<ConcatStorages<
                    args::$args<A>,
                    args::UnsizedArgsStorage<args::SizedToUnsizedStorage<KwargsArgsAdapter<'a, 'py, kwargs::$kwargs<$($py,)? $($b)?>>>>,
                >>;
                #[inline(always)]
                fn combine(self, kwargs: KwargsArgsAdapter<'a, 'py, kwargs::$kwargs<$($py,)? $($b)?>>) -> Self::Output {
                    args::UnsizedArgsStorage(ConcatStorages(self, args::UnsizedArgsStorage(args::SizedToUnsizedStorage(kwargs))))
                }
                type OutputNoNames = args::UnsizedArgsStorage<ConcatStorages<
                    args::$args<A>,
                    args::UnsizedArgsStorage<args::SizedToUnsizedStorage<KwargsArgsNoNamesAdapter<kwargs::$kwargs<$($py,)? $($b)?>>>>,
                >>;
                #[inline(always)]
                fn combine_no_names(self, kwargs: kwargs::$kwargs<$($py,)? $($b)?>) -> Self::OutputNoNames {
                    args::UnsizedArgsStorage(ConcatStorages(self, args::UnsizedArgsStorage(args::SizedToUnsizedStorage(KwargsArgsNoNamesAdapter(kwargs)))))
                }
            }
        )+
    };
}
define_combine_sized_to_unsized!(
    UnsizedArgsStorage + (() B) ArrayKwargsStorage
    UnsizedArgsStorage + (() B) VecKwargsStorage
    UnsizedArgsStorage + (('py) B) KnownKwargsStorage
    // The following will never be used, since we check for unknown size kwargs and use normal
    // (tuple and dict) calling convention. But it needs to be here to please the compiler.
    UnsizedArgsStorage + (() B) UnsizedKwargsStorage
    // The following will never be used, since we check for empty kwargs and pass NULL.
    // But it needs to be here to please the compiler.
    UnsizedArgsStorage + (()) EmptyKwargsStorage
);

macro_rules! define_combine_existing {
    (
        $(
            (( $($py:lifetime)? ) $($t:ident)?) $kwargs:ident
        )+
    ) => {
        $(
            impl<'a, 'py, S, $($t)?> CombineArgsKwargs<'a, 'py, kwargs::$kwargs<$($py,)? $($t)?>> for args::ExistingArgListSlice<S>
            where
                S: args::ExistingArgListSliceTrait,
                kwargs::$kwargs<$($py,)? $($t)?>: kwargs::FundamentalStorage<'py>,
                args::VecArgsStorage<args::ExistingArgListSlice<S>>: CombineArgsKwargs<'a, 'py, kwargs::$kwargs<$($py,)? $($t)?>>,
            {
                type Output = <
                    args::VecArgsStorage<args::ExistingArgListSlice<S>>
                        as CombineArgsKwargs<'a, 'py, kwargs::$kwargs<$($py,)? $($t)?>>
                >::Output;
                #[inline(always)]
                fn combine(self, kwargs: KwargsArgsAdapter<'a, 'py, kwargs::$kwargs<$($py,)? $($t)?>>) -> Self::Output {
                    <
                        args::VecArgsStorage<args::ExistingArgListSlice<S>>
                            as CombineArgsKwargs<'a, 'py, kwargs::$kwargs<$($py,)? $($t)?>>
                    >::combine(args::VecArgsStorage(self), kwargs)
                }
                type OutputNoNames = <
                    args::VecArgsStorage<args::ExistingArgListSlice<S>>
                        as CombineArgsKwargs<'a, 'py, kwargs::$kwargs<$($py,)? $($t)?>>
                >::OutputNoNames;
                #[inline(always)]
                fn combine_no_names(self, kwargs: kwargs::$kwargs<$($py,)? $($t)?>) -> Self::OutputNoNames {
                    <
                        args::VecArgsStorage<args::ExistingArgListSlice<S>>
                            as CombineArgsKwargs<'a, 'py, kwargs::$kwargs<$($py,)? $($t)?>>
                    >::combine_no_names(args::VecArgsStorage(self), kwargs)
                }
            }
        )+
    };
}
define_combine_existing!(
    (() T) ArrayKwargsStorage
    (() T) VecKwargsStorage
    (() T) UnsizedKwargsStorage
    (('py) T) KnownKwargsStorage
    (()) EmptyKwargsStorage
);
