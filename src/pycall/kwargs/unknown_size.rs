use std::mem::MaybeUninit;

use crate::conversion::IntoPyObject;
use crate::pycall::storage::DynKnownSizeRawStorage;
use crate::types::{PyDict, PyString, PyTuple};
use crate::{ffi, Borrowed, Bound, PyResult, Python};

use super::helpers::set_kwargs_from_iter;
use super::{ConcatStorages, ExistingNames, PPPyObject, ResolveKwargs};

pub struct UnsizedKwargsStorage<T>(pub(super) T);

impl<'py, T: ResolveKwargs<'py>> ResolveKwargs<'py> for UnsizedKwargsStorage<T> {
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

impl<'py, A, B> ResolveKwargs<'py> for UnsizedKwargsStorage<ConcatStorages<A, B>>
where
    A: ResolveKwargs<'py>,
    B: ResolveKwargs<'py>,
{
    type RawStorage = DynKnownSizeRawStorage;
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
        self.0 .0.write_to_dict(dict)?;
        self.0 .1.write_to_dict(dict)
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        self.0 .0.has_known_size() && self.0 .1.has_known_size()
    }
    const IS_EMPTY: bool = A::IS_EMPTY && B::IS_EMPTY;
}

pub struct AnyIteratorKwargs<I>(pub(super) I);

impl<'py, I, K, V> ResolveKwargs<'py> for AnyIteratorKwargs<I>
where
    I: Iterator<Item = (K, V)>,
    K: IntoPyObject<'py, Target = PyString>,
    V: IntoPyObject<'py>,
{
    type RawStorage = MaybeUninit<()>;
    type Guard = std::convert::Infallible;
    #[inline(always)]
    fn init(
        self,
        _args: PPPyObject,
        _kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        _index: &mut ffi::Py_ssize_t,
        _existing_names: &mut ExistingNames,
    ) -> PyResult<Self::Guard> {
        panic!("Any iterator doesn't have a known size")
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.size_hint().0
    }
    #[inline(always)]
    fn write_to_dict(self, dict: Borrowed<'_, 'py, PyDict>) -> PyResult<usize> {
        let mut len = 0;
        set_kwargs_from_iter(dict, self.0.inspect(|_| len += 1))?;
        Ok(len)
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        false
    }
    const IS_EMPTY: bool = false;
}
