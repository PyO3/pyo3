// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::{
    exceptions, ffi, AsPyPointer, FromPyObject, IntoPy, PyAny, PyErr, PyObject, PyResult, Python,
    ToPyObject,
};
use std::convert::TryFrom;
use std::i64;
use std::os::raw::c_long;

macro_rules! int_fits_larger_int {
    ($rust_type:ty, $larger_type:ty) => {
        impl ToPyObject for $rust_type {
            #[inline]
            fn to_object(&self, py: Python) -> PyObject {
                (*self as $larger_type).into_py(py)
            }
        }
        impl IntoPy<PyObject> for $rust_type {
            fn into_py(self, py: Python) -> PyObject {
                (self as $larger_type).into_py(py)
            }
        }

        impl<'source> FromPyObject<'source> for $rust_type {
            fn extract(obj: &'source PyAny) -> PyResult<Self> {
                let val: $larger_type = obj.extract()?;
                <$rust_type>::try_from(val)
                    .map_err(|e| exceptions::PyOverflowError::new_err(e.to_string()))
            }
        }
    };
}

/// Represents a Python `int` object.
///
/// You can usually avoid directly working with this type
/// by using [`ToPyObject`](trait.ToPyObject.html)
/// and [extract](struct.PyAny.html#method.extract)
/// with the primitive Rust integer types.
#[repr(transparent)]
pub struct PyLong(PyAny);

pyobject_native_type_core!(PyLong, ffi::PyLong_Type, #checkfunction=ffi::PyLong_Check);

macro_rules! int_fits_c_long {
    ($rust_type:ty) => {
        impl ToPyObject for $rust_type {
            fn to_object(&self, py: Python) -> PyObject {
                unsafe { PyObject::from_owned_ptr(py, ffi::PyLong_FromLong(*self as c_long)) }
            }
        }
        impl IntoPy<PyObject> for $rust_type {
            fn into_py(self, py: Python) -> PyObject {
                unsafe { PyObject::from_owned_ptr(py, ffi::PyLong_FromLong(self as c_long)) }
            }
        }

        impl<'source> FromPyObject<'source> for $rust_type {
            fn extract(obj: &'source PyAny) -> PyResult<Self> {
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
                <$rust_type>::try_from(val)
                    .map_err(|e| exceptions::PyOverflowError::new_err(e.to_string()))
            }
        }
    };
}

macro_rules! int_convert_u64_or_i64 {
    ($rust_type:ty, $pylong_from_ll_or_ull:expr, $pylong_as_ll_or_ull:expr) => {
        impl ToPyObject for $rust_type {
            #[inline]
            fn to_object(&self, py: Python) -> PyObject {
                unsafe { PyObject::from_owned_ptr(py, $pylong_from_ll_or_ull(*self)) }
            }
        }
        impl IntoPy<PyObject> for $rust_type {
            #[inline]
            fn into_py(self, py: Python) -> PyObject {
                unsafe { PyObject::from_owned_ptr(py, $pylong_from_ll_or_ull(self)) }
            }
        }
        impl<'source> FromPyObject<'source> for $rust_type {
            fn extract(ob: &'source PyAny) -> PyResult<$rust_type> {
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
    };
}

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

#[cfg(not(Py_LIMITED_API))]
mod fast_128bit_int_conversion {
    use super::*;

    // for 128bit Integers
    macro_rules! int_convert_128 {
        ($rust_type: ty, $is_signed: expr) => {
            impl ToPyObject for $rust_type {
                #[inline]
                fn to_object(&self, py: Python) -> PyObject {
                    (*self).into_py(py)
                }
            }
            impl IntoPy<PyObject> for $rust_type {
                fn into_py(self, py: Python) -> PyObject {
                    unsafe {
                        // Always use little endian
                        let bytes = self.to_le_bytes();
                        let obj = ffi::_PyLong_FromByteArray(
                            bytes.as_ptr() as *const std::os::raw::c_uchar,
                            bytes.len(),
                            1,
                            $is_signed,
                        );
                        PyObject::from_owned_ptr(py, obj)
                    }
                }
            }

            impl<'source> FromPyObject<'source> for $rust_type {
                fn extract(ob: &'source PyAny) -> PyResult<$rust_type> {
                    unsafe {
                        let num = ffi::PyNumber_Index(ob.as_ptr());
                        if num.is_null() {
                            return Err(PyErr::fetch(ob.py()));
                        }
                        let mut buffer = [0; std::mem::size_of::<$rust_type>()];
                        let ok = ffi::_PyLong_AsByteArray(
                            num as *mut ffi::PyLongObject,
                            buffer.as_mut_ptr(),
                            buffer.len(),
                            1,
                            $is_signed,
                        );
                        ffi::Py_DECREF(num);
                        crate::err::error_on_minusone(ob.py(), ok)?;
                        Ok(<$rust_type>::from_le_bytes(buffer))
                    }
                }
            }
        };
    }

    int_convert_128!(i128, 1);
    int_convert_128!(u128, 0);
}

// For ABI3 we implement the conversion manually.
#[cfg(Py_LIMITED_API)]
mod slow_128bit_int_conversion {
    use super::*;
    const SHIFT: usize = 64;

    // for 128bit Integers
    macro_rules! int_convert_128 {
        ($rust_type: ty, $half_type: ty) => {
            impl ToPyObject for $rust_type {
                #[inline]
                fn to_object(&self, py: Python) -> PyObject {
                    (*self).into_py(py)
                }
            }

            impl IntoPy<PyObject> for $rust_type {
                fn into_py(self, py: Python) -> PyObject {
                    let lower = self as u64;
                    let upper = (self >> SHIFT) as $half_type;
                    unsafe {
                        let shifted = PyObject::from_owned_ptr(
                            py,
                            ffi::PyNumber_Lshift(
                                upper.into_py(py).as_ptr(),
                                SHIFT.into_py(py).as_ptr(),
                            ),
                        );
                        PyObject::from_owned_ptr(
                            py,
                            ffi::PyNumber_Or(shifted.as_ptr(), lower.into_py(py).as_ptr()),
                        )
                    }
                }
            }

            impl<'source> FromPyObject<'source> for $rust_type {
                fn extract(ob: &'source PyAny) -> PyResult<$rust_type> {
                    let py = ob.py();
                    unsafe {
                        let lower = err_if_invalid_value(
                            py,
                            -1 as _,
                            ffi::PyLong_AsUnsignedLongLongMask(ob.as_ptr()),
                        )? as $rust_type;
                        let shifted = PyObject::from_owned_ptr_or_err(
                            py,
                            ffi::PyNumber_Rshift(ob.as_ptr(), SHIFT.into_py(py).as_ptr()),
                        )?;
                        let upper: $half_type = shifted.extract(py)?;
                        Ok((<$rust_type>::from(upper) << SHIFT) | lower)
                    }
                }
            }
        };
    }

    int_convert_128!(i128, i64);
    int_convert_128!(u128, u64);
}

fn err_if_invalid_value<T: PartialEq>(
    py: Python,
    invalid_value: T,
    actual_value: T,
) -> PyResult<T> {
    if actual_value == invalid_value {
        if let Some(err) = PyErr::take(py) {
            return Err(err);
        }
    }

    Ok(actual_value)
}

#[cfg(test)]
mod test_128bit_intergers {
    use super::*;
    use crate::types::PyDict;

    #[cfg(not(target_arch = "wasm32"))]
    use proptest::prelude::*;

    #[cfg(not(target_arch = "wasm32"))]
    proptest! {
        #[test]
        fn test_i128_roundtrip(x: i128) {
            Python::with_gil(|py| {
                let x_py = x.into_py(py);
                let locals = PyDict::new(py);
                locals.set_item("x_py", x_py.clone_ref(py)).unwrap();
                py.run(&format!("assert x_py == {}", x), None, Some(locals)).unwrap();
                let roundtripped: i128 = x_py.extract(py).unwrap();
                assert_eq!(x, roundtripped);
            })
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    proptest! {
        #[test]
        fn test_u128_roundtrip(x: u128) {
            Python::with_gil(|py| {
                let x_py = x.into_py(py);
                let locals = PyDict::new(py);
                locals.set_item("x_py", x_py.clone_ref(py)).unwrap();
                py.run(&format!("assert x_py == {}", x), None, Some(locals)).unwrap();
                let roundtripped: u128 = x_py.extract(py).unwrap();
                assert_eq!(x, roundtripped);
            })
        }
    }

    #[test]
    fn test_i128_max() {
        Python::with_gil(|py| {
            let v = std::i128::MAX;
            let obj = v.to_object(py);
            assert_eq!(v, obj.extract::<i128>(py).unwrap());
            assert_eq!(v as u128, obj.extract::<u128>(py).unwrap());
            assert!(obj.extract::<u64>(py).is_err());
        })
    }

    #[test]
    fn test_i128_min() {
        Python::with_gil(|py| {
            let v = std::i128::MIN;
            let obj = v.to_object(py);
            assert_eq!(v, obj.extract::<i128>(py).unwrap());
            assert!(obj.extract::<i64>(py).is_err());
            assert!(obj.extract::<u128>(py).is_err());
        })
    }

    #[test]
    fn test_u128_max() {
        Python::with_gil(|py| {
            let v = std::u128::MAX;
            let obj = v.to_object(py);
            assert_eq!(v, obj.extract::<u128>(py).unwrap());
            assert!(obj.extract::<i128>(py).is_err());
        })
    }

    #[test]
    fn test_i128_overflow() {
        Python::with_gil(|py| {
            let obj = py.eval("(1 << 130) * -1", None, None).unwrap();
            let err = obj.extract::<i128>().unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyOverflowError>(py));
        })
    }

    #[test]
    fn test_u128_overflow() {
        Python::with_gil(|py| {
            let obj = py.eval("1 << 130", None, None).unwrap();
            let err = obj.extract::<u128>().unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyOverflowError>(py));
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Python;
    use crate::ToPyObject;

    #[test]
    fn test_u32_max() {
        Python::with_gil(|py| {
            let v = std::u32::MAX;
            let obj = v.to_object(py);
            assert_eq!(v, obj.extract::<u32>(py).unwrap());
            assert_eq!(u64::from(v), obj.extract::<u64>(py).unwrap());
            assert!(obj.extract::<i32>(py).is_err());
        });
    }

    #[test]
    fn test_i64_max() {
        Python::with_gil(|py| {
            let v = std::i64::MAX;
            let obj = v.to_object(py);
            assert_eq!(v, obj.extract::<i64>(py).unwrap());
            assert_eq!(v as u64, obj.extract::<u64>(py).unwrap());
            assert!(obj.extract::<u32>(py).is_err());
        });
    }

    #[test]
    fn test_i64_min() {
        Python::with_gil(|py| {
            let v = std::i64::MIN;
            let obj = v.to_object(py);
            assert_eq!(v, obj.extract::<i64>(py).unwrap());
            assert!(obj.extract::<i32>(py).is_err());
            assert!(obj.extract::<u64>(py).is_err());
        });
    }

    #[test]
    fn test_u64_max() {
        Python::with_gil(|py| {
            let v = std::u64::MAX;
            let obj = v.to_object(py);
            assert_eq!(v, obj.extract::<u64>(py).unwrap());
            assert!(obj.extract::<i64>(py).is_err());
        });
    }

    macro_rules! test_common (
        ($test_mod_name:ident, $t:ty) => (
            mod $test_mod_name {
                use crate::exceptions;
                use crate::ToPyObject;
                use crate::Python;

                #[test]
                fn from_py_string_type_error() {
                    Python::with_gil(|py|{


                    let obj = ("123").to_object(py);
                    let err = obj.extract::<$t>(py).unwrap_err();
                    assert!(err.is_instance_of::<exceptions::PyTypeError>(py));
                    });
                }

                #[test]
                fn from_py_float_type_error() {
                    Python::with_gil(|py|{

                    let obj = (12.3).to_object(py);
                    let err = obj.extract::<$t>(py).unwrap_err();
                    assert!(err.is_instance_of::<exceptions::PyTypeError>(py));});
                }

                #[test]
                fn to_py_object_and_back() {
                    Python::with_gil(|py|{

                    let val = 123 as $t;
                    let obj = val.to_object(py);
                    assert_eq!(obj.extract::<$t>(py).unwrap(), val as $t);});
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
    test_common!(i128, i128);
    test_common!(u128, u128);
}
