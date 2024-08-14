use std::os::raw::{c_char, c_long};
use std::ptr;

use pyo3_ffi::*;

static mut MODULE_DEF: PyModuleDef = PyModuleDef {
    m_base: PyModuleDef_HEAD_INIT(),
    m_name: c_str!("string_sum").as_ptr(),
    m_doc: c_str!("A Python module written in Rust.").as_ptr(),
    m_size: 0,
    m_methods: unsafe { METHODS as *const [PyMethodDef] as *mut PyMethodDef },
    m_slots: std::ptr::null_mut(),
    m_traverse: None,
    m_clear: None,
    m_free: None,
};

static mut METHODS: &[PyMethodDef] = &[
    PyMethodDef {
        ml_name: c_str!("sum_as_string").as_ptr(),
        ml_meth: PyMethodDefPointer {
            _PyCFunctionFast: sum_as_string,
        },
        ml_flags: METH_FASTCALL,
        ml_doc: c_str!("returns the sum of two integers as a string").as_ptr(),
    },
    // A zeroed PyMethodDef to mark the end of the array.
    PyMethodDef::zeroed(),
];

// The module initialization function, which must be named `PyInit_<your_module>`.
#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyInit_string_sum() -> *mut PyObject {
    PyModule_Create(ptr::addr_of_mut!(MODULE_DEF))
}

/// A helper to parse function arguments
/// If we used PyO3's proc macros they'd handle all of this boilerplate for us :)
unsafe fn parse_arg_as_i32(obj: *mut PyObject, n_arg: usize) -> Option<i32> {
    if PyLong_Check(obj) == 0 {
        let msg = format!(
            "sum_as_string expected an int for positional argument {}\0",
            n_arg
        );
        PyErr_SetString(PyExc_TypeError, msg.as_ptr().cast::<c_char>());
        return None;
    }

    // Let's keep the behaviour consistent on platforms where `c_long` is bigger than 32 bits.
    // In particular, it is an i32 on Windows but i64 on most Linux systems
    let mut overflow = 0;
    let i_long: c_long = PyLong_AsLongAndOverflow(obj, &mut overflow);

    if overflow != 0 {
        raise_overflowerror(obj);
        None
    } else if let Ok(i) = i_long.try_into() {
        Some(i)
    } else {
        raise_overflowerror(obj);
        None
    }
}

unsafe fn raise_overflowerror(obj: *mut PyObject) {
    let obj_repr = PyObject_Str(obj);
    if !obj_repr.is_null() {
        let mut size = 0;
        let p = PyUnicode_AsUTF8AndSize(obj_repr, &mut size);
        if !p.is_null() {
            let s = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                p.cast::<u8>(),
                size as usize,
            ));
            let msg = format!("cannot fit {} in 32 bits\0", s);

            PyErr_SetString(PyExc_OverflowError, msg.as_ptr().cast::<c_char>());
        }
        Py_DECREF(obj_repr);
    }
}

pub unsafe extern "C" fn sum_as_string(
    _self: *mut PyObject,
    args: *mut *mut PyObject,
    nargs: Py_ssize_t,
) -> *mut PyObject {
    if nargs != 2 {
        PyErr_SetString(
            PyExc_TypeError,
            c_str!("sum_as_string expected 2 positional arguments").as_ptr(),
        );
        return std::ptr::null_mut();
    }

    let (first, second) = (*args, *args.add(1));

    let first = match parse_arg_as_i32(first, 1) {
        Some(x) => x,
        None => return std::ptr::null_mut(),
    };
    let second = match parse_arg_as_i32(second, 2) {
        Some(x) => x,
        None => return std::ptr::null_mut(),
    };

    match first.checked_add(second) {
        Some(sum) => {
            let string = sum.to_string();
            PyUnicode_FromStringAndSize(string.as_ptr().cast::<c_char>(), string.len() as isize)
        }
        None => {
            PyErr_SetString(
                PyExc_OverflowError,
                c_str!("arguments too large to add").as_ptr(),
            );
            std::ptr::null_mut()
        }
    }
}
