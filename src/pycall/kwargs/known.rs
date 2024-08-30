use std::marker::PhantomData;
use std::mem::MaybeUninit;

use crate::ffi_ptr_ext::FfiPtrExt;
use crate::prelude::IntoPyObject;
use crate::pycall::storage::RawStorage;
use crate::pycall::PPPyObject;
use crate::types::{PyAnyMethods, PyDict, PyString, PyTuple};
use crate::{ffi, Borrowed, BoundObject, Py, PyResult, Python};

use super::{ConcatArrays, ExistingNames, ResolveKwargs};

#[macro_export]
#[doc(hidden)]
macro_rules! known_kwargs {
    ( $( $names:literal )* ) => {{
        static KNOWN_NAMES: $crate::sync::GILOnceCell<$crate::pycall::KnownKwargsNames> =
            $crate::sync::GILOnceCell::new();
        KNOWN_NAMES.get_or_init(
            unsafe { $crate::Python::assume_gil_acquired() },
            || $crate::pycall::KnownKwargsNames::new(&[ $($names),* ]),
        )
    }};
}

pub struct KnownKwargsNames(pub(in super::super) Py<PyTuple>);

impl KnownKwargsNames {
    #[inline]
    pub fn new(names: &[&'static str]) -> Self {
        let tuple = unsafe {
            let py = Python::assume_gil_acquired();
            let tuple = ffi::PyTuple_New(names.len() as ffi::Py_ssize_t)
                .assume_owned_or_err(py)
                .expect("failed to initialize tuple for kwargs")
                .downcast_into_unchecked::<PyTuple>();
            for (i, name) in names.into_iter().enumerate() {
                let name = PyString::new(py, name);
                ffi::PyTuple_SET_ITEM(tuple.as_ptr(), i as ffi::Py_ssize_t, name.into_ptr());
            }
            tuple
        };
        Self(tuple.unbind())
    }
}

// The list is reversed!
pub trait TypeLevelPyObjectListTrait<'py> {
    type RawStorage: for<'a> RawStorage<InitParam<'a> = PPPyObject> + 'static;
    fn add_args(
        self,
        py: Python<'py>,
        args: PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()>;
    fn add_to_dict(
        self,
        dict: Borrowed<'_, 'py, PyDict>,
        names: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()>;
    fn drop_arg(arg: &mut PPPyObject);
    const LEN: usize;
}

#[repr(C)]
pub struct TypeLevelPyObjectListCons<T, Next>(pub(in super::super) T, pub(in super::super) Next);

impl<'py, T, Prev> TypeLevelPyObjectListTrait<'py> for TypeLevelPyObjectListCons<T, Prev>
where
    T: IntoPyObject<'py>,
    Prev: TypeLevelPyObjectListTrait<'py>,
{
    type RawStorage = MaybeUninit<ConcatArrays<*mut ffi::PyObject, Prev::RawStorage>>;
    #[inline(always)]
    fn add_args(
        self,
        py: Python<'py>,
        args: PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        unsafe {
            self.1.add_args(py, args, index)?;
            args.offset(*index)
                .write(self.0.into_pyobject(py).map_err(Into::into)?.into_ptr_raw());
            *index += 1;
        }
        Ok(())
    }
    fn add_to_dict(
        self,
        dict: Borrowed<'_, 'py, PyDict>,
        names: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        self.1.add_to_dict(dict, names, index)?;
        unsafe {
            let name = ffi::PyTuple_GET_ITEM(names.as_ptr(), *index);
            let value = self.0.into_pyobject(dict.py()).map_err(Into::into)?;
            ffi::PyDict_SetItem(dict.as_ptr(), name, value.as_borrowed().as_ptr());
        }
        *index += 1;
        Ok(())
    }
    #[inline(always)]
    fn drop_arg(arg: &mut PPPyObject) {
        unsafe {
            Prev::drop_arg(arg);
            (*arg).cast::<T::Output>().drop_in_place();
            *arg = arg.add(1);
        }
    }
    const LEN: usize = 1 + Prev::LEN;
}

#[repr(C)]
pub struct TypeLevelPyObjectListNil;

impl<'py> TypeLevelPyObjectListTrait<'py> for TypeLevelPyObjectListNil {
    type RawStorage = MaybeUninit<()>;
    #[inline(always)]
    fn add_args(
        self,
        _py: Python<'py>,
        _args: PPPyObject,
        _index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        Ok(())
    }
    #[inline(always)]
    fn add_to_dict(
        self,
        _dict: Borrowed<'_, 'py, PyDict>,
        _names: Borrowed<'_, 'py, PyTuple>,
        _index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        Ok(())
    }
    #[inline(always)]
    fn drop_arg(_arg: &mut PPPyObject) {}
    const LEN: usize = 0;
}

pub struct KnownKwargsStorage<'py, Values> {
    pub(in super::super) names: Borrowed<'static, 'py, PyTuple>,
    pub(in super::super) values: Values,
}

pub struct KnownKwargsGuard<'py, Values: TypeLevelPyObjectListTrait<'py>> {
    args: PPPyObject,
    _marker: PhantomData<(Values, &'py ())>,
}

impl<'py, Values: TypeLevelPyObjectListTrait<'py>> Drop for KnownKwargsGuard<'py, Values> {
    #[inline(always)]
    fn drop(&mut self) {
        Values::drop_arg(&mut { self.args });
    }
}

impl<'py, Values: TypeLevelPyObjectListTrait<'py>> ResolveKwargs<'py>
    for KnownKwargsStorage<'py, Values>
{
    type RawStorage = Values::RawStorage;
    type Guard = KnownKwargsGuard<'py, Values>;
    #[inline(always)]
    fn init(
        self,
        args: PPPyObject,
        kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
        existing_names: &mut ExistingNames,
    ) -> PyResult<Self::Guard> {
        let ptr = unsafe { args.offset(*index) };
        for i in 0..Values::LEN as ffi::Py_ssize_t {
            unsafe {
                let name = ffi::PyTuple_GET_ITEM(self.names.as_ptr(), i)
                    .assume_borrowed_unchecked(kwargs_tuple.py())
                    .downcast_unchecked();
                existing_names.check_borrowed(name, kwargs_tuple, *index + i)?;
            }
        }
        self.values.add_args(kwargs_tuple.py(), ptr, index)?;
        Ok(KnownKwargsGuard {
            args: ptr,
            _marker: PhantomData,
        })
    }
    #[inline(always)]
    fn len(&self) -> usize {
        Values::LEN
    }
    #[inline(always)]
    fn write_to_dict(self, dict: Borrowed<'_, 'py, PyDict>) -> PyResult<usize> {
        let mut len = 0;
        self.values.add_to_dict(dict, self.names, &mut len)?;
        Ok(len as usize)
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const IS_EMPTY: bool = false;
    #[inline(always)]
    fn as_names_pytuple(&self) -> Option<Borrowed<'static, 'py, PyTuple>> {
        Some(self.names)
    }
    #[inline(always)]
    fn init_no_names(self, py: Python<'py>, args: PPPyObject) -> PyResult<Self::Guard> {
        self.values.add_args(py, args, &mut 0)?;
        Ok(KnownKwargsGuard {
            args,
            _marker: PhantomData,
        })
    }
}
