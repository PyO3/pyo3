//! We have 6 fundamental storages: `ArrayArgsStorage`, `EmptyArgsStorage`, `ExistingArgListSlice`,
//! `UnsizedArgsStorage`, `AppendEmptyArgForVectorcall`, and `VecArgsStorage`. We need to define all
//! combinations between them, that means 5^2+5=30 impls (`AppendEmptyArgForVectorcall` is special:
//! it only concat when it is LHS, and it cannot concat with itself). Fortunately, macros can help with that.

use super::array::ArrayArgsStorage;
use super::empty::EmptyArgsStorage;
use super::existing::{
    ExistingArgListSlice, ExistingArgListSliceTrait, ExistingArgListVecStorageAdapter,
};
use super::unknown_size::{SizedToUnsizedStorage, UnsizedArgsStorage};
use super::vec::VecArgsStorage;
use super::{AppendEmptyArgForVectorcall, ConcatStorages, ResolveArgs};

/// A storage that can be concatenated with other storages.
///
/// A storage can be a remark how to handle some unpacked argument (e.g. tuples implement `ResolveArgs`),
/// or it can also carry instructions how to create the whole list of arguments (e.g. an `ArrayArgsStorage`).
/// This trait signifies the latter.
pub trait FundamentalStorage<'py>: ResolveArgs<'py> {}
macro_rules! impl_fundamental_storage {
    ( $($storage:ident)+ ) => {
        $(
            impl<'py, T> FundamentalStorage<'py> for $storage<T> where
                $storage<T>: ResolveArgs<'py>
            {
            }
        )+
    };
}
impl_fundamental_storage!(ArrayArgsStorage VecArgsStorage UnsizedArgsStorage ExistingArgListSlice);
impl<'py> FundamentalStorage<'py> for EmptyArgsStorage {}
impl<'py> FundamentalStorage<'py> for AppendEmptyArgForVectorcall {}

pub trait ConcatArgsStorages<'py, Rhs: FundamentalStorage<'py>>: FundamentalStorage<'py> {
    type Output: ResolveArgs<'py>;
    fn concat(self, other: Rhs) -> Self::Output;
}

macro_rules! define_concat {
    (
        $(
            $storage1:ident + $storage2:ident = $result:ident
        )+
    ) => {
        $(
            impl<'py, A, B> ConcatArgsStorages<'py, $storage2<B>> for $storage1<A>
            where
                $storage1<A>: ResolveArgs<'py>,
                $storage2<B>: ResolveArgs<'py>,
                $result<ConcatStorages<$storage1<A>, $storage2<B>>>: ResolveArgs<'py>,
            {
                type Output = $result<ConcatStorages<$storage1<A>, $storage2<B>>>;
                #[inline(always)]
                fn concat(self, other: $storage2<B>) -> Self::Output {
                    $result(ConcatStorages(self, other))
                }
            }
        )+
    };
}
define_concat!(
    ArrayArgsStorage + ArrayArgsStorage = ArrayArgsStorage
    ArrayArgsStorage + VecArgsStorage = VecArgsStorage
    VecArgsStorage + ArrayArgsStorage = VecArgsStorage
    VecArgsStorage + VecArgsStorage = VecArgsStorage
);

macro_rules! define_concat_empty {
    ( $( $other:ident )+ ) => {
        $(
            impl<'py, T> ConcatArgsStorages<'py, $other<T>> for EmptyArgsStorage
            where
                $other<T>: ResolveArgs<'py>,
            {
                type Output = $other<T>;
                #[inline(always)]
                fn concat(self, other: $other<T>) -> Self::Output {
                    other
                }
            }
            impl<'py, T> ConcatArgsStorages<'py, EmptyArgsStorage> for $other<T>
            where
                $other<T>: ResolveArgs<'py>,
            {
                type Output = $other<T>;
                #[inline(always)]
                fn concat(self, _other: EmptyArgsStorage) -> Self::Output {
                    self
                }
            }
        )+
    };
}
define_concat_empty!(
    ArrayArgsStorage VecArgsStorage UnsizedArgsStorage ExistingArgListSlice
);
impl<'py> ConcatArgsStorages<'py, EmptyArgsStorage> for EmptyArgsStorage {
    #[inline(always)]
    fn concat(self, _other: EmptyArgsStorage) -> Self::Output {
        EmptyArgsStorage
    }
    type Output = EmptyArgsStorage;
}

macro_rules! define_concat_existing {
    ( $( $other:ident )+ ) => {
        $(
            impl<'py, S, T> ConcatArgsStorages<'py, $other<T>> for ExistingArgListSlice<S>
            where
                ExistingArgListSlice<S>: ResolveArgs<'py>,
                $other<T>: ResolveArgs<'py>,
                VecArgsStorage<ExistingArgListVecStorageAdapter<S>>: ConcatArgsStorages<'py, $other<T>>,
            {
                type Output = <VecArgsStorage<ExistingArgListVecStorageAdapter<S>> as ConcatArgsStorages<'py, $other<T>>>::Output;
                #[inline(always)]
                fn concat(self, other: $other<T>) -> Self::Output {
                    <VecArgsStorage<ExistingArgListVecStorageAdapter<S>> as ConcatArgsStorages<'py, $other<T>>>::concat(
                        VecArgsStorage(ExistingArgListVecStorageAdapter(self)), other)
                }
            }
            impl<'py, S, T> ConcatArgsStorages<'py, ExistingArgListSlice<S>> for $other<T>
            where
                ExistingArgListSlice<S>: ResolveArgs<'py>,
                VecArgsStorage<ExistingArgListVecStorageAdapter<S>>: ResolveArgs<'py>,
                $other<T>: ResolveArgs<'py>,
                $other<T>: ConcatArgsStorages<'py, VecArgsStorage<ExistingArgListVecStorageAdapter<S>>>,
            {
                type Output = <$other<T> as ConcatArgsStorages<'py, VecArgsStorage<ExistingArgListVecStorageAdapter<S>>>>::Output;
                #[inline(always)]
                fn concat(self, other: ExistingArgListSlice<S>) -> Self::Output {
                    <$other<T> as ConcatArgsStorages<'py, VecArgsStorage<ExistingArgListVecStorageAdapter<S>>>>::concat(
                        self, VecArgsStorage(ExistingArgListVecStorageAdapter(other)))
                }
            }
        )+
    };
}
define_concat_existing!(ArrayArgsStorage VecArgsStorage UnsizedArgsStorage);
impl<'py, A, B> ConcatArgsStorages<'py, ExistingArgListSlice<B>> for ExistingArgListSlice<A>
where
    ExistingArgListSlice<A>: ResolveArgs<'py>,
    ExistingArgListSlice<B>: ResolveArgs<'py>,
    VecArgsStorage<
        ConcatStorages<
            VecArgsStorage<ExistingArgListVecStorageAdapter<A>>,
            VecArgsStorage<ExistingArgListVecStorageAdapter<B>>,
        >,
    >: ResolveArgs<'py>,
{
    type Output = VecArgsStorage<
        ConcatStorages<
            VecArgsStorage<ExistingArgListVecStorageAdapter<A>>,
            VecArgsStorage<ExistingArgListVecStorageAdapter<B>>,
        >,
    >;
    #[inline(always)]
    fn concat(self, other: ExistingArgListSlice<B>) -> Self::Output {
        VecArgsStorage(ConcatStorages(
            VecArgsStorage(ExistingArgListVecStorageAdapter(self)),
            VecArgsStorage(ExistingArgListVecStorageAdapter(other)),
        ))
    }
}

macro_rules! define_concat_sized_to_unsized {
    (
        $( $other:ident )+
    ) => {
        $(
            impl<'py, T, U> ConcatArgsStorages<'py, $other<T>> for UnsizedArgsStorage<U>
            where
                UnsizedArgsStorage<U>: ResolveArgs<'py>,
                $other<T>: ResolveArgs<'py>,
                UnsizedArgsStorage<ConcatStorages<UnsizedArgsStorage<U>, UnsizedArgsStorage<SizedToUnsizedStorage<$other<T>>>>>: ResolveArgs<'py>,
            {
                type Output = UnsizedArgsStorage<ConcatStorages<UnsizedArgsStorage<U>, UnsizedArgsStorage<SizedToUnsizedStorage<$other<T>>>>>;
                #[inline(always)]
                fn concat(self, other: $other<T>) -> Self::Output {
                    UnsizedArgsStorage(ConcatStorages(self, UnsizedArgsStorage(SizedToUnsizedStorage(other))))
                }
            }
            impl<'py, T, U> ConcatArgsStorages<'py, UnsizedArgsStorage<U>> for $other<T>
            where
                UnsizedArgsStorage<U>: ResolveArgs<'py>,
                $other<T>: ResolveArgs<'py>,
                UnsizedArgsStorage<ConcatStorages<UnsizedArgsStorage<SizedToUnsizedStorage<$other<T>>>, UnsizedArgsStorage<U>>>: ResolveArgs<'py>,
            {
                type Output = UnsizedArgsStorage<ConcatStorages<UnsizedArgsStorage<SizedToUnsizedStorage<$other<T>>>, UnsizedArgsStorage<U>>>;
                #[inline(always)]
                fn concat(self, other: UnsizedArgsStorage<U>) -> Self::Output {
                    UnsizedArgsStorage(ConcatStorages(UnsizedArgsStorage(SizedToUnsizedStorage(self)), other))
                }
            }
        )+
    };
}
define_concat_sized_to_unsized!(ArrayArgsStorage VecArgsStorage);
impl<'py, A, B> ConcatArgsStorages<'py, UnsizedArgsStorage<B>> for UnsizedArgsStorage<A>
where
    UnsizedArgsStorage<A>: ResolveArgs<'py>,
    UnsizedArgsStorage<B>: ResolveArgs<'py>,
    UnsizedArgsStorage<ConcatStorages<UnsizedArgsStorage<A>, UnsizedArgsStorage<B>>>:
        ResolveArgs<'py>,
{
    type Output = UnsizedArgsStorage<ConcatStorages<UnsizedArgsStorage<A>, UnsizedArgsStorage<B>>>;
    #[inline(always)]
    fn concat(self, other: UnsizedArgsStorage<B>) -> Self::Output {
        UnsizedArgsStorage(ConcatStorages(self, other))
    }
}

macro_rules! define_concat_append_empty_arg_for_vectorcall {
    ( $( ( $($generic:ident)? ) $other:ident )+ ) => {
        $(
            impl<'py, $($generic)?> ConcatArgsStorages<'py, $other<$($generic)?>> for AppendEmptyArgForVectorcall
            where
                $other<$($generic)?>: ResolveArgs<'py>,
                ArrayArgsStorage<AppendEmptyArgForVectorcall>: ConcatArgsStorages<'py, $other<$($generic)?>>,
            {
                type Output = <ArrayArgsStorage<AppendEmptyArgForVectorcall> as ConcatArgsStorages<'py, $other<$($generic)?>>>::Output;
                #[inline(always)]
                fn concat(self, other: $other<$($generic)?>) -> Self::Output {
                    <ArrayArgsStorage<AppendEmptyArgForVectorcall> as ConcatArgsStorages<'py, $other<$($generic)?>>>::concat(ArrayArgsStorage(self), other)
                }
            }
        )+
    };
}
define_concat_append_empty_arg_for_vectorcall!(
    (T) ArrayArgsStorage (T) VecArgsStorage (T) UnsizedArgsStorage
    () EmptyArgsStorage
);
impl<'py, S> ConcatArgsStorages<'py, ExistingArgListSlice<S>> for AppendEmptyArgForVectorcall
where
    S: ExistingArgListSliceTrait,
{
    type Output = ExistingArgListSlice<S>;
    #[inline(always)]
    fn concat(self, other: ExistingArgListSlice<S>) -> Self::Output {
        other
    }
}
