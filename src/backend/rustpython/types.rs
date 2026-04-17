use crate::err::{self, PyResult};
use crate::ffi::Py_ssize_t;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::{Borrowed, Bound, BoundObject};
use crate::sync::PyOnceLock;
use crate::types::any::PyAnyMethods;
use crate::types::{PyAny, PyDateTime, PyDict, PyFrozenSet, PySet, PyTime, PyTuple, PyType, PyTypeMethods, PyTzInfo};
use crate::{ffi, IntoPyObject, IntoPyObjectExt, Py, Python};

#[inline]
pub(crate) fn dict_type_object(py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(py, "builtins", "dict").unwrap().as_type_ptr()
}

#[cfg(not(any(PyPy, GraalPy)))]
fn dict_view_type_object(py: Python<'_>, method: &str, cache: &PyOnceLock<Py<PyType>>) -> *mut ffi::PyTypeObject {
    cache
        .get_or_init(py, || {
            let dict = PyDict::new(py);
            let view = dict.call_method0(method).unwrap();
            view.get_type().unbind()
        })
        .bind(py)
        .as_type_ptr()
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub(crate) fn dict_keys_type_object(py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    dict_view_type_object(py, "keys", &TYPE)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub(crate) fn dict_values_type_object(py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    dict_view_type_object(py, "values", &TYPE)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub(crate) fn dict_items_type_object(py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    dict_view_type_object(py, "items", &TYPE)
}

#[inline]
pub(crate) fn string_type_object(py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(py, "builtins", "str").unwrap().as_type_ptr()
}

#[inline]
pub(crate) fn bytes_type_object(py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(py, "builtins", "bytes").unwrap().as_type_ptr()
}

#[inline]
pub(crate) fn complex_type_object(py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(py, "builtins", "complex")
        .unwrap()
        .as_type_ptr()
}

#[inline]
pub(crate) fn slice_type_object(py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(py, "builtins", "slice").unwrap().as_type_ptr()
}

#[inline]
pub(crate) fn mappingproxy_type_object(py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(py, "_types", "MappingProxyType")
        .unwrap()
        .as_type_ptr()
}

#[inline]
pub(crate) fn list_type_object(py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(py, "builtins", "list").unwrap().as_type_ptr()
}

#[inline]
pub(crate) fn tuple_type_object(py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(py, "builtins", "tuple").unwrap().as_type_ptr()
}

#[inline]
pub(crate) fn set_type_object(py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(py, "builtins", "set").unwrap().as_type_ptr()
}

#[inline]
pub(crate) fn frozenset_type_object(py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(py, "builtins", "frozenset")
        .unwrap()
        .as_type_ptr()
}

#[inline]
pub(crate) fn dict_len(dict: *mut ffi::PyObject) -> Py_ssize_t {
    unsafe { ffi::PyDict_Size(dict) }
}

pub(crate) struct PyFrozenSetBuilderState<'py> {
    py_set: Bound<'py, PySet>,
}

pub(crate) fn new_frozenset_builder(py: Python<'_>) -> PyResult<PyFrozenSetBuilderState<'_>> {
    Ok(PyFrozenSetBuilderState {
        py_set: unsafe {
            ffi::PySet_New(std::ptr::null_mut())
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
    fn inner(set: &Bound<'_, PySet>, key: Borrowed<'_, '_, PyAny>) -> PyResult<()> {
        err::error_on_minusone(set.py(), unsafe { ffi::PySet_Add(set.as_ptr(), key.as_ptr()) })
    }

    inner(
        &builder.py_set,
        key.into_pyobject_or_pyerr(builder.py_set.py())?
            .into_any()
            .as_borrowed(),
    )
}

pub(crate) fn frozenset_builder_finalize(
    builder: PyFrozenSetBuilderState<'_>,
) -> Bound<'_, PyFrozenSet> {
    unsafe {
        ffi::PyFrozenSet_New(builder.py_set.as_ptr())
            .assume_owned_or_err(builder.py_set.py())
            .expect("PyFrozenSet_New from PySet should succeed")
            .cast_into_unchecked()
    }
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
        let list = ffi::PyList_New(len.try_into().expect("tuple too large"));
        let list = list.assume_owned(py).cast_into_unchecked::<crate::types::PyList>();
        let mut counter: Py_ssize_t = 0;
        for (index, obj) in (&mut elements).take(len as usize).enumerate() {
            err::error_on_minusone(
                py,
                ffi::PyList_SetItem(list.as_ptr(), index as Py_ssize_t, obj?.into_ptr()),
            )?;
            counter += 1;
        }
        assert!(elements.next().is_none(), "Attempted to create PyTuple but `elements` was larger than reported by its `ExactSizeIterator` implementation.");
        assert_eq!(len, counter, "Attempted to create PyTuple but `elements` was smaller than reported by its `ExactSizeIterator` implementation.");
        Ok(ffi::PySequence_Tuple(list.as_ptr())
            .assume_owned(py)
            .cast_into_unchecked())
    }
}

pub(crate) fn array_into_tuple<'py, const N: usize>(
    py: Python<'py>,
    array: [Bound<'py, PyAny>; N],
) -> Bound<'py, PyTuple> {
    unsafe {
        let list = ffi::PyList_New(N.try_into().expect("0 < N <= 12"));
        for (index, obj) in array.into_iter().enumerate() {
            let rc = ffi::PyList_SetItem(list, index as ffi::Py_ssize_t, obj.into_ptr());
            err::error_on_minusone(py, rc).expect("failed to initialize tuple list staging buffer");
        }
        ffi::PySequence_Tuple(list)
            .assume_owned(py)
            .cast_into_unchecked()
    }
}

#[inline]
pub(crate) fn tuple_len(tuple: *mut ffi::PyObject) -> usize {
    unsafe { ffi::PyTuple_Size(tuple) as usize }
}

pub(crate) unsafe fn borrowed_tuple_item_for_extract<'a, 'py>(
    tuple: Borrowed<'a, 'py, PyTuple>,
    index: usize,
) -> PyResult<Borrowed<'a, 'py, PyAny>> {
    unsafe { ffi::PyTuple_GetItem(tuple.as_ptr(), index as Py_ssize_t).assume_borrowed_or_err(tuple.py()) }
}

pub(crate) unsafe fn borrowed_tuple_item_unchecked<'a, 'py>(
    tuple: Borrowed<'a, 'py, PyTuple>,
    index: usize,
) -> Borrowed<'a, 'py, PyAny> {
    unsafe {
        ffi::PyTuple_GetItem(tuple.as_ptr(), index as Py_ssize_t)
            .assume_borrowed_or_err(tuple.py())
            .expect("caller must provide an in-bounds tuple index")
    }
}

pub(crate) fn datetime_tzinfo<'py>(value: &Bound<'py, PyDateTime>) -> Option<Bound<'py, PyTzInfo>> {
    let res = value.getattr("tzinfo").ok()?;
    if res.is_none() {
        None
    } else {
        Some(unsafe { res.cast_into_unchecked() })
    }
}

pub(crate) fn time_tzinfo<'py>(value: &Bound<'py, PyTime>) -> Option<Bound<'py, PyTzInfo>> {
    let res = value.getattr("tzinfo").ok()?;
    if res.is_none() {
        None
    } else {
        Some(unsafe { res.cast_into_unchecked() })
    }
}
