mod array;
mod concat;
mod empty;
mod existing;
mod helpers;
mod pyobjects;
mod selector;
mod unknown_size;
mod vec;
mod vectorcall_arguments_offset;

use crate::types::PyTuple;
use crate::{ffi, Borrowed, PyResult, Python};

pub use empty::EmptyArgsStorage;
pub use selector::{select_traits, ArgsStorageSelector};

pub(super) use array::{ArrayArgsStorage, Tuple};
pub(super) use concat::{ConcatArgsStorages, FundamentalStorage};
pub(super) use existing::{ExistingArgListSlice, ExistingArgListSliceTrait};
pub(super) use unknown_size::{SizedToUnsizedStorage, UnsizedArgsStorage};
pub(super) use vec::VecArgsStorage;
pub(super) use vectorcall_arguments_offset::AppendEmptyArgForVectorcall;

use super::storage::RawStorage;
use super::PPPyObject;

#[derive(Debug)]
pub enum ArgumentsOffsetFlag {
    /// Do not add an offset, as this will lead into more expensive conversion.
    ///
    /// We use this when unpacking an existing slice of args, because then the flag will
    /// mean we need to allocate a new space.
    DoNotOffset,
    /// Like [`ArgumentsOffsetFlag::DoNotOffset`], but the memory is mutable,
    /// and as such, when calling a method, that does not require offsetting - only
    /// write access to `args[0]`, you can provide that.
    DoNotOffsetButCanChangeArgs0,
    /// You can add an offset and mutate `args[0]`.
    Normal,
}

pub(super) type InitParam<'a, 'py, T> =
    <<T as ResolveArgs<'py>>::RawStorage as RawStorage>::InitParam<'a>;

#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be unpacked in a Python call",
    note = "the following types can be unpacked in a Python call: \
        any iterable Python object, any `IntoIterator` that yields \
        types that can be converted into Python objects"
)]
pub trait ResolveArgs<'py>: Sized {
    type RawStorage: RawStorage;
    type Guard;
    fn init(
        self,
        py: Python<'py>,
        storage: InitParam<'_, 'py, Self>,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard>;
    fn len(&self) -> usize;
    fn write_to_tuple(
        self,
        tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()>;
    fn write_initialized_to_tuple(
        tuple: Borrowed<'_, 'py, PyTuple>,
        guard: Self::Guard,
        raw_storage: &mut PPPyObject,
        index: &mut ffi::Py_ssize_t,
    );
    #[inline(always)]
    fn as_pytuple(&self, _py: Python<'py>) -> Option<Borrowed<'_, 'py, PyTuple>> {
        None
    }
    fn has_known_size(&self) -> bool;
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag;
    const IS_EMPTY: bool;
    const IS_ONE: bool;
    /// This is false for array storages, are they are already on stack.
    const USE_STACK_FOR_SMALL_LEN: bool;
}

pub struct ConcatStorages<A, B>(pub(super) A, pub(super) B);

/// This struct is used to create an array whose size is the sum of two smaller arrays, without generic_const_exprs.
#[repr(C)]
pub struct ConcatArrays<A, B>(A, B);
