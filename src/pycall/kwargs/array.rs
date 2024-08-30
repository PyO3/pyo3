use std::mem::MaybeUninit;

use crate::conversion::IntoPyObject;
use crate::types::{PyDict, PyString, PyTuple};
use crate::{ffi, Borrowed, Bound, BoundObject, PyResult, Python};

use super::helpers::{set_kwarg, set_kwargs_from_iter, DropManyGuard, DropOneGuard};
use super::{ConcatArrays, ConcatStorages, ExistingNames, PPPyObject, ResolveKwargs};

pub struct ArrayKwargsStorage<T>(pub(in super::super) T);

impl<'py, T: ResolveKwargs<'py>> ResolveKwargs<'py> for ArrayKwargsStorage<T> {
    type RawStorage = T::RawStorage;
    type Guard = T::Guard;
    #[inline(always)]
    fn init(
        self,
        args: PPPyObject,
        kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
        existing_names: &mut ExistingNames,
    ) -> PyResult<Self::Guard> {
        self.0.init(args, kwargs_tuple, index, existing_names)
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len()
    }
    #[inline(always)]
    fn write_to_dict(self, dict: Borrowed<'_, 'py, PyDict>) -> PyResult<usize> {
        self.0.write_to_dict(dict)
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        self.0.has_known_size()
    }
    const IS_EMPTY: bool = T::IS_EMPTY;
    #[inline(always)]
    fn as_names_pytuple(&self) -> Option<Borrowed<'static, 'py, PyTuple>> {
        self.0.as_names_pytuple()
    }
    #[inline(always)]
    fn can_be_cheaply_converted_to_pydict(&self, py: Python<'py>) -> bool {
        self.0.can_be_cheaply_converted_to_pydict(py)
    }
    #[inline(always)]
    fn init_no_names(self, py: Python<'py>, args: PPPyObject) -> PyResult<Self::Guard> {
        self.0.init_no_names(py, args)
    }
    #[inline(always)]
    fn into_pydict(self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        self.0.into_pydict(py)
    }
}

impl<'py, A, B> ResolveKwargs<'py> for ArrayKwargsStorage<ConcatStorages<A, B>>
where
    A: ResolveKwargs<'py>,
    B: ResolveKwargs<'py>,
{
    type RawStorage = MaybeUninit<ConcatArrays<A::RawStorage, B::RawStorage>>;
    type Guard = (A::Guard, B::Guard);
    #[inline(always)]
    fn init(
        self,
        args: PPPyObject,
        kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
        existing_names: &mut ExistingNames,
    ) -> PyResult<Self::Guard> {
        let g1 = self.0 .0.init(args, kwargs_tuple, index, existing_names)?;
        let g2 = self.0 .1.init(args, kwargs_tuple, index, existing_names)?;
        Ok((g1, g2))
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.0 .0.len() + self.0 .1.len()
    }
    #[inline(always)]
    fn write_to_dict(self, dict: Borrowed<'_, 'py, PyDict>) -> PyResult<usize> {
        let len1 = self.0 .0.write_to_dict(dict)?;
        let len2 = self.0 .1.write_to_dict(dict)?;
        Ok(len1 + len2)
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        self.0 .0.has_known_size() && self.0 .1.has_known_size()
    }
    const IS_EMPTY: bool = A::IS_EMPTY && B::IS_EMPTY;
}

impl<'py, K, V, const N: usize> ResolveKwargs<'py> for [(K, V); N]
where
    K: IntoPyObject<'py, Target = PyString>,
    V: IntoPyObject<'py>,
{
    type RawStorage = MaybeUninit<[*mut ffi::PyObject; N]>;
    type Guard = DropManyGuard<V::Output>;
    #[inline(always)]
    fn init(
        self,
        args: PPPyObject,
        kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
        existing_names: &mut ExistingNames,
    ) -> PyResult<Self::Guard> {
        DropManyGuard::from_iter(args, kwargs_tuple, self, index, existing_names)
    }
    #[inline(always)]
    fn len(&self) -> usize {
        N
    }
    #[inline(always)]
    fn write_to_dict(self, dict: Borrowed<'_, 'py, PyDict>) -> PyResult<usize> {
        set_kwargs_from_iter(dict, self)?;
        Ok(N)
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const IS_EMPTY: bool = N == 0;
}

impl<'a, 'py, K, V, const N: usize> ResolveKwargs<'py> for &'a [(K, V); N]
where
    &'a K: IntoPyObject<'py, Target = PyString>,
    &'a V: IntoPyObject<'py>,
{
    type RawStorage = MaybeUninit<[*mut ffi::PyObject; N]>;
    type Guard = DropManyGuard<<&'a V as IntoPyObject<'py>>::Output>;
    #[inline(always)]
    fn init(
        self,
        args: PPPyObject,
        kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
        existing_names: &mut ExistingNames,
    ) -> PyResult<Self::Guard> {
        DropManyGuard::from_iter(
            args,
            kwargs_tuple,
            self.iter().map(|(k, v)| (k, v)),
            index,
            existing_names,
        )
    }
    #[inline(always)]
    fn len(&self) -> usize {
        N
    }
    #[inline(always)]
    fn write_to_dict(self, dict: Borrowed<'_, 'py, PyDict>) -> PyResult<usize> {
        set_kwargs_from_iter(dict, self.iter().map(|(k, v)| (k, v)))?;
        Ok(N)
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const IS_EMPTY: bool = N == 0;
}

impl<'a, 'py, K, V, const N: usize> ResolveKwargs<'py> for &'a mut [(K, V); N]
where
    &'a K: IntoPyObject<'py, Target = PyString>,
    &'a V: IntoPyObject<'py>,
{
    type RawStorage = MaybeUninit<[*mut ffi::PyObject; N]>;
    type Guard = DropManyGuard<<&'a V as IntoPyObject<'py>>::Output>;
    #[inline(always)]
    fn init(
        self,
        args: PPPyObject,
        kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
        existing_names: &mut ExistingNames,
    ) -> PyResult<Self::Guard> {
        DropManyGuard::from_iter(
            args,
            kwargs_tuple,
            self.iter().map(|(k, v)| (k, v)),
            index,
            existing_names,
        )
    }
    #[inline(always)]
    fn len(&self) -> usize {
        N
    }
    #[inline(always)]
    fn write_to_dict(self, dict: Borrowed<'_, 'py, PyDict>) -> PyResult<usize> {
        set_kwargs_from_iter(dict, self.iter().map(|(k, v)| (k, v)))?;
        Ok(N)
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const IS_EMPTY: bool = N == 0;
}

/// A helper trait so that we don't have to repeat the macro for tuples both here and in selection.
pub trait Tuple<'py>: ResolveKwargs<'py> {}

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
        $first_a:ident $first_b:ident, $( $rest_a:ident $rest_b:ident, )*
    ) => {
        impl<'py, $first_a, $first_b, $( $rest_a, $rest_b, )*> Tuple<'py> for ( ($first_a, $first_b), $( ($rest_a, $rest_b), )* )
        where
            $first_a: IntoPyObject<'py, Target = PyString>,
            $first_b: IntoPyObject<'py>,
            $(
                $rest_a: IntoPyObject<'py, Target = PyString>,
                $rest_b: IntoPyObject<'py>,
            )*
        {}

        impl<'py, $first_a, $first_b, $( $rest_a, $rest_b, )*> ResolveKwargs<'py> for ( ($first_a, $first_b), $( ($rest_a, $rest_b), )* )
        where
            $first_a: IntoPyObject<'py, Target = PyString>,
            $first_b: IntoPyObject<'py>,
            $(
                $rest_a: IntoPyObject<'py, Target = PyString>,
                $rest_b: IntoPyObject<'py>,
            )*
        {
            type RawStorage = MaybeUninit<[*mut ffi::PyObject; 1 $( + impl_resolve_args_for_tuple!( @count $rest_b ) )*]>;
            type Guard = DropOneGuard<'py, impl_resolve_args_for_tuple!( @guard_type $first_b::Output, $($rest_b,)* )>;
            #[inline(always)]
            fn init(
                self,
                args: PPPyObject,
                kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
                index: &mut ffi::Py_ssize_t,
                existing_names: &mut ExistingNames,
            ) -> PyResult<Self::Guard> {
                #[allow(non_snake_case)]
                let ( ($first_a, $first_b), $( ($rest_a, $rest_b), )* ) = self;
                Ok(
                    DropOneGuard::from_write(args, kwargs_tuple, index, existing_names, $first_a, $first_b)?
                        $( .write(kwargs_tuple, index, existing_names, $rest_a, $rest_b)? )*
                )
            }
            #[inline(always)]
            fn len(&self) -> usize {
                1 $( + impl_resolve_args_for_tuple!( @count $rest_a ) )*
            }
            #[inline(always)]
            fn write_to_dict(self, dict: Borrowed<'_, 'py, PyDict>) -> PyResult<usize> {
                let py = dict.py();
                #[allow(non_snake_case)]
                let ( ($first_a, $first_b), $( ($rest_a, $rest_b), )* ) = self;
                set_kwarg(
                    dict,
                    $first_a.into_pyobject(py).map_err(Into::into)?.as_borrowed(),
                    $first_b.into_pyobject(py).map_err(Into::into)?.into_any().as_borrowed(),
                )?;
                $(
                    set_kwarg(
                        dict,
                        $rest_a.into_pyobject(py).map_err(Into::into)?.as_borrowed(),
                        $rest_b.into_pyobject(py).map_err(Into::into)?.into_any().as_borrowed(),
                    )?;
                )*
                Ok(1 $( + impl_resolve_args_for_tuple!( @count $rest_b ) )*)
            }
            #[inline(always)]
            fn has_known_size(&self) -> bool {
                true
            }
            const IS_EMPTY: bool = false;
        }

        impl_resolve_args_for_tuple!( $($rest_a $rest_b,)* );
    };
}

// If you are changing the size of the tuple here, make sure to change `build_unknown_non_unpacked_kwargs()` in
// pyo3-macros-backend/src/pycall.rs too.
impl_resolve_args_for_tuple!(A1 A2, B1 B2, C1 C2, D1 D2, E1 E2, F1 F2, G1 G2, H1 H2, I1 I2, J1 J2, K1 K2, L1 L2, M1 M2,);
