// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::os::raw::{c_long, c_uchar};

extern crate num_traits;
use self::num_traits::cast::cast;

use super::num_common::{err_if_invalid_value, IS_LITTLE_ENDIAN};
use conversion::{FromPyObject, IntoPyObject, ToPyObject};
use err::{PyErr, PyResult};
use ffi;
use instance::{Py, PyObjectWithGIL};
use object::PyObject;
use python::{IntoPyPointer, Python, ToPyPointer};
use types::{exceptions, PyObjectRef};

/// Represents a Python `int` object.
///
/// Note that in Python 2.x, `int` and `long` are different types.
///
/// You can usually avoid directly working with this type
/// by using [`ToPyObject`](trait.ToPyObject.html)
/// and [extract](struct.PyObject.html#method.extract)
/// with the primitive Rust integer types.
#[repr(transparent)]
pub struct PyInt(PyObject);

pyobject_native_type!(PyInt, ffi::PyInt_Type, ffi::PyInt_Check);

/// In Python 2.x, represents a Python `long` object.
/// Both `PyInt` and `PyLong` refer to the same type on Python 3.x.
///
/// You can usually avoid directly working with this type
/// by using [`ToPyObject`](trait.ToPyObject.html)
/// and [extract](struct.PyObject.html#method.extract)
/// with the primitive Rust integer types.
pub struct PyLong(PyObject);

pyobject_native_type!(PyLong, ffi::PyLong_Type, ffi::PyLong_Check);

impl PyInt {
    /// Creates a new Python 2.7 `int` object.
    ///
    /// Note: you might want to call `val.to_object(py)` instead
    /// to avoid truncation if the value does not fit into a `c_long`,
    /// and to make your code compatible with Python 3.x.
    pub fn new(_py: Python, val: c_long) -> Py<PyInt> {
        unsafe { Py::from_owned_ptr_or_panic(ffi::PyLong_FromLong(val)) }
    }

    /// Gets the value of this integer.
    ///
    /// Warning: `PyInt::value()` is only supported for Python 2.7 `int` objects,
    /// but not for `long` objects.
    /// In almost all cases, you can avoid the distinction between these types
    /// by simply calling `obj.extract::<i32>()`.
    pub fn value(&self) -> c_long {
        unsafe { ffi::PyInt_AS_LONG(self.0.as_ptr()) }
    }
}

macro_rules! int_fits_c_long(
    ($rust_type:ty) => (
        impl ToPyObject for $rust_type {
            fn to_object(&self, py: Python) -> PyObject {
                unsafe {
                    PyObject::from_owned_ptr_or_panic(py, ffi::PyInt_FromLong(*self as c_long))
                }
            }
        }
        impl IntoPyObject for $rust_type {
            fn into_object(self, py: Python) -> PyObject {
                unsafe {
                    PyObject::from_owned_ptr_or_panic(py, ffi::PyInt_FromLong(self as c_long))
                }
            }
        }

        impl<'source> FromPyObject<'source> for $rust_type {
            fn extract(obj: &'source PyObjectRef) -> PyResult<Self> {
                let val = unsafe { ffi::PyLong_AsLong(obj.as_ptr()) };
                if val == -1 && PyErr::occurred(obj.py()) {
                    return Err(PyErr::fetch(obj.py()));
                }
                match cast::<c_long, $rust_type>(val) {
                    Some(v) => Ok(v),
                    None => Err(exceptions::OverflowError.into())
                }
            }
        }
    )
);

macro_rules! int_convert_u64_or_i64 (
    ($rust_type:ty, $pylong_from_ll_or_ull:expr, $pylong_as_ull_or_ull:expr) => (
        impl ToPyObject for $rust_type {
            fn to_object(&self, py: Python) -> PyObject {
                unsafe {
                    let ptr = match cast::<$rust_type, c_long>(*self) {
                        Some(v) => ffi::PyInt_FromLong(v),
                        None => $pylong_from_ll_or_ull(*self)
                    };
                    PyObject::from_owned_ptr_or_panic(py, ptr)
                }
            }
        }
        impl IntoPyObject for $rust_type {
            fn into_object(self, py: Python) -> PyObject {
                unsafe {
                    let ptr = match cast::<$rust_type, c_long>(self) {
                        Some(v) => ffi::PyInt_FromLong(v),
                        None => $pylong_from_ll_or_ull(self)
                    };
                    PyObject::from_owned_ptr_or_panic(py, ptr)
                }
            }
        }

        impl <'source> FromPyObject<'source> for $rust_type {
            fn extract(obj: &'source PyObjectRef) -> PyResult<$rust_type>
            {
                let ptr = obj.as_ptr();
                unsafe {
                    if ffi::PyLong_Check(ptr) != 0 {
                        err_if_invalid_value(obj.py(), !0, $pylong_as_ull_or_ull(ptr))
                    } else if ffi::PyInt_Check(ptr) != 0 {
                        match cast::<c_long, $rust_type>(ffi::PyInt_AS_LONG(ptr)) {
                            Some(v) => Ok(v),
                            None => Err(exceptions::OverflowError.into())
                        }
                    } else {
                        let num = PyObject::from_owned_ptr_or_err(
                            obj.py(), ffi::PyNumber_Long(ptr))?;
                        err_if_invalid_value(
                            obj.py(), !0, $pylong_as_ull_or_ull(num.into_ptr()))
                    }
                }
            }
        }
    )
);

int_fits_c_long!(i8);
int_fits_c_long!(u8);
int_fits_c_long!(i16);
int_fits_c_long!(u16);
int_fits_c_long!(i32);

// If c_long is 64-bits, we can use more types with int_fits_c_long!:
#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
int_fits_c_long!(u32);
#[cfg(any(target_pointer_width = "32", target_os = "windows"))]
int_fits_larger_int!(u32, u64);

#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
int_fits_c_long!(i64);

// manual implementation for i64 on systems with 32-bit long
#[cfg(any(target_pointer_width = "32", target_os = "windows"))]
int_convert_u64_or_i64!(i64, ffi::PyLong_FromLongLong, ffi::PyLong_AsLongLong);

#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
int_fits_c_long!(isize);
#[cfg(any(target_pointer_width = "32", target_os = "windows"))]
int_fits_larger_int!(isize, i64);

int_fits_larger_int!(usize, u64);

// u64 has a manual implementation as it never fits into signed long
int_convert_u64_or_i64!(
    u64,
    ffi::PyLong_FromUnsignedLongLong,
    ffi::PyLong_AsUnsignedLongLong
);

int_convert_bignum!(i128, 16, IS_LITTLE_ENDIAN, 1);
int_convert_bignum!(u128, 16, IS_LITTLE_ENDIAN, 0);

#[cfg(test)]
mod test {
    use conversion::ToPyObject;
    use python::Python;

    macro_rules! num_to_py_object_and_back (
        ($func_name:ident, $t1:ty, $t2:ty) => (
            #[test]
            fn $func_name() {
                let gil = Python::acquire_gil();
                let py = gil.python();
                let val = 123 as $t1;
                let obj = val.to_object(py);
                assert_eq!(obj.extract::<$t2>(py).unwrap(), val as $t2);
            }
        )
    );

    num_to_py_object_and_back!(to_from_f64, f64, f64);
    num_to_py_object_and_back!(to_from_f32, f32, f32);
    num_to_py_object_and_back!(to_from_i8, i8, i8);
    num_to_py_object_and_back!(to_from_u8, u8, u8);
    num_to_py_object_and_back!(to_from_i16, i16, i16);
    num_to_py_object_and_back!(to_from_u16, u16, u16);
    num_to_py_object_and_back!(to_from_i32, i32, i32);
    num_to_py_object_and_back!(to_from_u32, u32, u32);
    num_to_py_object_and_back!(to_from_i64, i64, i64);
    num_to_py_object_and_back!(to_from_u64, u64, u64);
    num_to_py_object_and_back!(to_from_isize, isize, isize);
    num_to_py_object_and_back!(to_from_usize, usize, usize);
    num_to_py_object_and_back!(to_from_i128, i128, i128);
    num_to_py_object_and_back!(to_from_u128, u128, u128);
    num_to_py_object_and_back!(float_to_i32, f64, i32);
    num_to_py_object_and_back!(float_to_u32, f64, u32);
    num_to_py_object_and_back!(float_to_i64, f64, i64);
    num_to_py_object_and_back!(float_to_u64, f64, u64);
    num_to_py_object_and_back!(int_to_float, i32, f64);
}
