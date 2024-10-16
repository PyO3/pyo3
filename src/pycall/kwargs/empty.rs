use std::mem::MaybeUninit;

use crate::types::{PyDict, PyTuple};
use crate::{ffi, Borrowed, PyResult};

use super::{ExistingNames, PPPyObject, ResolveKwargs};

pub struct EmptyKwargsStorage;

impl<'py> ResolveKwargs<'py> for EmptyKwargsStorage {
    type RawStorage = MaybeUninit<()>;
    type Guard = ();
    #[inline(always)]
    fn init(
        self,
        _args: PPPyObject,
        _kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        _index: &mut ffi::Py_ssize_t,
        _existing_names: &mut ExistingNames,
    ) -> PyResult<Self::Guard> {
        unreachable!(
            "`EmptyKwargsStorage` should never be converted into a dict or tuple, \
            rather it should pass NULL for kwargs"
        )
    }
    #[inline(always)]
    fn len(&self) -> usize {
        0
    }
    #[inline(always)]
    fn write_to_dict(self, _dict: Borrowed<'_, 'py, PyDict>) -> PyResult<usize> {
        unreachable!(
            "`EmptyKwargsStorage` should never be converted into a dict or tuple, \
            rather it should pass NULL for kwargs"
        )
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const IS_EMPTY: bool = true;
}
