use crate::err::{self, PyResult};
use crate::ffi::Py_ssize_t;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::{Borrowed, Bound, BoundObject};
use crate::sync::PyOnceLock;
use crate::types::any::PyAnyMethods;
use crate::types::{
    PyAny, PyCode, PyCodeInput, PyDateTime, PyFrozenSet, PyList, PyModule, PyString, PyTime,
    PyTuple, PyType, PyTypeMethods, PyTzInfo,
};
use crate::{ffi, IntoPyObject, IntoPyObjectExt, Py, Python};
use crate::py_result_ext::PyResultExt;

#[cfg(all(Py_3_10, not(Py_LIMITED_API)))]
use crate::ffi::{PyDateTime_DATE_GET_TZINFO, PyDateTime_TIME_GET_TZINFO, Py_IsNone};

#[inline]
pub(crate) fn dict_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyDict_Type
}

#[inline]
pub(crate) fn module_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyModule_Type
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
pub(crate) fn int_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyLong_Type
}

#[inline]
pub(crate) fn bytes_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyBytes_Type
}

#[inline]
pub(crate) fn bytearray_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyByteArray_Type
}

#[inline]
pub(crate) fn range_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyRange_Type
}

pub(crate) fn empty_code<'py>(
    py: Python<'py>,
    file_name: &std::ffi::CStr,
    _func_name: &std::ffi::CStr,
    _first_line_number: i32,
) -> Bound<'py, PyCode> {
    crate::types::PyCode::compile(py, c"", file_name, PyCodeInput::File)
        .expect("CPython backend failed to create an empty code object")
}

#[inline]
pub(crate) fn module_import<'py, N>(py: Python<'py>, name: N) -> PyResult<Bound<'py, PyModule>>
where
    N: IntoPyObject<'py, Target = PyString>,
{
    let name = name.into_pyobject_or_pyerr(py)?;
    unsafe {
        ffi::PyImport_Import(name.as_ptr())
            .assume_owned_or_err(py)
            .cast_into_unchecked()
    }
}

#[inline]
pub(crate) fn module_index<'py>(
    module: &Bound<'py, PyModule>,
    _dict: &Bound<'py, crate::types::PyDict>,
    __all__: &Bound<'py, PyString>,
) -> PyResult<Bound<'py, PyList>> {
    match module.getattr(__all__) {
        Ok(idx) => idx.cast_into().map_err(crate::err::PyErr::from),
        Err(err) => {
            if err.is_instance_of::<crate::exceptions::PyAttributeError>(module.py()) {
                let l = crate::types::PyList::empty(module.py());
                module.setattr(__all__, &l)?;
                Ok(l)
            } else {
                Err(err)
            }
        }
    }
}

#[inline]
pub(crate) fn module_filename_test_should_skip() -> bool {
    false
}

#[inline]
pub(crate) fn super_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(_py, "builtins", "super").unwrap().as_type_ptr()
}

#[inline]
pub(crate) fn weakref_reference_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(_py, "_weakref", "ReferenceType")
        .unwrap()
        .as_type_ptr()
}

#[inline]
pub(crate) fn traceback_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(_py, "types", "TracebackType")
        .unwrap()
        .as_type_ptr()
}

#[inline]
pub(crate) fn capsule_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyCapsule_Type
}

#[inline]
pub(crate) fn complex_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyComplex_Type
}

#[inline]
pub(crate) fn cfunction_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyCFunction_Type
}

#[cfg(not(Py_LIMITED_API))]
#[inline]
pub(crate) fn pyfunction_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(_py, "types", "FunctionType")
        .unwrap()
        .as_type_ptr()
}

#[inline]
pub(crate) fn code_type_object(py: Python<'_>) -> *mut ffi::PyTypeObject {
    static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    TYPE.import(py, "types", "CodeType").unwrap().as_type_ptr()
}

#[inline]
pub(crate) fn slice_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PySlice_Type
}

#[inline]
pub(crate) fn mappingproxy_type_object(_py: Python<'_>) -> *mut ffi::PyTypeObject {
    &raw mut ffi::PyDictProxy_Type
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

pub(crate) fn datetime_tzinfo<'py>(value: &Bound<'py, PyDateTime>) -> Option<Bound<'py, PyTzInfo>> {
    #[cfg(all(not(Py_3_10), not(Py_LIMITED_API)))]
    unsafe {
        let ptr = value.as_ptr() as *mut ffi::PyDateTime_DateTime;
        if (*ptr).hastzinfo != 0 {
            Some(
                (*ptr)
                    .tzinfo
                    .assume_borrowed(value.py())
                    .to_owned()
                    .cast_into_unchecked(),
            )
        } else {
            None
        }
    }

    #[cfg(all(Py_3_10, not(Py_LIMITED_API)))]
    unsafe {
        let res = PyDateTime_DATE_GET_TZINFO(value.as_ptr());
        if Py_IsNone(res) == 1 {
            None
        } else {
            Some(
                res.assume_borrowed(value.py())
                    .to_owned()
                    .cast_into_unchecked(),
            )
        }
    }

    #[cfg(Py_LIMITED_API)]
    unsafe {
        let tzinfo = value.getattr(crate::intern!(value.py(), "tzinfo")).ok()?;
        if tzinfo.is_none() {
            None
        } else {
            Some(tzinfo.cast_into_unchecked())
        }
    }
}

pub(crate) fn time_tzinfo<'py>(value: &Bound<'py, PyTime>) -> Option<Bound<'py, PyTzInfo>> {
    #[cfg(all(not(Py_3_10), not(Py_LIMITED_API)))]
    unsafe {
        let ptr = value.as_ptr() as *mut ffi::PyDateTime_Time;
        if (*ptr).hastzinfo != 0 {
            Some(
                (*ptr)
                    .tzinfo
                    .assume_borrowed(value.py())
                    .to_owned()
                    .cast_into_unchecked(),
            )
        } else {
            None
        }
    }

    #[cfg(all(Py_3_10, not(Py_LIMITED_API)))]
    unsafe {
        let res = PyDateTime_TIME_GET_TZINFO(value.as_ptr());
        if Py_IsNone(res) == 1 {
            None
        } else {
            Some(
                res.assume_borrowed(value.py())
                    .to_owned()
                    .cast_into_unchecked(),
            )
        }
    }

    #[cfg(Py_LIMITED_API)]
    unsafe {
        let tzinfo = value.getattr(crate::intern!(value.py(), "tzinfo")).ok()?;
        if tzinfo.is_none() {
            None
        } else {
            Some(tzinfo.cast_into_unchecked())
        }
    }
}
