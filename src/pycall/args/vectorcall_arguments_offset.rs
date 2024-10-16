//! `PY_VECTORCALL_ARGUMENTS_OFFSET` needs a first empty arg.

use std::mem::MaybeUninit;

use crate::pycall::PPPyObject;
use crate::types::PyTuple;
use crate::{ffi, Borrowed, PyResult, Python};

use super::{ArgumentsOffsetFlag, ResolveArgs};

pub struct AppendEmptyArgForVectorcall;

impl<'py> ResolveArgs<'py> for AppendEmptyArgForVectorcall {
    type RawStorage = MaybeUninit<*mut ffi::PyObject>;
    type Guard = ();
    #[inline(always)]
    fn init(
        self,
        _py: Python<'py>,
        storage: PPPyObject,
        _base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        unsafe {
            storage.write(std::ptr::null_mut());
        }
        Ok(())
    }
    #[inline(always)]
    fn len(&self) -> usize {
        1
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        _tuple: Borrowed<'_, 'py, PyTuple>,
        _index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        unreachable!("AppendEmptyArgForVectorcall should never be converted into a tuple, it only exists for vectorcall")
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        _tuple: Borrowed<'_, 'py, PyTuple>,
        _guard: Self::Guard,
        _raw_storage: &mut PPPyObject,
        _index: &mut ffi::Py_ssize_t,
    ) {
        unreachable!("AppendEmptyArgForVectorcall should never be converted into a tuple, it only exists for vectorcall")
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = false;
    const IS_ONE: bool = false;
    const USE_STACK_FOR_SMALL_LEN: bool = false;
}
