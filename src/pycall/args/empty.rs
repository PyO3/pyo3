use std::mem::MaybeUninit;

use crate::pycall::PPPyObject;
use crate::types::PyTuple;
use crate::{ffi, Borrowed, PyResult, Python};

use super::{ArgumentsOffsetFlag, ResolveArgs};

pub struct EmptyArgsStorage;

impl<'py> ResolveArgs<'py> for EmptyArgsStorage {
    type RawStorage = MaybeUninit<()>;
    type Guard = ();
    #[inline(always)]
    fn init(
        self,
        _py: Python<'py>,
        _storage: PPPyObject,
        _base_storage: *const PPPyObject,
    ) -> PyResult<()> {
        Ok(())
    }
    #[inline(always)]
    fn len(&self) -> usize {
        0
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        _tuple: Borrowed<'_, 'py, PyTuple>,
        _index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        Ok(())
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        _tuple: Borrowed<'_, 'py, PyTuple>,
        _guard: Self::Guard,
        _raw_storage: &mut PPPyObject,
        _index: &mut ffi::Py_ssize_t,
    ) {
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = true;
    const IS_ONE: bool = false;
    const USE_STACK_FOR_SMALL_LEN: bool = false;
}
