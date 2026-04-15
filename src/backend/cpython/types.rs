use crate::err::{self, PyResult};
use crate::ffi::Py_ssize_t;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::{Borrowed, Bound, BoundObject};
use crate::sync::PyOnceLock;
use crate::types::{PyAny, PyFrozenSet, PyTuple, PyType, PyTypeMethods};
use crate::{ffi, IntoPyObject, IntoPyObjectExt, Py, Python};

#[inline]
pub(crate) fn dict_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyDict_Type
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub(crate) fn dict_keys_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyDictKeys_Type
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub(crate) fn dict_values_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyDictValues_Type
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub(crate) fn dict_items_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyDictItems_Type
}

#[inline]
pub(crate) fn string_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(_py, "builtins", "str").unwrap().as_type_ptr()
}

#[inline]
pub(crate) fn list_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyList_Type
}

#[inline]
pub(crate) fn tuple_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(_py, "builtins", "tuple").unwrap().as_type_ptr()
}

#[inline]
pub(crate) fn set_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PySet_Type
}

#[inline]
pub(crate) fn frozenset_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyFrozenSet_Type
}

#[inline]
pub(crate) fn dict_len(dict: *mut ffi::PyObject) -> Py_ssize_t {
    unsafe {
        ffi::PyDict_Size(dict)
    }
}

pub(crate) struct PyFrozenSetBuilderState<'py> {
    py_frozen_set: Bound<'py, PyFrozenSet>,
}

pub(crate) fn new_frozenset_builder(py: Python<'_>) -> PyResult<PyFrozenSetBuilderState<'_>> {
    Ok(PyFrozenSetBuilderState {
        py_frozen_set: unsafe {
            ffi::PyFrozenSet_New(std::ptr::null_mut())
                .assume_owned_or_err(py)?
                .cast_into_unchecked()
        },
    })
}

pub(crate) fn frozenset_builder_add<'py, K>(
    builder: &mut PyFrozenSetBuilderState<'py>,
    key: K,
) -> PyResult<()>
where
    K: IntoPyObject<'py>,
{
    fn inner(frozenset: &Bound<'_, PyFrozenSet>, key: Borrowed<'_, '_, PyAny>) -> PyResult<()> {
        err::error_on_minusone(frozenset.py(), unsafe {
            ffi::PySet_Add(frozenset.as_ptr(), key.as_ptr())
        })
    }

    inner(
        &builder.py_frozen_set,
        key.into_pyobject_or_pyerr(builder.py_frozen_set.py())?
            .into_any()
            .as_borrowed(),
    )
}

pub(crate) fn frozenset_builder_finalize(
    builder: PyFrozenSetBuilderState<'_>,
) -> Bound<'_, PyFrozenSet> {
    builder.py_frozen_set
}

#[track_caller]
pub(crate) fn try_new_tuple_from_iter<'py>(
    py: Python<'py>,
    mut elements: impl ExactSizeIterator<Item = PyResult<Bound<'py, PyAny>>>,
) -> PyResult<Bound<'py, PyTuple>> {
    unsafe {
        let len: Py_ssize_t = elements
            .len()
            .try_into()
            .expect("out of range integral type conversion attempted on `elements.len()`");

        let ptr = ffi::PyTuple_New(len);
        let tup = ptr.assume_owned(py).cast_into_unchecked();
        let mut counter: Py_ssize_t = 0;

        for obj in (&mut elements).take(len as usize) {
            #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
            ffi::PyTuple_SET_ITEM(ptr, counter, obj?.into_ptr());
            #[cfg(any(Py_LIMITED_API, PyPy, GraalPy))]
            ffi::PyTuple_SetItem(ptr, counter, obj?.into_ptr());
            counter += 1;
        }

        assert!(elements.next().is_none(), "Attempted to create PyTuple but `elements` was larger than reported by its `ExactSizeIterator` implementation.");
        assert_eq!(len, counter, "Attempted to create PyTuple but `elements` was smaller than reported by its `ExactSizeIterator` implementation.");

        Ok(tup)
    }
}

pub(crate) fn array_into_tuple<'py, const N: usize>(
    py: Python<'py>,
    array: [Bound<'py, PyAny>; N],
) -> Bound<'py, PyTuple> {
    unsafe {
        let ptr = ffi::PyTuple_New(N.try_into().expect("0 < N <= 12"));
        let tup = ptr.assume_owned(py).cast_into_unchecked();
        for (index, obj) in array.into_iter().enumerate() {
            #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
            ffi::PyTuple_SET_ITEM(ptr, index as ffi::Py_ssize_t, obj.into_ptr());
            #[cfg(any(Py_LIMITED_API, PyPy, GraalPy))]
            ffi::PyTuple_SetItem(ptr, index as ffi::Py_ssize_t, obj.into_ptr());
        }
        tup
    }
}

#[inline]
pub(crate) fn tuple_len(tuple: *mut ffi::PyObject) -> usize {
    unsafe {
        #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
        let size = ffi::PyTuple_GET_SIZE(tuple);
        #[cfg(any(Py_LIMITED_API, PyPy, GraalPy))]
        let size = ffi::PyTuple_Size(tuple);
        size as usize
    }
}

pub(crate) unsafe fn borrowed_tuple_item_for_extract<'a, 'py>(
    tuple: Borrowed<'a, 'py, PyTuple>,
    index: usize,
) -> PyResult<Borrowed<'a, 'py, PyAny>> {
    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    unsafe {
        Ok(ffi::PyTuple_GET_ITEM(tuple.as_ptr(), index as Py_ssize_t).assume_borrowed_unchecked(tuple.py()))
    }

    #[cfg(any(Py_LIMITED_API, PyPy, GraalPy))]
    unsafe {
        ffi::PyTuple_GetItem(tuple.as_ptr(), index as Py_ssize_t).assume_borrowed_or_err(tuple.py())
    }
}

pub(crate) unsafe fn borrowed_tuple_item_unchecked<'a, 'py>(
    tuple: Borrowed<'a, 'py, PyTuple>,
    index: usize,
) -> Borrowed<'a, 'py, PyAny> {
    unsafe {
        ffi::PyTuple_GET_ITEM(tuple.as_ptr(), index as Py_ssize_t).assume_borrowed_unchecked(tuple.py())
    }
}
