//! We have 5 fundamental storages: `ArrayKwargsStorage`, `EmptyKwargsStorage`, `KnownKwargsStorage`,
//! `UnsizedKwargsStorage`, and `VecKwargsStorage`. We need to define all
//! combinations between them, that means 4^2+4=20 impls (`KnownKwargsStorage` is special:
//! it only concat when it is LHS, and it cannot concat with itself). Fortunately, macros can help with that.

use super::array::ArrayKwargsStorage;
use super::empty::EmptyKwargsStorage;
use super::known::KnownKwargsStorage;
use super::unknown_size::UnsizedKwargsStorage;
use super::vec::VecKwargsStorage;
use super::{ConcatStorages, ResolveKwargs, TypeLevelPyObjectListTrait};

/// A storage that can be concatenated with other storages.
///
/// A storage can be a remark how to handle some unpacked argument (e.g. tuples implement `ResolveKwargs`),
/// or it can also carry instructions how to create the whole list of arguments (e.g. an `ArrayKwargsStorage`).
/// This trait signifies the latter.
pub trait FundamentalStorage<'py>: ResolveKwargs<'py> {}
macro_rules! impl_fundamental_storage {
    ( $($storage:ident)+ ) => {
        $(
            impl<'py, T> FundamentalStorage<'py> for $storage<T> where
                $storage<T>: ResolveKwargs<'py>
            {
            }
        )+
    };
}
impl_fundamental_storage!(ArrayKwargsStorage VecKwargsStorage UnsizedKwargsStorage);
impl<'py> FundamentalStorage<'py> for EmptyKwargsStorage {}
impl<'py, Values: TypeLevelPyObjectListTrait<'py>> FundamentalStorage<'py>
    for KnownKwargsStorage<'py, Values>
{
}

pub trait ConcatKwargsStorages<'py, Rhs: FundamentalStorage<'py>>: FundamentalStorage<'py> {
    type Output: ResolveKwargs<'py>;
    fn concat(self, other: Rhs) -> Self::Output;
}

macro_rules! define_concat {
    (
        $(
            $storage1:ident + $storage2:ident = $result:ident
        )+
    ) => {
        $(
            impl<'py, A, B> ConcatKwargsStorages<'py, $storage2<B>> for $storage1<A>
            where
                $storage1<A>: ResolveKwargs<'py>,
                $storage2<B>: ResolveKwargs<'py>,
                $result<ConcatStorages<$storage1<A>, $storage2<B>>>: ResolveKwargs<'py>,
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
    ArrayKwargsStorage + ArrayKwargsStorage = ArrayKwargsStorage
    ArrayKwargsStorage + VecKwargsStorage = VecKwargsStorage
    VecKwargsStorage + ArrayKwargsStorage = VecKwargsStorage
    VecKwargsStorage + VecKwargsStorage = VecKwargsStorage
);

macro_rules! define_concat_empty {
    ( $( $other:ident )+ ) => {
        $(
            impl<'py, T> ConcatKwargsStorages<'py, $other<T>> for EmptyKwargsStorage
            where
                $other<T>: ResolveKwargs<'py>,
            {
                type Output = $other<T>;
                #[inline(always)]
                fn concat(self, other: $other<T>) -> Self::Output {
                    other
                }
            }
            impl<'py, T> ConcatKwargsStorages<'py, EmptyKwargsStorage> for $other<T>
            where
                $other<T>: ResolveKwargs<'py>,
            {
                type Output = $other<T>;
                #[inline(always)]
                fn concat(self, _other: EmptyKwargsStorage) -> Self::Output {
                    self
                }
            }
        )+
    };
}
define_concat_empty!(
    ArrayKwargsStorage VecKwargsStorage UnsizedKwargsStorage
);
impl<'py> ConcatKwargsStorages<'py, EmptyKwargsStorage> for EmptyKwargsStorage {
    #[inline(always)]
    fn concat(self, _other: EmptyKwargsStorage) -> Self::Output {
        EmptyKwargsStorage
    }
    type Output = EmptyKwargsStorage;
}
impl<'py, Values> ConcatKwargsStorages<'py, EmptyKwargsStorage> for KnownKwargsStorage<'py, Values>
where
    Values: TypeLevelPyObjectListTrait<'py>,
{
    #[inline(always)]
    fn concat(self, _other: EmptyKwargsStorage) -> Self::Output {
        self
    }
    type Output = KnownKwargsStorage<'py, Values>;
}

macro_rules! define_concat_known {
    ( $( $other:ident )+ ) => {
        $(
            impl<'py, T, Values> ConcatKwargsStorages<'py, $other<T>> for KnownKwargsStorage<'py, Values>
            where
                Values: TypeLevelPyObjectListTrait<'py>,
                $other<T>: ResolveKwargs<'py>,
                ArrayKwargsStorage<KnownKwargsStorage<'py, Values>>: ConcatKwargsStorages<'py, $other<T>>,
            {
                type Output = <ArrayKwargsStorage<KnownKwargsStorage<'py, Values>> as ConcatKwargsStorages<'py, $other<T>>>::Output;
                #[inline(always)]
                fn concat(self, other: $other<T>) -> Self::Output {
                    <ArrayKwargsStorage<KnownKwargsStorage<'py, Values>> as ConcatKwargsStorages<'py, $other<T>>>::concat(
                        ArrayKwargsStorage(self), other)
                }
            }
        )+
    };
}
define_concat_known!(ArrayKwargsStorage VecKwargsStorage UnsizedKwargsStorage);

macro_rules! define_concat_sized_to_unsized {
    (
        $( $other:ident )+
    ) => {
        $(
            impl<'py, T, U> ConcatKwargsStorages<'py, $other<T>> for UnsizedKwargsStorage<U>
            where
                UnsizedKwargsStorage<U>: ResolveKwargs<'py>,
                $other<T>: ResolveKwargs<'py>,
            {
                type Output = UnsizedKwargsStorage<ConcatStorages<UnsizedKwargsStorage<U>, $other<T>>>;
                #[inline(always)]
                fn concat(self, other: $other<T>) -> Self::Output {
                    UnsizedKwargsStorage(ConcatStorages(self, other))
                }
            }
            impl<'py, T, U> ConcatKwargsStorages<'py, UnsizedKwargsStorage<U>> for $other<T>
            where
                UnsizedKwargsStorage<U>: ResolveKwargs<'py>,
                $other<T>: ResolveKwargs<'py>,
            {
                type Output = UnsizedKwargsStorage<ConcatStorages<$other<T>, UnsizedKwargsStorage<U>>>;
                #[inline(always)]
                fn concat(self, other: UnsizedKwargsStorage<U>) -> Self::Output {
                    UnsizedKwargsStorage(ConcatStorages(self, other))
                }
            }
        )+
    };
}
define_concat_sized_to_unsized!(ArrayKwargsStorage VecKwargsStorage);
impl<'py, A, B> ConcatKwargsStorages<'py, UnsizedKwargsStorage<B>> for UnsizedKwargsStorage<A>
where
    UnsizedKwargsStorage<A>: ResolveKwargs<'py>,
    UnsizedKwargsStorage<B>: ResolveKwargs<'py>,
    UnsizedKwargsStorage<ConcatStorages<UnsizedKwargsStorage<A>, UnsizedKwargsStorage<B>>>:
        ResolveKwargs<'py>,
{
    type Output =
        UnsizedKwargsStorage<ConcatStorages<UnsizedKwargsStorage<A>, UnsizedKwargsStorage<B>>>;
    #[inline(always)]
    fn concat(self, other: UnsizedKwargsStorage<B>) -> Self::Output {
        UnsizedKwargsStorage(ConcatStorages(self, other))
    }
}
