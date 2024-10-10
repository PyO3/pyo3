use crate::conversion::IntoPyObject;
use crate::pycall::storage::DynKnownSizeRawStorage;
use crate::pycall::trusted_len::TrustedLen;
use crate::types::{PyDict, PyString, PyTuple};
use crate::{ffi, Borrowed, Bound, PyResult, Python};

use super::helpers::{set_kwargs_from_iter, DropManyGuard};
use super::{ConcatStorages, ExistingNames, PPPyObject, ResolveKwargs};

pub struct VecKwargsStorage<T>(pub(super) T);

impl<'py, T: ResolveKwargs<'py>> ResolveKwargs<'py> for VecKwargsStorage<T> {
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

impl<'py, A, B> ResolveKwargs<'py> for VecKwargsStorage<ConcatStorages<A, B>>
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

pub struct TrustedLenIterator<I>(pub(super) I);

impl<'py, I, K, V> ResolveKwargs<'py> for TrustedLenIterator<I>
where
    I: TrustedLen<Item = (K, V)>,
    K: IntoPyObject<'py, Target = PyString>,
    V: IntoPyObject<'py>,
{
    type RawStorage = DynKnownSizeRawStorage;
    type Guard = DropManyGuard<V::Output>;
    #[inline(always)]
    fn init(
        self,
        args: PPPyObject,
        kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
        existing_names: &mut ExistingNames,
    ) -> PyResult<Self::Guard> {
        DropManyGuard::from_iter(args, kwargs_tuple, self.0, index, existing_names)
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.size_hint().0
    }
    #[inline(always)]
    fn write_to_dict(self, dict: Borrowed<'_, 'py, PyDict>) -> PyResult<usize> {
        let len = self.len();
        set_kwargs_from_iter(dict, self.0)?;
        Ok(len)
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const IS_EMPTY: bool = false;
}

pub struct ExactSizeIterator<I> {
    iter: I,
    // We cannot rely on the iterator providing the same `len()` every time (this will lead to unsoundness),
    // so we save it here.
    len: usize,
}

impl<I: std::iter::ExactSizeIterator> ExactSizeIterator<I> {
    #[inline(always)]
    pub(super) fn new(iter: I) -> Self {
        Self {
            len: iter.len(),
            iter,
        }
    }
}

impl<'py, I, K, V> ResolveKwargs<'py> for ExactSizeIterator<I>
where
    I: std::iter::ExactSizeIterator<Item = (K, V)>,
    K: IntoPyObject<'py, Target = PyString>,
    V: IntoPyObject<'py>,
{
    type RawStorage = DynKnownSizeRawStorage;
    type Guard = DropManyGuard<V::Output>;
    #[inline(always)]
    fn init(
        self,
        args: PPPyObject,
        kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
        existing_names: &mut ExistingNames,
    ) -> PyResult<Self::Guard> {
        let mut i = 0;
        let guard = DropManyGuard::from_iter(
            args,
            kwargs_tuple,
            self.iter.inspect(|_| {
                i += 1;
                if i > self.len {
                    panic!("an ExactSizeIterator produced more items than it declared");
                }
            }),
            index,
            existing_names,
        );
        if i != self.len {
            panic!("an ExactSizeIterator produced less items than it declared");
        }
        guard
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }
    #[inline(always)]
    fn write_to_dict(self, dict: Borrowed<'_, 'py, PyDict>) -> PyResult<usize> {
        // No need to check for violations of `ExactSizeIterator`, dict can grow as needed.
        set_kwargs_from_iter(dict, self.iter)?;
        // FIXME: Figure out if it's a problem if an iterator will yield different numbers
        // of items than `len` (probably not, the check will be messed up and may fail
        // succeed wrongly, but they broke the contract of `ExactSizeIterator` so this is fine).
        Ok(self.len)
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const IS_EMPTY: bool = false;
}
