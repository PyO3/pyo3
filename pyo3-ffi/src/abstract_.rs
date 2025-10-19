use crate::object::*;
use crate::pyport::Py_ssize_t;
#[cfg(any(Py_3_12, all(Py_3_8, not(Py_LIMITED_API))))]
use libc::size_t;
use std::ffi::{c_char, c_int};

#[inline]
#[cfg(all(
    not(Py_3_13), // CPython exposed as a function in 3.13, in object.h
    not(all(PyPy, not(Py_3_11))) // PyPy exposed as a function until PyPy 3.10, macro in 3.11+
))]
pub unsafe fn PyObject_DelAttrString(o: *mut PyObject, attr_name: *const c_char) -> c_int {
    PyObject_SetAttrString(o, attr_name, std::ptr::null_mut())
}

#[inline]
#[cfg(all(
    not(Py_3_13), // CPython exposed as a function in 3.13, in object.h
    not(all(PyPy, not(Py_3_11))) // PyPy exposed as a function until PyPy 3.10, macro in 3.11+
))]
pub unsafe fn PyObject_DelAttr(o: *mut PyObject, attr_name: *mut PyObject) -> c_int {
    PyObject_SetAttr(o, attr_name, std::ptr::null_mut())
}

extern "C" {
    #[cfg(all(
        not(PyPy),
        any(Py_3_10, all(not(Py_LIMITED_API), Py_3_9)) // Added to python in 3.9 but to limited API in 3.10
    ))]
    #[cfg_attr(PyPy, link_name = "PyPyObject_CallNoArgs")]
    pub fn PyObject_CallNoArgs(func: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Call")]
    pub fn PyObject_Call(
        callable_object: *mut PyObject,
        args: *mut PyObject,
        kw: *mut PyObject,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_CallObject")]
    pub fn PyObject_CallObject(
        callable_object: *mut PyObject,
        args: *mut PyObject,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_CallFunction")]
    pub fn PyObject_CallFunction(
        callable_object: *mut PyObject,
        format: *const c_char,
        ...
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_CallMethod")]
    pub fn PyObject_CallMethod(
        o: *mut PyObject,
        method: *const c_char,
        format: *const c_char,
        ...
    ) -> *mut PyObject;

    #[cfg(not(Py_3_13))]
    #[cfg_attr(PyPy, link_name = "_PyPyObject_CallFunction_SizeT")]
    pub fn _PyObject_CallFunction_SizeT(
        callable_object: *mut PyObject,
        format: *const c_char,
        ...
    ) -> *mut PyObject;
    #[cfg(not(Py_3_13))]
    #[cfg_attr(PyPy, link_name = "_PyPyObject_CallMethod_SizeT")]
    pub fn _PyObject_CallMethod_SizeT(
        o: *mut PyObject,
        method: *const c_char,
        format: *const c_char,
        ...
    ) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyObject_CallFunctionObjArgs")]
    pub fn PyObject_CallFunctionObjArgs(callable: *mut PyObject, ...) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_CallMethodObjArgs")]
    pub fn PyObject_CallMethodObjArgs(
        o: *mut PyObject,
        method: *mut PyObject,
        ...
    ) -> *mut PyObject;
}
#[cfg(any(Py_3_12, all(Py_3_8, not(Py_LIMITED_API))))]
pub const PY_VECTORCALL_ARGUMENTS_OFFSET: size_t =
    1 << (8 * std::mem::size_of::<size_t>() as size_t - 1);

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyObject_Vectorcall")]
    #[cfg(any(Py_3_12, all(Py_3_11, not(Py_LIMITED_API))))]
    pub fn PyObject_Vectorcall(
        callable: *mut PyObject,
        args: *const *mut PyObject,
        nargsf: size_t,
        kwnames: *mut PyObject,
    ) -> *mut PyObject;

    #[cfg(any(Py_3_12, all(Py_3_9, not(any(Py_LIMITED_API, PyPy)))))]
    pub fn PyObject_VectorcallMethod(
        name: *mut PyObject,
        args: *const *mut PyObject,
        nargsf: size_t,
        kwnames: *mut PyObject,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Type")]
    pub fn PyObject_Type(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Size")]
    pub fn PyObject_Size(o: *mut PyObject) -> Py_ssize_t;
}

#[inline]
pub unsafe fn PyObject_Length(o: *mut PyObject) -> Py_ssize_t {
    PyObject_Size(o)
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyObject_GetItem")]
    pub fn PyObject_GetItem(o: *mut PyObject, key: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_SetItem")]
    pub fn PyObject_SetItem(o: *mut PyObject, key: *mut PyObject, v: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_DelItemString")]
    pub fn PyObject_DelItemString(o: *mut PyObject, key: *const c_char) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_DelItem")]
    pub fn PyObject_DelItem(o: *mut PyObject, key: *mut PyObject) -> c_int;
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyObject_Format")]
    pub fn PyObject_Format(obj: *mut PyObject, format_spec: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_GetIter")]
    pub fn PyObject_GetIter(arg1: *mut PyObject) -> *mut PyObject;
}

// Before 3.8 PyIter_Check was defined in CPython as a macro,
// but the implementation of that in PyO3 did not work, see
// https://github.com/PyO3/pyo3/pull/2914
//
// This is a slow implementation which should function equivalently.
#[cfg(not(any(Py_3_8, PyPy)))]
#[inline]
pub unsafe fn PyIter_Check(o: *mut PyObject) -> c_int {
    crate::PyObject_HasAttrString(crate::Py_TYPE(o).cast(), c_str!("__next__").as_ptr())
}

extern "C" {
    #[cfg(any(Py_3_8, PyPy))]
    #[cfg_attr(PyPy, link_name = "PyPyIter_Check")]
    pub fn PyIter_Check(obj: *mut PyObject) -> c_int;

    #[cfg_attr(PyPy, link_name = "PyPyIter_Next")]
    pub fn PyIter_Next(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg(all(not(PyPy), Py_3_10))]
    #[cfg_attr(PyPy, link_name = "PyPyIter_Send")]
    pub fn PyIter_Send(
        iter: *mut PyObject,
        arg: *mut PyObject,
        presult: *mut *mut PyObject,
    ) -> PySendResult;

    #[cfg_attr(PyPy, link_name = "PyPyNumber_Check")]
    pub fn PyNumber_Check(o: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Add")]
    pub fn PyNumber_Add(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Subtract")]
    pub fn PyNumber_Subtract(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Multiply")]
    pub fn PyNumber_Multiply(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_MatrixMultiply")]
    pub fn PyNumber_MatrixMultiply(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_FloorDivide")]
    pub fn PyNumber_FloorDivide(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_TrueDivide")]
    pub fn PyNumber_TrueDivide(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Remainder")]
    pub fn PyNumber_Remainder(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Divmod")]
    pub fn PyNumber_Divmod(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Power")]
    pub fn PyNumber_Power(o1: *mut PyObject, o2: *mut PyObject, o3: *mut PyObject)
        -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Negative")]
    pub fn PyNumber_Negative(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Positive")]
    pub fn PyNumber_Positive(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Absolute")]
    pub fn PyNumber_Absolute(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Invert")]
    pub fn PyNumber_Invert(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Lshift")]
    pub fn PyNumber_Lshift(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Rshift")]
    pub fn PyNumber_Rshift(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_And")]
    pub fn PyNumber_And(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Xor")]
    pub fn PyNumber_Xor(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Or")]
    pub fn PyNumber_Or(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
}

// Defined as this macro in Python limited API, but relies on
// non-limited PyTypeObject. Don't expose this since it cannot be used.
#[cfg(not(any(Py_LIMITED_API, PyPy)))]
#[inline]
pub unsafe fn PyIndex_Check(o: *mut PyObject) -> c_int {
    let tp_as_number = (*Py_TYPE(o)).tp_as_number;
    (!tp_as_number.is_null() && (*tp_as_number).nb_index.is_some()) as c_int
}

extern "C" {
    #[cfg(any(all(Py_3_8, Py_LIMITED_API), PyPy))]
    #[link_name = "PyPyIndex_Check"]
    pub fn PyIndex_Check(o: *mut PyObject) -> c_int;

    #[cfg_attr(PyPy, link_name = "PyPyNumber_Index")]
    pub fn PyNumber_Index(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_AsSsize_t")]
    pub fn PyNumber_AsSsize_t(o: *mut PyObject, exc: *mut PyObject) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Long")]
    pub fn PyNumber_Long(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_Float")]
    pub fn PyNumber_Float(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_InPlaceAdd")]
    pub fn PyNumber_InPlaceAdd(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_InPlaceSubtract")]
    pub fn PyNumber_InPlaceSubtract(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_InPlaceMultiply")]
    pub fn PyNumber_InPlaceMultiply(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_InPlaceMatrixMultiply")]
    pub fn PyNumber_InPlaceMatrixMultiply(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_InPlaceFloorDivide")]
    pub fn PyNumber_InPlaceFloorDivide(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_InPlaceTrueDivide")]
    pub fn PyNumber_InPlaceTrueDivide(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_InPlaceRemainder")]
    pub fn PyNumber_InPlaceRemainder(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_InPlacePower")]
    pub fn PyNumber_InPlacePower(
        o1: *mut PyObject,
        o2: *mut PyObject,
        o3: *mut PyObject,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_InPlaceLshift")]
    pub fn PyNumber_InPlaceLshift(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_InPlaceRshift")]
    pub fn PyNumber_InPlaceRshift(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_InPlaceAnd")]
    pub fn PyNumber_InPlaceAnd(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_InPlaceXor")]
    pub fn PyNumber_InPlaceXor(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyNumber_InPlaceOr")]
    pub fn PyNumber_InPlaceOr(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    pub fn PyNumber_ToBase(n: *mut PyObject, base: c_int) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPySequence_Check")]
    pub fn PySequence_Check(o: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPySequence_Size")]
    pub fn PySequence_Size(o: *mut PyObject) -> Py_ssize_t;

    #[cfg(PyPy)]
    #[link_name = "PyPySequence_Length"]
    pub fn PySequence_Length(o: *mut PyObject) -> Py_ssize_t;
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PySequence_Length(o: *mut PyObject) -> Py_ssize_t {
    PySequence_Size(o)
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPySequence_Concat")]
    pub fn PySequence_Concat(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPySequence_Repeat")]
    pub fn PySequence_Repeat(o: *mut PyObject, count: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPySequence_GetItem")]
    pub fn PySequence_GetItem(o: *mut PyObject, i: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPySequence_GetSlice")]
    pub fn PySequence_GetSlice(o: *mut PyObject, i1: Py_ssize_t, i2: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPySequence_SetItem")]
    pub fn PySequence_SetItem(o: *mut PyObject, i: Py_ssize_t, v: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPySequence_DelItem")]
    pub fn PySequence_DelItem(o: *mut PyObject, i: Py_ssize_t) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPySequence_SetSlice")]
    pub fn PySequence_SetSlice(
        o: *mut PyObject,
        i1: Py_ssize_t,
        i2: Py_ssize_t,
        v: *mut PyObject,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPySequence_DelSlice")]
    pub fn PySequence_DelSlice(o: *mut PyObject, i1: Py_ssize_t, i2: Py_ssize_t) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPySequence_Tuple")]
    pub fn PySequence_Tuple(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPySequence_List")]
    pub fn PySequence_List(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPySequence_Fast")]
    pub fn PySequence_Fast(o: *mut PyObject, m: *const c_char) -> *mut PyObject;
    // skipped PySequence_Fast_GET_SIZE
    // skipped PySequence_Fast_GET_ITEM
    // skipped PySequence_Fast_GET_ITEMS
    pub fn PySequence_Count(o: *mut PyObject, value: *mut PyObject) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPySequence_Contains")]
    pub fn PySequence_Contains(seq: *mut PyObject, ob: *mut PyObject) -> c_int;
}

#[inline]
pub unsafe fn PySequence_In(o: *mut PyObject, value: *mut PyObject) -> c_int {
    PySequence_Contains(o, value)
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPySequence_Index")]
    pub fn PySequence_Index(o: *mut PyObject, value: *mut PyObject) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPySequence_InPlaceConcat")]
    pub fn PySequence_InPlaceConcat(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPySequence_InPlaceRepeat")]
    pub fn PySequence_InPlaceRepeat(o: *mut PyObject, count: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyMapping_Check")]
    pub fn PyMapping_Check(o: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyMapping_Size")]
    pub fn PyMapping_Size(o: *mut PyObject) -> Py_ssize_t;

    #[cfg(PyPy)]
    #[link_name = "PyPyMapping_Length"]
    pub fn PyMapping_Length(o: *mut PyObject) -> Py_ssize_t;
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyMapping_Length(o: *mut PyObject) -> Py_ssize_t {
    PyMapping_Size(o)
}

#[inline]
pub unsafe fn PyMapping_DelItemString(o: *mut PyObject, key: *mut c_char) -> c_int {
    PyObject_DelItemString(o, key)
}

#[inline]
pub unsafe fn PyMapping_DelItem(o: *mut PyObject, key: *mut PyObject) -> c_int {
    PyObject_DelItem(o, key)
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyMapping_HasKeyString")]
    pub fn PyMapping_HasKeyString(o: *mut PyObject, key: *const c_char) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyMapping_HasKey")]
    pub fn PyMapping_HasKey(o: *mut PyObject, key: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyMapping_Keys")]
    pub fn PyMapping_Keys(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyMapping_Values")]
    pub fn PyMapping_Values(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyMapping_Items")]
    pub fn PyMapping_Items(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyMapping_GetItemString")]
    pub fn PyMapping_GetItemString(o: *mut PyObject, key: *const c_char) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyMapping_SetItemString")]
    pub fn PyMapping_SetItemString(
        o: *mut PyObject,
        key: *const c_char,
        value: *mut PyObject,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_IsInstance")]
    pub fn PyObject_IsInstance(object: *mut PyObject, typeorclass: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_IsSubclass")]
    pub fn PyObject_IsSubclass(object: *mut PyObject, typeorclass: *mut PyObject) -> c_int;
}
