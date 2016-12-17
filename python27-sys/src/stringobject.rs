use libc::{c_char, c_int, c_long};
use pyport::Py_ssize_t;
use object::*;

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct PyStringObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub ob_size: Py_ssize_t,
    pub ob_shash: c_long,
    pub ob_sstate: c_int,
    pub ob_sval: [c_char; 1],
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyBaseString_Type: PyTypeObject;
    pub static mut PyString_Type: PyTypeObject;
}


#[inline(always)]
pub unsafe fn PyString_Check(op : *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_STRING_SUBCLASS)
}

#[inline(always)]
pub unsafe fn PyString_CheckExact(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyString_Type;
    (Py_TYPE(op) == u) as c_int
}


#[inline(always)]
pub unsafe fn PyString_GET_SIZE(op : *mut PyObject) -> Py_ssize_t {
    (*(op as *mut PyStringObject)).ob_size
}

#[inline(always)]
pub unsafe fn PyString_AS_STRING(op : *mut PyObject) -> *mut c_char {
    (*(op as *mut PyStringObject)).ob_sval.as_mut_ptr()
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyString_FromString(v: *const c_char) -> *mut PyObject;
    pub fn PyString_FromStringAndSize(v: *const c_char,
                                      len: Py_ssize_t) -> *mut PyObject;
    pub fn PyString_FromFormat(format: *const c_char, ...)
     -> *mut PyObject;
    pub fn PyString_Size(string: *mut PyObject) -> Py_ssize_t;
    pub fn PyString_AsString(string: *mut PyObject) -> *mut c_char;
    pub fn PyString_AsStringAndSize(obj: *mut PyObject,
                                    s: *mut *mut c_char,
                                    len: *mut Py_ssize_t) -> c_int;
    pub fn PyString_Concat(string: *mut *mut PyObject, newpart: *mut PyObject);
    pub fn PyString_ConcatAndDel(string: *mut *mut PyObject,
                                 newpart: *mut PyObject);
    pub fn _PyString_Resize(string: *mut *mut PyObject, newsize: Py_ssize_t)
     -> c_int;
    pub fn PyString_Format(format: *mut PyObject, args: *mut PyObject)
     -> *mut PyObject;
    pub fn PyString_InternInPlace(string: *mut *mut PyObject);
    pub fn PyString_InternFromString(v: *const c_char)
     -> *mut PyObject;
    pub fn PyString_Decode(s: *const c_char, size: Py_ssize_t,
                           encoding: *const c_char,
                           errors: *const c_char) -> *mut PyObject;
    pub fn PyString_AsDecodedObject(str: *mut PyObject,
                                    encoding: *const c_char,
                                    errors: *const c_char)
     -> *mut PyObject;
    pub fn PyString_Encode(s: *const c_char, size: Py_ssize_t,
                           encoding: *const c_char,
                           errors: *const c_char) -> *mut PyObject;
    pub fn PyString_AsEncodedObject(str: *mut PyObject,
                                    encoding: *const c_char,
                                    errors: *const c_char)
     -> *mut PyObject;
    
    /*
    pub fn PyString_Repr(arg1: *mut PyObject, arg2: c_int)
     -> *mut PyObject;
    pub fn _PyString_Eq(arg1: *mut PyObject, arg2: *mut PyObject)
     -> c_int;
    pub fn _PyString_FormatLong(arg1: *mut PyObject, arg2: c_int,
                                arg3: c_int, arg4: c_int,
                                arg5: *mut *mut c_char,
                                arg6: *mut c_int) -> *mut PyObject;
    pub fn PyString_DecodeEscape(arg1: *const c_char,
                                 arg2: Py_ssize_t,
                                 arg3: *const c_char,
                                 arg4: Py_ssize_t,
                                 arg5: *const c_char)
     -> *mut PyObject;
    pub fn PyString_InternImmortal(arg1: *mut *mut PyObject);
    pub fn _Py_ReleaseInternedStrings();
    pub fn _PyString_Join(sep: *mut PyObject, x: *mut PyObject)
     -> *mut PyObject;
    pub fn PyString_AsEncodedString(str: *mut PyObject,
                                    encoding: *const c_char,
                                    errors: *const c_char)
     -> *mut PyObject;
    pub fn PyString_AsDecodedString(str: *mut PyObject,
                                    encoding: *const c_char,
                                    errors: *const c_char)
     -> *mut PyObject;

    pub fn _PyString_InsertThousandsGroupingLocale(buffer:
                                                       *mut c_char,
                                                   n_buffer: Py_ssize_t,
                                                   digits:
                                                       *mut c_char,
                                                   n_digits: Py_ssize_t,
                                                   min_width: Py_ssize_t)
     -> Py_ssize_t;
    pub fn _PyString_InsertThousandsGrouping(buffer: *mut c_char,
                                             n_buffer: Py_ssize_t,
                                             digits: *mut c_char,
                                             n_digits: Py_ssize_t,
                                             min_width: Py_ssize_t,
                                             grouping: *const c_char,
                                             thousands_sep:
                                                 *const c_char)
     -> Py_ssize_t;
    pub fn _PyBytes_FormatAdvanced(obj: *mut PyObject,
                                   format_spec: *mut c_char,
                                   format_spec_len: Py_ssize_t)
     -> *mut PyObject;*/
}

