use libc::{c_void, c_char, c_int};
use pyport::Py_ssize_t;
use std::ptr;
use object::*;

#[inline]
pub unsafe fn PyObject_DelAttrString(o: *mut PyObject, attr_name: *const c_char) -> c_int {
    PyObject_SetAttrString(o, attr_name, ptr::null_mut())
}

#[inline]
pub unsafe fn PyObject_DelAttr(o: *mut PyObject, attr_name: *mut PyObject) -> c_int {
    PyObject_SetAttr(o, attr_name, ptr::null_mut())
}

#[link(name = "python2.7")]
extern "C" {
    pub fn PyObject_Cmp(o1: *mut PyObject, o2: *mut PyObject,
                        result: *mut c_int) -> c_int;
    pub fn PyObject_Call(callable_object: *mut PyObject, args: *mut PyObject,
                         kw: *mut PyObject) -> *mut PyObject;
    pub fn PyObject_CallObject(callable_object: *mut PyObject,
                               args: *mut PyObject) -> *mut PyObject;
    pub fn PyObject_CallFunction(callable_object: *mut PyObject,
                                 format: *mut c_char, ...)
     -> *mut PyObject;
    pub fn PyObject_CallMethod(o: *mut PyObject, m: *mut c_char,
                               format: *mut c_char, ...)
     -> *mut PyObject;
    fn _PyObject_CallFunction_SizeT(callable: *mut PyObject,
                                        format: *mut c_char, ...)
     -> *mut PyObject;
    fn _PyObject_CallMethod_SizeT(o: *mut PyObject,
                                      name: *mut c_char,
                                      format: *mut c_char, ...)
     -> *mut PyObject;
    pub fn PyObject_CallFunctionObjArgs(callable: *mut PyObject, ...)
     -> *mut PyObject;
    pub fn PyObject_CallMethodObjArgs(o: *mut PyObject, m: *mut PyObject, ...)
     -> *mut PyObject;
    pub fn PyObject_Type(o: *mut PyObject) -> *mut PyObject;
    pub fn PyObject_Size(o: *mut PyObject) -> Py_ssize_t;
    pub fn _PyObject_LengthHint(o: *mut PyObject, arg1: Py_ssize_t)
     -> Py_ssize_t;
    pub fn PyObject_GetItem(o: *mut PyObject, key: *mut PyObject)
     -> *mut PyObject;
    pub fn PyObject_SetItem(o: *mut PyObject, key: *mut PyObject,
                            v: *mut PyObject) -> c_int;
    pub fn PyObject_DelItemString(o: *mut PyObject, key: *mut c_char)
     -> c_int;
    pub fn PyObject_DelItem(o: *mut PyObject, key: *mut PyObject)
     -> c_int;
    pub fn PyObject_AsCharBuffer(obj: *mut PyObject,
                                 buffer: *mut *const c_char,
                                 buffer_len: *mut Py_ssize_t)
     -> c_int;
    pub fn PyObject_CheckReadBuffer(obj: *mut PyObject) -> c_int;
    pub fn PyObject_AsReadBuffer(obj: *mut PyObject,
                                 buffer: *mut *const c_void,
                                 buffer_len: *mut Py_ssize_t)
     -> c_int;
    pub fn PyObject_AsWriteBuffer(obj: *mut PyObject,
                                  buffer: *mut *mut c_void,
                                  buffer_len: *mut Py_ssize_t)
     -> c_int;

    pub fn PyObject_GetBuffer(obj: *mut PyObject, view: *mut Py_buffer,
                              flags: c_int) -> c_int;

    pub fn PyBuffer_GetPointer(view: *mut Py_buffer, indices: *mut Py_ssize_t)
     -> *mut c_void;
    pub fn PyBuffer_SizeFromFormat(arg1: *const c_char)
     -> c_int;
    pub fn PyBuffer_ToContiguous(buf: *mut c_void,
                                 view: *mut Py_buffer, len: Py_ssize_t,
                                 fort: c_char) -> c_int;
    pub fn PyBuffer_FromContiguous(view: *mut Py_buffer,
                                   buf: *mut c_void, len: Py_ssize_t,
                                   fort: c_char) -> c_int;
    pub fn PyObject_CopyData(dest: *mut PyObject, src: *mut PyObject)
     -> c_int;
    pub fn PyBuffer_IsContiguous(view: *mut Py_buffer, fort: c_char)
     -> c_int;
    pub fn PyBuffer_FillContiguousStrides(ndims: c_int,
                                          shape: *mut Py_ssize_t,
                                          strides: *mut Py_ssize_t,
                                          itemsize: c_int,
                                          fort: c_char);
    pub fn PyBuffer_FillInfo(view: *mut Py_buffer, o: *mut PyObject,
                             buf: *mut c_void, len: Py_ssize_t,
                             readonly: c_int, flags: c_int)
     -> c_int;
    pub fn PyBuffer_Release(view: *mut Py_buffer);
    pub fn PyObject_Format(obj: *mut PyObject, format_spec: *mut PyObject)
     -> *mut PyObject;
    pub fn PyObject_GetIter(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyIter_Next(arg1: *mut PyObject) -> *mut PyObject;
    fn _PyObject_NextNotImplemented(arg1: *mut PyObject) -> *mut PyObject;

    pub fn PyNumber_Check(o: *mut PyObject) -> c_int;
    pub fn PyNumber_Add(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_Subtract(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_Multiply(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_Divide(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_FloorDivide(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_TrueDivide(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_Remainder(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_Divmod(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_Power(o1: *mut PyObject, o2: *mut PyObject,
                          o3: *mut PyObject) -> *mut PyObject;
    pub fn PyNumber_Negative(o: *mut PyObject) -> *mut PyObject;
    pub fn PyNumber_Positive(o: *mut PyObject) -> *mut PyObject;
    pub fn PyNumber_Absolute(o: *mut PyObject) -> *mut PyObject;
    pub fn PyNumber_Invert(o: *mut PyObject) -> *mut PyObject;
    pub fn PyNumber_Lshift(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_Rshift(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_And(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_Xor(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_Or(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject;
    pub fn PyNumber_Index(o: *mut PyObject) -> *mut PyObject;
    pub fn PyNumber_AsSsize_t(o: *mut PyObject, exc: *mut PyObject)
     -> Py_ssize_t;
    fn _PyNumber_ConvertIntegralToInt(integral: *mut PyObject,
                                          error_format: *const c_char)
     -> *mut PyObject;
    pub fn PyNumber_Int(o: *mut PyObject) -> *mut PyObject;
    pub fn PyNumber_Long(o: *mut PyObject) -> *mut PyObject;
    pub fn PyNumber_Float(o: *mut PyObject) -> *mut PyObject;
    pub fn PyNumber_InPlaceAdd(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_InPlaceSubtract(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_InPlaceMultiply(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_InPlaceDivide(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_InPlaceFloorDivide(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_InPlaceTrueDivide(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_InPlaceRemainder(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_InPlacePower(o1: *mut PyObject, o2: *mut PyObject,
                                 o3: *mut PyObject) -> *mut PyObject;
    pub fn PyNumber_InPlaceLshift(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_InPlaceRshift(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_InPlaceAnd(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_InPlaceXor(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_InPlaceOr(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyNumber_ToBase(n: *mut PyObject, base: c_int)
     -> *mut PyObject;
    pub fn PySequence_Check(o: *mut PyObject) -> c_int;
    pub fn PySequence_Size(o: *mut PyObject) -> Py_ssize_t;
    pub fn PySequence_Length(o: *mut PyObject) -> Py_ssize_t;
    pub fn PySequence_Concat(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PySequence_Repeat(o: *mut PyObject, count: Py_ssize_t)
     -> *mut PyObject;
    pub fn PySequence_GetItem(o: *mut PyObject, i: Py_ssize_t)
     -> *mut PyObject;
    pub fn PySequence_GetSlice(o: *mut PyObject, i1: Py_ssize_t,
                               i2: Py_ssize_t) -> *mut PyObject;
    pub fn PySequence_SetItem(o: *mut PyObject, i: Py_ssize_t,
                              v: *mut PyObject) -> c_int;
    pub fn PySequence_DelItem(o: *mut PyObject, i: Py_ssize_t)
     -> c_int;
    pub fn PySequence_SetSlice(o: *mut PyObject, i1: Py_ssize_t,
                               i2: Py_ssize_t, v: *mut PyObject)
     -> c_int;
    pub fn PySequence_DelSlice(o: *mut PyObject, i1: Py_ssize_t,
                               i2: Py_ssize_t) -> c_int;
    pub fn PySequence_Tuple(o: *mut PyObject) -> *mut PyObject;
    pub fn PySequence_List(o: *mut PyObject) -> *mut PyObject;
    pub fn PySequence_Fast(o: *mut PyObject, m: *const c_char)
     -> *mut PyObject;
    pub fn PySequence_Count(o: *mut PyObject, value: *mut PyObject)
     -> Py_ssize_t;
    pub fn PySequence_Contains(seq: *mut PyObject, ob: *mut PyObject)
     -> c_int;
    pub fn _PySequence_IterSearch(seq: *mut PyObject, obj: *mut PyObject,
                                  operation: c_int) -> Py_ssize_t;
    pub fn PySequence_In(o: *mut PyObject, value: *mut PyObject)
     -> c_int;
    pub fn PySequence_Index(o: *mut PyObject, value: *mut PyObject)
     -> Py_ssize_t;
    pub fn PySequence_InPlaceConcat(o1: *mut PyObject, o2: *mut PyObject)
     -> *mut PyObject;
    pub fn PySequence_InPlaceRepeat(o: *mut PyObject, count: Py_ssize_t)
     -> *mut PyObject;
    pub fn PyMapping_Check(o: *mut PyObject) -> c_int;
    pub fn PyMapping_Size(o: *mut PyObject) -> Py_ssize_t;
    pub fn PyMapping_Length(o: *mut PyObject) -> Py_ssize_t;
    pub fn PyMapping_HasKeyString(o: *mut PyObject, key: *mut c_char)
     -> c_int;
    pub fn PyMapping_HasKey(o: *mut PyObject, key: *mut PyObject)
     -> c_int;
    pub fn PyMapping_GetItemString(o: *mut PyObject, key: *mut c_char)
     -> *mut PyObject;
    pub fn PyMapping_SetItemString(o: *mut PyObject, key: *mut c_char,
                                   value: *mut PyObject) -> c_int;
    pub fn PyObject_IsInstance(object: *mut PyObject,
                               typeorclass: *mut PyObject) -> c_int;
    pub fn PyObject_IsSubclass(object: *mut PyObject,
                               typeorclass: *mut PyObject) -> c_int;
}


#[inline]
pub unsafe fn PyObject_CheckBuffer(obj: *mut PyObject) -> c_int {
    let t = (*obj).ob_type;
    let b = (*t).tp_as_buffer;
    (!b.is_null() &&
     (PyType_HasFeature(t, Py_TPFLAGS_HAVE_NEWBUFFER) != 0) &&
     ((*b).bf_getbuffer.is_some())) as c_int
}


#[inline]
pub unsafe fn PyIter_Check(obj: *mut PyObject) -> c_int {
    let t = (*obj).ob_type;
    (PyType_HasFeature(t, Py_TPFLAGS_HAVE_ITER) != 0 &&
      match (*t).tp_iternext {
        None => false,
        Some(f) => f as *const c_void != _PyObject_NextNotImplemented as *const c_void,
      }) as c_int
}

#[inline]
pub unsafe fn PyIndex_Check(obj: *mut PyObject) -> c_int {
    let t = (*obj).ob_type;
    let n = (*t).tp_as_number;
    (!n.is_null() && PyType_HasFeature(t, Py_TPFLAGS_HAVE_INDEX) != 0 && (*n).nb_index.is_some()) as c_int
}

#[inline]
pub unsafe fn PySequence_Fast_GET_SIZE(o : *mut PyObject) -> Py_ssize_t {
    if ::listobject::PyList_Check(o) != 0 {
        ::listobject::PyList_GET_SIZE(o)
    } else {
        ::tupleobject::PyTuple_GET_SIZE(o)
    }
}

#[inline]
pub unsafe fn PySequence_Fast_GET_ITEM(o : *mut PyObject, i : Py_ssize_t) -> *mut PyObject {
    if ::listobject::PyList_Check(o) != 0 {
        ::listobject::PyList_GET_ITEM(o, i)
    } else {
        ::tupleobject::PyTuple_GET_ITEM(o, i)
    }
}

#[inline]
pub unsafe fn PySequence_Fast_ITEMS(o : *mut PyObject) -> *mut *mut PyObject {
    if ::listobject::PyList_Check(o) != 0 {
        (*(o as *mut ::listobject::PyListObject)).ob_item
    } else {
        (*(o as *mut ::tupleobject::PyTupleObject)).ob_item.as_mut_ptr()
    }
}

#[inline]
pub unsafe fn PySequence_ITEM(o : *mut PyObject, i : Py_ssize_t) -> *mut PyObject {
    (*(*Py_TYPE(o)).tp_as_sequence).sq_item.unwrap()(o, i)
}

pub const PY_ITERSEARCH_COUNT    : c_int = 1;
pub const PY_ITERSEARCH_INDEX    : c_int = 2;
pub const PY_ITERSEARCH_CONTAINS : c_int = 3;

#[inline]
pub unsafe fn PyMapping_DelItemString(o : *mut PyObject, key : *mut c_char) -> c_int {
    PyObject_DelItemString(o, key)
}

#[inline]
pub unsafe fn PyMapping_DelItem(o : *mut PyObject, key : *mut PyObject) -> c_int {
    PyObject_DelItem(o, key)
}

#[inline]
pub unsafe fn PyMapping_Keys(o : *mut PyObject) -> *mut PyObject {
    PyObject_CallMethod(o, "keys\0".as_ptr() as *mut i8, ptr::null_mut())
}

#[inline]
pub unsafe fn PyMapping_Values(o : *mut PyObject) -> *mut PyObject {
    PyObject_CallMethod(o, "values\0".as_ptr() as *mut i8, ptr::null_mut())
}

#[inline]
pub unsafe fn PyMapping_Items(o : *mut PyObject) -> *mut PyObject {
    PyObject_CallMethod(o, "items\0".as_ptr() as *mut i8, ptr::null_mut())
}

