use crate::{
    vectorcallfunc, PyListObject, PyList_Check, PyList_GET_ITEM, PyList_GET_SIZE, PyObject,
    PyTupleObject, PyTuple_GET_ITEM, PyTuple_GET_SIZE, Py_TYPE, Py_ssize_t,
};
#[cfg(all(any(not(PyPy), not(Py_3_11)), not(Py_3_12)))]
use std::ffi::c_char;
#[cfg(not(Py_3_12))]
use std::ffi::c_int;

#[cfg(not(Py_3_11))]
use crate::Py_buffer;

#[cfg(not(any(PyPy, Py_3_11)))]
use crate::{PyCallable_Check, PyType_HasFeature, Py_TPFLAGS_HAVE_VECTORCALL};
#[cfg(not(Py_3_12))]
use crate::{PyThreadState, PyThreadState_GET, PyTuple_Check};
use libc::size_t;

// skipped private _PyObject_CallMethodId
// skipped private _PyStack_AsDict

#[cfg(not(Py_3_12))]
extern_libpython! {
    #[cfg(not(PyPy))]
    fn _Py_CheckFunctionResult(
        tstate: *mut PyThreadState,
        callable: *mut PyObject,
        result: *mut PyObject,
        where_: *const c_char,
    ) -> *mut PyObject;

    #[cfg(not(PyPy))]
    fn _PyObject_MakeTpCall(
        tstate: *mut PyThreadState,
        callable: *mut PyObject,
        args: *const *mut PyObject,
        nargs: Py_ssize_t,
        keywords: *mut PyObject,
    ) -> *mut PyObject;
}

#[cfg(not(Py_3_12))]
const PY_VECTORCALL_ARGUMENTS_OFFSET: size_t =
    1 << (8 * std::mem::size_of::<size_t>() as size_t - 1);

#[cfg(Py_3_12)] // public API from 3.12
use crate::PY_VECTORCALL_ARGUMENTS_OFFSET;

#[inline(always)]
pub unsafe fn PyVectorcall_NARGS(n: size_t) -> Py_ssize_t {
    let n = n & !PY_VECTORCALL_ARGUMENTS_OFFSET;
    n.try_into().expect("cannot fail due to mask")
}

#[cfg(any(PyPy, Py_3_11))]
extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyVectorcall_Function")]
    pub fn PyVectorcall_Function(callable: *mut PyObject) -> Option<vectorcallfunc>;
}

#[cfg(not(any(PyPy, Py_3_11)))]
#[inline(always)]
pub unsafe fn PyVectorcall_Function(callable: *mut PyObject) -> Option<vectorcallfunc> {
    assert!(!callable.is_null());
    let tp = crate::Py_TYPE(callable);
    if PyType_HasFeature(tp, Py_TPFLAGS_HAVE_VECTORCALL) == 0 {
        return None;
    }
    assert!(PyCallable_Check(callable) > 0);
    let offset = (*tp).tp_vectorcall_offset;
    assert!(offset > 0);
    let ptr = callable.cast::<c_char>().offset(offset).cast();
    *ptr
}

#[cfg(not(Py_3_12))]
#[cfg(not(PyPy))]
#[inline(always)]
unsafe fn _PyObject_VectorcallTstate(
    tstate: *mut PyThreadState,
    callable: *mut PyObject,
    args: *const *mut PyObject,
    nargsf: size_t,
    kwnames: *mut PyObject,
) -> *mut PyObject {
    assert!(kwnames.is_null() || PyTuple_Check(kwnames) > 0);
    assert!(!args.is_null() || PyVectorcall_NARGS(nargsf) == 0);

    match PyVectorcall_Function(callable) {
        None => {
            let nargs = PyVectorcall_NARGS(nargsf);
            _PyObject_MakeTpCall(tstate, callable, args, nargs, kwnames)
        }
        Some(func) => {
            let res = func(callable, args, nargsf, kwnames);
            _Py_CheckFunctionResult(tstate, callable, res, std::ptr::null_mut())
        }
    }
}

#[cfg(not(any(PyPy, GraalPy, Py_3_11)))] // exported as a function from 3.11, see abstract.rs
#[inline(always)]
pub unsafe fn PyObject_Vectorcall(
    callable: *mut PyObject,
    args: *const *mut PyObject,
    nargsf: size_t,
    kwnames: *mut PyObject,
) -> *mut PyObject {
    _PyObject_VectorcallTstate(PyThreadState_GET(), callable, args, nargsf, kwnames)
}

#[cfg(not(Py_3_12))]
#[cfg(not(PyPy))]
#[inline(always)]
pub unsafe fn PyObject_CallOneArg(func: *mut PyObject, arg: *mut PyObject) -> *mut PyObject {
    assert!(!arg.is_null());
    let args_array = [std::ptr::null_mut(), arg];
    let args = args_array.as_ptr().offset(1); // For PY_VECTORCALL_ARGUMENTS_OFFSET
    let tstate = PyThreadState_GET();
    let nargsf = 1 | PY_VECTORCALL_ARGUMENTS_OFFSET;
    _PyObject_VectorcallTstate(tstate, func, args, nargsf, std::ptr::null_mut())
}

#[cfg(Py_3_12)]
extern_libpython! {
    #[cfg_attr(
        all(not(any(PyPy, GraalPy)), not(Py_3_9)),
        link_name = "_PyObject_VectorcallDict"
    )]
    #[cfg_attr(all(PyPy, not(Py_3_9)), link_name = "_PyPyObject_VectorcallDict")]
    #[cfg_attr(all(PyPy, Py_3_9), link_name = "PyPyObject_VectorcallDict")]
    pub fn PyObject_VectorcallDict(
        callable: *mut PyObject,
        args: *const *mut PyObject,
        nargsf: size_t,
        kwdict: *mut PyObject,
    ) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyObject_CallOneArg")]
    pub fn PyObject_CallOneArg(func: *mut PyObject, arg: *mut PyObject) -> *mut PyObject;
}

#[cfg(all(Py_3_9, not(PyPy)))]
#[inline(always)]
pub unsafe fn PyObject_CallMethodNoArgs(
    self_: *mut PyObject,
    name: *mut PyObject,
) -> *mut PyObject {
    crate::PyObject_VectorcallMethod(
        name,
        &self_,
        1 | PY_VECTORCALL_ARGUMENTS_OFFSET,
        std::ptr::null_mut(),
    )
}

#[cfg(all(Py_3_9, not(PyPy)))]
#[inline(always)]
pub unsafe fn PyObject_CallMethodOneArg(
    self_: *mut PyObject,
    name: *mut PyObject,
    arg: *mut PyObject,
) -> *mut PyObject {
    let args = [self_, arg];
    assert!(!arg.is_null());
    crate::PyObject_VectorcallMethod(
        name,
        args.as_ptr(),
        2 | PY_VECTORCALL_ARGUMENTS_OFFSET,
        std::ptr::null_mut(),
    )
}

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyObject_LengthHint")]
    pub fn PyObject_LengthHint(o: *mut PyObject, arg1: Py_ssize_t) -> Py_ssize_t;

    #[cfg(not(Py_3_11))] // moved to src/buffer.rs from 3.11
    #[cfg(all(Py_3_9, not(PyPy)))]
    pub fn PyObject_CheckBuffer(obj: *mut PyObject) -> c_int;
}

#[cfg(not(any(Py_3_9, PyPy)))]
#[inline]
pub unsafe fn PyObject_CheckBuffer(o: *mut PyObject) -> c_int {
    let tp_as_buffer = (*crate::Py_TYPE(o)).tp_as_buffer;
    (!tp_as_buffer.is_null() && (*tp_as_buffer).bf_getbuffer.is_some()) as c_int
}

#[cfg(not(Py_3_11))] // moved to src/buffer.rs from 3.11
extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyObject_GetBuffer")]
    pub fn PyObject_GetBuffer(obj: *mut PyObject, view: *mut Py_buffer, flags: c_int) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyBuffer_GetPointer")]
    pub fn PyBuffer_GetPointer(
        view: *mut Py_buffer,
        indices: *mut Py_ssize_t,
    ) -> *mut std::ffi::c_void;
    #[cfg_attr(PyPy, link_name = "PyPyBuffer_SizeFromFormat")]
    pub fn PyBuffer_SizeFromFormat(format: *const c_char) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyBuffer_ToContiguous")]
    pub fn PyBuffer_ToContiguous(
        buf: *mut std::ffi::c_void,
        view: *mut Py_buffer,
        len: Py_ssize_t,
        order: c_char,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyBuffer_FromContiguous")]
    pub fn PyBuffer_FromContiguous(
        view: *mut Py_buffer,
        buf: *mut std::ffi::c_void,
        len: Py_ssize_t,
        order: c_char,
    ) -> c_int;
    pub fn PyObject_CopyData(dest: *mut PyObject, src: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyBuffer_IsContiguous")]
    pub fn PyBuffer_IsContiguous(view: *const Py_buffer, fort: c_char) -> c_int;
    pub fn PyBuffer_FillContiguousStrides(
        ndims: c_int,
        shape: *mut Py_ssize_t,
        strides: *mut Py_ssize_t,
        itemsize: c_int,
        fort: c_char,
    );
    #[cfg_attr(PyPy, link_name = "PyPyBuffer_FillInfo")]
    pub fn PyBuffer_FillInfo(
        view: *mut Py_buffer,
        o: *mut PyObject,
        buf: *mut std::ffi::c_void,
        len: Py_ssize_t,
        readonly: c_int,
        flags: c_int,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyBuffer_Release")]
    pub fn PyBuffer_Release(view: *mut Py_buffer);
}

#[inline(always)]
pub unsafe fn PySequence_ITEM(seq: *mut PyObject, i: Py_ssize_t) -> *mut PyObject {
    (*(*Py_TYPE(seq)).tp_as_sequence).sq_item.unwrap_unchecked()(seq, i)
}

#[inline(always)]
pub unsafe fn PySequence_Fast_GET_SIZE(seq: *mut PyObject) -> Py_ssize_t {
    if PyList_Check(seq) == 1 {
        PyList_GET_SIZE(seq)
    } else {
        PyTuple_GET_SIZE(seq)
    }
}

#[inline(always)]
pub unsafe fn PySequence_Fast_GET_ITEM(seq: *mut PyObject, i: Py_ssize_t) -> *mut PyObject {
    if PyList_Check(seq) == 1 {
        PyList_GET_ITEM(seq, i)
    } else {
        PyTuple_GET_ITEM(seq, i)
    }
}

#[inline(always)]
pub unsafe fn PySequence_Fast_ITEMS(seq: *mut PyObject) -> *mut *mut PyObject {
    if PyList_Check(seq) == 1 {
        (*seq.cast::<PyListObject>()).ob_item
    } else {
        (*seq.cast::<PyTupleObject>()).ob_item.as_mut_ptr()
    }
}
