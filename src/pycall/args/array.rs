use std::mem::MaybeUninit;

use crate::conversion::IntoPyObject;
use crate::types::PyTuple;
use crate::{ffi, Borrowed, BoundObject, PyResult, Python};

use super::helpers::{
    concat_known_sized, write_iter_to_tuple, write_raw_storage_to_tuple, DropManyGuard,
    DropOneGuard, WriteToTuple,
};
use super::{
    ArgumentsOffsetFlag, ConcatArrays, ConcatStorages, PPPyObject, RawStorage, ResolveArgs,
};

pub struct ArrayArgsStorage<T>(pub(in super::super) T);

impl<'py, T: ResolveArgs<'py>> ResolveArgs<'py> for ArrayArgsStorage<T>
where
    T::RawStorage: for<'a> RawStorage<InitParam<'a> = PPPyObject> + 'static,
{
    type RawStorage = T::RawStorage;
    type Guard = T::Guard;
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: PPPyObject,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        self.0.init(py, storage, base_storage)
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len()
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        self.0.write_to_tuple(tuple, index)
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        tuple: Borrowed<'_, 'py, PyTuple>,
        guard: Self::Guard,
        raw_storage: &mut PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) {
        T::write_initialized_to_tuple(tuple, guard, raw_storage, index)
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    #[inline(always)]
    fn as_pytuple(&self, py: Python<'py>) -> Option<Borrowed<'_, 'py, PyTuple>> {
        self.0.as_pytuple(py)
    }
    const IS_EMPTY: bool = T::IS_EMPTY;
    const IS_ONE: bool = T::IS_ONE;
    const USE_STACK_FOR_SMALL_LEN: bool = T::USE_STACK_FOR_SMALL_LEN;
}

impl<'py, A, B> ResolveArgs<'py> for ArrayArgsStorage<ConcatStorages<A, B>>
where
    A: ResolveArgs<'py>,
    B: ResolveArgs<'py>,
    A::RawStorage: for<'a> RawStorage<InitParam<'a> = PPPyObject> + 'static,
    B::RawStorage: for<'a> RawStorage<InitParam<'a> = PPPyObject> + 'static,
{
    type RawStorage = MaybeUninit<ConcatArrays<A::RawStorage, B::RawStorage>>;
    type Guard = (A::Guard, B::Guard);
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: PPPyObject,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        concat_known_sized(self.0 .0, self.0 .1, py, storage, base_storage)
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.0 .0.len() + self.0 .1.len()
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        self.0 .0.write_to_tuple(tuple, index)?;
        self.0 .1.write_to_tuple(tuple, index)
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        tuple: Borrowed<'_, 'py, PyTuple>,
        guard: Self::Guard,
        raw_storage: &mut PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) {
        A::write_initialized_to_tuple(tuple, guard.0, raw_storage, index);
        B::write_initialized_to_tuple(tuple, guard.1, raw_storage, index);
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = A::IS_EMPTY && B::IS_EMPTY;
    const IS_ONE: bool = (A::IS_EMPTY && B::IS_ONE) || (A::IS_ONE && B::IS_EMPTY);
    const USE_STACK_FOR_SMALL_LEN: bool = false;
}

impl<'py, T, const N: usize> ResolveArgs<'py> for [T; N]
where
    T: IntoPyObject<'py>,
{
    type RawStorage = MaybeUninit<[*mut ffi::PyObject; N]>;
    type Guard = DropManyGuard<T::Output>;
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: PPPyObject,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        DropManyGuard::from_iter(py, storage, base_storage, self)
    }
    #[inline(always)]
    fn len(&self) -> usize {
        N
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        write_iter_to_tuple(tuple, self, index)
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        tuple: Borrowed<'_, 'py, PyTuple>,
        guard: Self::Guard,
        raw_storage: &mut PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) {
        write_raw_storage_to_tuple::<T::Output, _>(tuple, raw_storage, index, N);
        std::mem::forget(guard);
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = N == 0;
    const IS_ONE: bool = N == 1;
    const USE_STACK_FOR_SMALL_LEN: bool = false;
}

impl<'a, 'py, T, const N: usize> ResolveArgs<'py> for &'a [T; N]
where
    &'a T: IntoPyObject<'py>,
{
    type RawStorage = MaybeUninit<[*mut ffi::PyObject; N]>;
    type Guard = DropManyGuard<<&'a T as IntoPyObject<'py>>::Output>;
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: PPPyObject,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        DropManyGuard::from_iter(py, storage, base_storage, self)
    }
    #[inline(always)]
    fn len(&self) -> usize {
        N
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        write_iter_to_tuple(tuple, self, index)
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        tuple: Borrowed<'_, 'py, PyTuple>,
        guard: Self::Guard,
        raw_storage: &mut PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) {
        write_raw_storage_to_tuple::<<&'a T as IntoPyObject<'py>>::Output, _>(
            tuple,
            raw_storage,
            index,
            N,
        );
        std::mem::forget(guard);
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = N == 0;
    const IS_ONE: bool = N == 1;
    const USE_STACK_FOR_SMALL_LEN: bool = false;
}

impl<'a, 'py, T, const N: usize> ResolveArgs<'py> for &'a mut [T; N]
where
    &'a T: IntoPyObject<'py>,
{
    type RawStorage = MaybeUninit<[*mut ffi::PyObject; N]>;
    type Guard = DropManyGuard<<&'a T as IntoPyObject<'py>>::Output>;
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: PPPyObject,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        DropManyGuard::from_iter(py, storage, base_storage, self.iter())
    }
    #[inline(always)]
    fn len(&self) -> usize {
        N
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        write_iter_to_tuple(tuple, self.iter(), index)
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        tuple: Borrowed<'_, 'py, PyTuple>,
        guard: Self::Guard,
        raw_storage: &mut PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) {
        write_raw_storage_to_tuple::<<&'a T as IntoPyObject<'py>>::Output, _>(
            tuple,
            raw_storage,
            index,
            N,
        );
        std::mem::forget(guard);
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = N == 0;
    const IS_ONE: bool = N == 1;
    const USE_STACK_FOR_SMALL_LEN: bool = false;
}

/// A helper trait so that we don't have to repeat the macro for tuples both here and in selection.
pub trait Tuple<'py>: ResolveArgs<'py> {}

macro_rules! impl_resolve_args_for_tuple {
    ( @guard_type $ty:ty, $next:ident, $($rest:ident,)* ) => {
        impl_resolve_args_for_tuple!( @guard_type ConcatArrays<$ty, $next::Output>, $($rest,)* )
    };
    ( @guard_type $ty:ty,  ) => {
        $ty
    };
    ( @count $t:ident ) => { 1 };
    ( ) => {};
    (
        $first:ident, $( $rest:ident, )*
    ) => {
        impl<'py, $first, $( $rest, )*> Tuple<'py> for ( $first, $($rest,)* )
        where
            $first: IntoPyObject<'py>,
            $(
                $rest: IntoPyObject<'py>,
            )*
        {}

        impl<'py, $first, $( $rest, )*> ResolveArgs<'py> for ( $first, $($rest,)* )
        where
            $first: IntoPyObject<'py>,
            $(
                $rest: IntoPyObject<'py>,
            )*
        {
            type RawStorage = MaybeUninit<[*mut ffi::PyObject; 1 $( + impl_resolve_args_for_tuple!(@count $rest) )*]>;
            type Guard = DropOneGuard<'py, impl_resolve_args_for_tuple!( @guard_type $first::Output, $($rest,)* )>;
            #[inline(always)]
            fn init(
                self,
                py: Python<'py>,
                storage: PPPyObject,
                base_storage: *const PPPyObject,
            ) -> PyResult<Self::Guard> {
                #[allow(non_snake_case)]
                let ( $first, $( $rest, )* ) = self;
                Ok(
                    DropOneGuard::from_write(py, storage, base_storage, $first)?
                        $( .write($rest)? )*
                )
            }
            #[inline(always)]
            fn len(&self) -> usize {
                1 $( + impl_resolve_args_for_tuple!(@count $rest) )*
            }
            #[inline(always)]
            fn write_to_tuple(
                self,
                tuple: Borrowed<'_, 'py, PyTuple>,
                index: &mut ffi::Py_ssize_t,
            ) -> PyResult<()> {
                #[allow(non_snake_case)]
                let ( $first, $( $rest, )* ) = self;
                WriteToTuple::new(tuple, index)
                    .write($first)?
                    $( .write($rest)? )*
                    .finish()
            }
            #[inline(always)]
            fn write_initialized_to_tuple(
                tuple: Borrowed<'_, 'py, PyTuple>,
                guard: Self::Guard,
                raw_storage: &mut PPPyObject,
                index: &mut ffi::Py_ssize_t,
            ) {
                let mut p = *raw_storage;
                let mut i = *index;
                unsafe {
                    let value = *p;
                    if !$first::Output::IS_OWNED {
                        ffi::Py_INCREF(value);
                    }
                    ffi::PyTuple_SET_ITEM(tuple.as_ptr(), i, value);
                    p = p.add(1);
                    i += 1;
                }
                *index = i;
                *raw_storage = p;
                std::mem::forget(guard);
            }
            #[inline(always)]
            fn has_known_size(&self) -> bool {
                true
            }
            const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
            const IS_EMPTY: bool = false;
            const IS_ONE: bool = 0 $( + impl_resolve_args_for_tuple!( @count $rest ) )* == 0;
            const USE_STACK_FOR_SMALL_LEN: bool = false;
        }

        impl_resolve_args_for_tuple!( $($rest,)* );
    };
}

// If you are changing the size of the tuple here, make sure to change `build_args()` in
// pyo3-macros-backend/src/pycall.rs too.
impl_resolve_args_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M,);
