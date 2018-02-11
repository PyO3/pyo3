// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::os::raw::c_long;

extern crate num_traits;
use self::num_traits::cast::cast;

use ffi;
use object::PyObject;
use python::{ToPyPointer, Python};
use err::{PyResult, PyErr};
use objects::{exc, PyObjectRef};
use instance::PyObjectWithToken;
use conversion::{ToPyObject, IntoPyObject, FromPyObject};

/// Represents a Python `int` object.
///
/// You can usually avoid directly working with this type
/// by using [`ToPyObject`](trait.ToPyObject.html)
/// and [extract](struct.PyObject.html#method.extract)
/// with the primitive Rust integer types.
pub struct PyLong(PyObject);

pyobject_convert!(PyLong);
pyobject_nativetype!(PyLong, PyLong_Type, PyLong_Check);


macro_rules! int_fits_c_long(
    ($rust_type:ty) => (
        impl ToPyObject for $rust_type {
            #[cfg_attr(feature = "cargo-clippy", allow(cast_lossless))]
            fn to_object(&self, py: Python) -> PyObject {
                unsafe {
                    PyObject::from_owned_ptr_or_panic(py, ffi::PyLong_FromLong(*self as c_long))
                }
            }
        }
        impl IntoPyObject for $rust_type {
            #[cfg_attr(feature = "cargo-clippy", allow(cast_lossless))]
            fn into_object(self, py: Python) -> PyObject {
                unsafe {
                    PyObject::from_owned_ptr_or_panic(py, ffi::PyLong_FromLong(self as c_long))
                }
            }
        }
        pyobject_extract!(obj to $rust_type => {
            let ptr = obj.as_ptr();
            let val = unsafe {
                let num = ffi::PyNumber_Index(ptr);
                if num.is_null() {
                    Err(PyErr::fetch(obj.py()))
                } else {
                    let val = err_if_invalid_value(obj.py(), -1, ffi::PyLong_AsLong(num));
                    ffi::Py_DECREF(num);
                    val
                }
            }?;
            match cast::<c_long, $rust_type>(val) {
                Some(v) => Ok(v),
                None => Err(exc::OverflowError.into())
            }
        });
    )
);


macro_rules! int_fits_larger_int(
    ($rust_type:ty, $larger_type:ty) => (
        impl ToPyObject for $rust_type {
            #[inline]
            fn to_object(&self, py: Python) -> PyObject {
                (*self as $larger_type).into_object(py)
            }
        }
        impl IntoPyObject for $rust_type {
            fn into_object(self, py: Python) -> PyObject {
                (self as $larger_type).into_object(py)
            }
        }
        pyobject_extract!(obj to $rust_type => {
            let val = try!(obj.extract::<$larger_type>());
            match cast::<$larger_type, $rust_type>(val) {
                Some(v) => Ok(v),
                None => Err(exc::OverflowError.into())
            }
        });
    )
);



#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn err_if_invalid_value<T: PartialEq>
    (py: Python, invalid_value: T, actual_value: T) -> PyResult<T>
{
    if actual_value == invalid_value && PyErr::occurred(py) {
        Err(PyErr::fetch(py))
    } else {
        Ok(actual_value)
    }
}

macro_rules! int_convert_u64_or_i64 (
    ($rust_type:ty, $pylong_from_ll_or_ull:expr, $pylong_as_ll_or_ull:expr) => (
        impl ToPyObject for $rust_type {
            #[inline]
            fn to_object(&self, py: Python) -> PyObject {
                unsafe {
                    PyObject::from_owned_ptr_or_panic(py, $pylong_from_ll_or_ull(*self))
                }
            }
        }
        impl IntoPyObject for $rust_type {
            #[inline]
            fn into_object(self, py: Python) -> PyObject {
                unsafe {
                    PyObject::from_owned_ptr_or_panic(py, $pylong_from_ll_or_ull(self))
                }
            }
        }
        impl<'source> FromPyObject<'source> for $rust_type {
            fn extract(ob: &'source PyObjectRef) -> PyResult<$rust_type>
            {
                let ptr = ob.as_ptr();
                unsafe {
                    let num = ffi::PyNumber_Index(ptr);
                    if num.is_null() {
                        Err(PyErr::fetch(ob.py()))
                    } else {
                        let result = err_if_invalid_value(ob.py(), !0, $pylong_as_ll_or_ull(num));
                        ffi::Py_DECREF(num);
                        result
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
#[cfg(all(target_pointer_width="64", not(target_os="windows")))]
int_fits_c_long!(u32);
#[cfg(any(target_pointer_width="32", target_os="windows"))]
int_fits_larger_int!(u32, u64);

#[cfg(all(target_pointer_width="64", not(target_os="windows")))]
int_fits_c_long!(i64);

// manual implementation for i64 on systems with 32-bit long
#[cfg(any(target_pointer_width="32", target_os="windows"))]
int_convert_u64_or_i64!(i64, ffi::PyLong_FromLongLong, ffi::PyLong_AsLongLong);

#[cfg(all(target_pointer_width="64", not(target_os="windows")))]
int_fits_c_long!(isize);
#[cfg(any(target_pointer_width="32", target_os="windows"))]
int_fits_larger_int!(isize, i64);

int_fits_larger_int!(usize, u64);

// u64 has a manual implementation as it never fits into signed long
int_convert_u64_or_i64!(u64, ffi::PyLong_FromUnsignedLongLong, ffi::PyLong_AsUnsignedLongLong);


#[cfg(test)]
mod test {
    use std;
    use python::Python;
    use conversion::ToPyObject;

    macro_rules! test_common (
        ($test_mod_name:ident, $t:ty) => (
            mod $test_mod_name {
                use super::*;
                use objects::exc;

                #[test]
                fn from_py_string_type_error() {
                    let gil = Python::acquire_gil();
                    let py = gil.python();

                    let obj = ("123").to_object(py);
                    let err = obj.extract::<$t>(py).unwrap_err();
                    assert!(err.is_instance::<exc::TypeError>(py));
                }

                #[test]
                fn from_py_float_type_error() {
                    let gil = Python::acquire_gil();
                    let py = gil.python();

                    let obj = (12.3).to_object(py);
                    let err = obj.extract::<$t>(py).unwrap_err();
                    assert!(err.is_instance::<exc::TypeError>(py));
                }

                #[test]
                fn to_py_object_and_back() {
                    let gil = Python::acquire_gil();
                    let py = gil.python();

                    let val = 123 as $t;
                    let obj = val.to_object(py);
                    assert_eq!(obj.extract::<$t>(py).unwrap(), val as $t);
                }
            }
        )
    );

    test_common!(i8, i8);
    test_common!(u8, u8);
    test_common!(i16, i16);
    test_common!(u16, u16);
    test_common!(i32, i32);
    test_common!(u32, u32);
    test_common!(i64, i64);
    test_common!(u64, u64);
    test_common!(isize, isize);
    test_common!(usize, usize);

    #[test]
    fn test_u32_max() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = std::u32::MAX;
        let obj = v.to_object(py);
        assert_eq!(v, obj.extract::<u32>(py).unwrap());
        assert_eq!(v as u64, obj.extract::<u64>(py).unwrap());
        assert!(obj.extract::<i32>(py).is_err());
    }

    #[test]
    fn test_i64_max() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = std::i64::MAX;
        let obj = v.to_object(py);
        assert_eq!(v, obj.extract::<i64>(py).unwrap());
        assert_eq!(v as u64, obj.extract::<u64>(py).unwrap());
        assert!(obj.extract::<u32>(py).is_err());
    }

    #[test]
    fn test_i64_min() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = std::i64::MIN;
        let obj = v.to_object(py);
        assert_eq!(v, obj.extract::<i64>(py).unwrap());
        assert!(obj.extract::<i32>(py).is_err());
        assert!(obj.extract::<u64>(py).is_err());
    }

    #[test]
    fn test_u64_max() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = std::u64::MAX;
        let obj = v.to_object(py);
        assert_eq!(v, obj.extract::<u64>(py).unwrap());
        assert!(obj.extract::<i64>(py).is_err());
    }
}
