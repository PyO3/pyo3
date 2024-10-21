mod array;
mod concat;
mod empty;
mod helpers;
mod known;
mod pyobjects;
mod selector;
mod unknown_size;
mod vec;

pub use empty::EmptyKwargsStorage;
pub use selector::{select_traits, KwargsStorageSelector};

pub(super) use array::{ArrayKwargsStorage, Tuple};
pub(super) use concat::{ConcatKwargsStorages, FundamentalStorage};
pub(super) use known::{
    KnownKwargsNames, KnownKwargsStorage, TypeLevelPyObjectListCons, TypeLevelPyObjectListNil,
    TypeLevelPyObjectListTrait,
};
pub(super) use unknown_size::UnsizedKwargsStorage;
pub(super) use vec::VecKwargsStorage;

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::mem::ManuallyDrop;

use crate::exceptions::PyTypeError;
use crate::types::{PyDict, PyString, PyTuple};
use crate::{ffi, Bound, PyAny};
use crate::{Borrowed, PyResult, Python};

use super::storage::RawStorage;
use super::PPPyObject;

struct ExistingName {
    hash: ffi::Py_hash_t,
    ptr: *mut ffi::PyObject,
}

impl ExistingName {
    #[inline]
    fn new(name: Borrowed<'_, '_, PyString>) -> Self {
        let hash = unsafe { ffi::PyObject_Hash(name.as_ptr()) };
        Self {
            hash,
            ptr: name.as_ptr(),
        }
    }
}

impl Hash for ExistingName {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_isize(self.hash);
    }
}

impl PartialEq for ExistingName {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        // PyUnicode comparison functions don't consider str subclasses.
        unsafe { ffi::PyObject_RichCompareBool(self.ptr, other.ptr, ffi::Py_EQ) == 1 }
    }
}

impl Eq for ExistingName {}

#[derive(Default)]
struct IdentityHasher(isize);

impl Hasher for IdentityHasher {
    #[inline]
    fn write_isize(&mut self, i: isize) {
        self.0 = i;
    }

    fn finish(&self) -> u64 {
        self.0 as u64
    }

    fn write(&mut self, _bytes: &[u8]) {
        panic!("should only hash isize in IdentityHasher")
    }
}

pub struct ExistingNames(HashMap<ExistingName, (), BuildHasherDefault<IdentityHasher>>);

impl ExistingNames {
    #[inline]
    pub(super) fn new(capacity: usize) -> Self {
        Self(HashMap::with_capacity_and_hasher(
            capacity,
            BuildHasherDefault::default(),
        ))
    }

    #[inline(always)]
    fn check_borrowed(
        &mut self,
        new_name: Borrowed<'_, '_, PyString>,
        kwargs_tuple: Borrowed<'_, '_, PyTuple>,
        index: ffi::Py_ssize_t,
    ) -> PyResult<()> {
        match self.0.entry(ExistingName::new(new_name.as_borrowed())) {
            std::collections::hash_map::Entry::Occupied(_) => {
                return Err(PyTypeError::new_err(
                    intern!(
                        kwargs_tuple.py(),
                        "got multiple values for keyword argument"
                    )
                    .clone()
                    .unbind(),
                ));
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(());
                unsafe {
                    ffi::PyTuple_SET_ITEM(kwargs_tuple.as_ptr(), index, new_name.as_ptr());
                }
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn insert(
        &mut self,
        new_name: Bound<'_, PyString>,
        new_value: Borrowed<'_, '_, PyAny>,
        args: PPPyObject,
        kwargs_tuple: Borrowed<'_, '_, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        self.check_borrowed(
            ManuallyDrop::new(new_name).as_borrowed(),
            kwargs_tuple,
            *index,
        )?;
        unsafe {
            args.offset(*index).write(new_value.as_ptr());
        }
        *index += 1;
        Ok(())
    }
}

#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be unpacked in a Python call",
    note = "the following types can be unpacked in a Python call: \
        any mapping Python object, any `IntoIterator` that yields \
        (IntoPyObject<Target = PyString>, IntoPyObject)"
)]
pub trait ResolveKwargs<'py>: Sized {
    type RawStorage: for<'a> RawStorage<InitParam<'a> = PPPyObject> + 'static;
    type Guard;
    fn init(
        self,
        args: PPPyObject,
        kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
        existing_names: &mut ExistingNames,
    ) -> PyResult<Self::Guard>;
    #[inline(always)]
    fn init_no_names(self, _py: Python<'py>, _args: PPPyObject) -> PyResult<Self::Guard> {
        unreachable!("`ResolveKwargs::init_no_names()` should only be called for known names")
    }
    fn len(&self) -> usize;
    /// This returns the number of kwargs written.
    fn write_to_dict(self, dict: Borrowed<'_, 'py, PyDict>) -> PyResult<usize>;
    #[inline(always)]
    fn can_be_cheaply_converted_to_pydict(&self, _py: Python<'py>) -> bool {
        false
    }
    #[inline(always)]
    fn into_pydict(self, _py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        panic!("cannot be cheaply converted into PyDict")
    }
    #[inline(always)]
    fn as_names_pytuple(&self) -> Option<Borrowed<'static, 'py, PyTuple>> {
        None
    }
    fn has_known_size(&self) -> bool;
    const IS_EMPTY: bool;
}

pub struct ConcatStorages<A, B>(A, B);

/// This struct is used to create an array whose size is the sum of two smaller arrays, without generic_const_exprs.
#[repr(C)]
pub struct ConcatArrays<A, B>(A, B);
