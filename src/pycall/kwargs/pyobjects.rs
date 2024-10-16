use std::ffi::c_int;
use std::mem::MaybeUninit;

use crate::err::error_on_minusone;
use crate::exceptions::{PyRuntimeError, PyTypeError};
use crate::pycall::as_pyobject::AsPyObject;
use crate::pycall::storage::DynKnownSizeRawStorage;
use crate::types::{PyAnyMethods, PyDict, PyDictMethods, PyString, PyTuple};
use crate::{ffi, Borrowed, Bound, BoundObject, PyResult, Python};

use super::{ExistingNames, PPPyObject, ResolveKwargs};

pub struct PyDictKwargsStorage<T> {
    value: T,
    is_not_dict_subclass: bool,
    len: usize,
}

impl<'py, T: AsPyObject<'py, PyObject = PyDict>> PyDictKwargsStorage<T> {
    #[inline(always)]
    pub fn new(value: T) -> Self {
        let value_borrowed = value.as_borrowed(unsafe { Python::assume_gil_acquired() });
        let is_not_dict_subclass = value_borrowed.is_exact_instance_of::<PyDict>();
        // Do not call `PyDictMethods::len()`, as it will be incorrect for dict subclasses.
        let len = PyAnyMethods::len(&**value_borrowed).unwrap_or(0);
        Self {
            value,
            is_not_dict_subclass,
            len,
        }
    }
}

const DICT_MERGE_ERR_ON_DUPLICATE: c_int = 2;

#[inline(always)]
fn copy_dict_if_needed<'py, T: AsPyObject<'py>>(
    py: Python<'py>,
    value: T,
) -> PyResult<Bound<'py, PyDict>> {
    if T::IS_OWNED && value.as_borrowed(py).into_any().get_refcnt() == 1 {
        Ok(unsafe { value.into_bound(py).into_any().downcast_into_unchecked() })
    } else {
        unsafe {
            value
                .as_borrowed(py)
                .into_any()
                .downcast_unchecked::<PyDict>()
        }
        .copy()
    }
}

impl<'py, T: AsPyObject<'py, PyObject = PyDict>> ResolveKwargs<'py> for PyDictKwargsStorage<T> {
    type RawStorage = DynKnownSizeRawStorage;
    // Need to keep the dict around because we borrow the values from it.
    type Guard = T;
    #[inline(always)]
    fn init(
        self,
        args: PPPyObject,
        kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
        existing_names: &mut ExistingNames,
    ) -> PyResult<Self::Guard> {
        debug_assert!(
            self.is_not_dict_subclass,
            "dict subclasses have no known size",
        );
        let py = kwargs_tuple.py();
        let dict = self.value.as_borrowed(py);
        unsafe {
            let mut pos = 0;
            let mut key = std::ptr::null_mut();
            let mut value = std::ptr::null_mut();
            let mut len = self.len as isize;
            let di_used = len;
            while ffi::PyDict_Next(dict.as_ptr(), &mut pos, &mut key, &mut value) != 0 {
                let ma_used = dict.len() as isize;

                if di_used != ma_used || len == -1 {
                    return Err(PyRuntimeError::new_err(
                        intern!(py, "dictionary changed during iteration")
                            .clone()
                            .unbind(),
                    ));
                };
                len -= 1;

                let key = Borrowed::from_ptr_unchecked(py, key)
                    .downcast::<PyString>()
                    .map_err(|err| {
                        let new_err = PyTypeError::new_err(
                            intern!(py, "keywords must be strings").clone().unbind(),
                        );
                        new_err.set_cause(py, Some(err.into()));
                        new_err
                    })?
                    .to_owned();
                let value = Borrowed::from_ptr_unchecked(py, value);
                existing_names.insert(key, value, args, kwargs_tuple, index)?;
            }
        }
        Ok(self.value)
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }
    #[inline(always)]
    fn write_to_dict(self, dict: Borrowed<'_, 'py, PyDict>) -> PyResult<usize> {
        unsafe {
            error_on_minusone(
                dict.py(),
                ffi::PyDict_Merge(
                    dict.as_ptr(),
                    self.value.as_borrowed(dict.py()).as_ptr(),
                    DICT_MERGE_ERR_ON_DUPLICATE,
                ),
            )?;
        }
        Ok(self.len)
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        self.is_not_dict_subclass
    }
    #[inline(always)]
    fn can_be_cheaply_converted_to_pydict(&self, _py: Python<'py>) -> bool {
        T::IS_OWNED && self.is_not_dict_subclass
    }
    #[inline(always)]
    fn into_pydict(self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        copy_dict_if_needed(py, self.value)
    }
    const IS_EMPTY: bool = false;
}

pub struct AnyPyMapping<T>(pub(super) T);

impl<'py, T: AsPyObject<'py>> ResolveKwargs<'py> for AnyPyMapping<T> {
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
        panic!("Python classes have no known size")
    }
    #[inline(always)]
    fn len(&self) -> usize {
        0
    }
    #[inline(always)]
    fn write_to_dict(self, dict: Borrowed<'_, 'py, PyDict>) -> PyResult<usize> {
        unsafe {
            let value = self.0.as_borrowed(dict.py());
            let len = value.into_any().len()?;
            error_on_minusone(
                dict.py(),
                ffi::PyDict_Merge(dict.as_ptr(), value.as_ptr(), DICT_MERGE_ERR_ON_DUPLICATE),
            )?;
            Ok(len)
        }
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        false
    }
    #[inline(always)]
    fn can_be_cheaply_converted_to_pydict(&self, py: Python<'py>) -> bool {
        T::IS_OWNED
            && self
                .0
                .as_borrowed(py)
                .into_any()
                .is_exact_instance_of::<PyDict>()
    }
    #[inline(always)]
    fn into_pydict(self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        copy_dict_if_needed(py, self.0)
    }
    const IS_EMPTY: bool = false;
}
