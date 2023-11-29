#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::types::any::PyAnyMethods;
use crate::{
    exceptions, ffi, Bound, FromPyObject, IntoPy, PyAny, PyErr, PyObject, PyResult, Python,
    ToPyObject,
};
use std::convert::TryFrom;
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};
use std::os::raw::c_long;

macro_rules! int_fits_larger_int {
    ($rust_type:ty, $larger_type:ty) => {
        impl ToPyObject for $rust_type {
            #[inline]
            fn to_object(&self, py: Python<'_>) -> PyObject {
                (*self as $larger_type).into_py(py)
            }
        }
        impl IntoPy<PyObject> for $rust_type {
            fn into_py(self, py: Python<'_>) -> PyObject {
                (self as $larger_type).into_py(py)
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_output() -> TypeInfo {
                <$larger_type>::type_output()
            }
        }

        impl FromPyObject<'_> for $rust_type {
            fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
                let val: $larger_type = obj.extract()?;
                <$rust_type>::try_from(val)
                    .map_err(|e| exceptions::PyOverflowError::new_err(e.to_string()))
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_input() -> TypeInfo {
                <$larger_type>::type_input()
            }
        }
    };
}

macro_rules! extract_int {
    ($obj:ident, $error_val:expr, $pylong_as:expr) => {
        extract_int!($obj, $error_val, $pylong_as, false)
    };

    ($obj:ident, $error_val:expr, $pylong_as:expr, $force_index_call: literal) => {
        // In python 3.8+ `PyLong_AsLong` and friends takes care of calling `PyNumber_Index`,
        // however 3.8 & 3.9 do lossy conversion of floats, hence we only use the
        // simplest logic for 3.10+ where that was fixed - python/cpython#82180.
        // `PyLong_AsUnsignedLongLong` does not call `PyNumber_Index`, hence the `force_index_call` argument
        // See https://github.com/PyO3/pyo3/pull/3742 for detials
        if cfg!(Py_3_10) && !$force_index_call {
            err_if_invalid_value($obj.py(), $error_val, unsafe { $pylong_as($obj.as_ptr()) })
        } else if let Ok(long) = $obj.downcast::<crate::types::PyLong>() {
            // fast path - checking for subclass of `int` just checks a bit in the type $object
            err_if_invalid_value($obj.py(), $error_val, unsafe { $pylong_as(long.as_ptr()) })
        } else {
            unsafe {
                let num = ffi::PyNumber_Index($obj.as_ptr());
                if num.is_null() {
                    Err(PyErr::fetch($obj.py()))
                } else {
                    let result = err_if_invalid_value($obj.py(), $error_val, $pylong_as(num));
                    ffi::Py_DECREF(num);
                    result
                }
            }
        }
    };
}

macro_rules! int_convert_u64_or_i64 {
    ($rust_type:ty, $pylong_from_ll_or_ull:expr, $pylong_as_ll_or_ull:expr, $force_index_call:literal) => {
        impl ToPyObject for $rust_type {
            #[inline]
            fn to_object(&self, py: Python<'_>) -> PyObject {
                unsafe { PyObject::from_owned_ptr(py, $pylong_from_ll_or_ull(*self)) }
            }
        }
        impl IntoPy<PyObject> for $rust_type {
            #[inline]
            fn into_py(self, py: Python<'_>) -> PyObject {
                unsafe { PyObject::from_owned_ptr(py, $pylong_from_ll_or_ull(self)) }
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_output() -> TypeInfo {
                TypeInfo::builtin("int")
            }
        }
        impl FromPyObject<'_> for $rust_type {
            fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<$rust_type> {
                extract_int!(obj, !0, $pylong_as_ll_or_ull, $force_index_call)
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_input() -> TypeInfo {
                Self::type_output()
            }
        }
    };
}

macro_rules! int_fits_c_long {
    ($rust_type:ty) => {
        impl ToPyObject for $rust_type {
            fn to_object(&self, py: Python<'_>) -> PyObject {
                unsafe { PyObject::from_owned_ptr(py, ffi::PyLong_FromLong(*self as c_long)) }
            }
        }
        impl IntoPy<PyObject> for $rust_type {
            fn into_py(self, py: Python<'_>) -> PyObject {
                unsafe { PyObject::from_owned_ptr(py, ffi::PyLong_FromLong(self as c_long)) }
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_output() -> TypeInfo {
                TypeInfo::builtin("int")
            }
        }

        impl<'source> FromPyObject<'source> for $rust_type {
            fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
                let val: c_long = extract_int!(obj, -1, ffi::PyLong_AsLong)?;
                <$rust_type>::try_from(val)
                    .map_err(|e| exceptions::PyOverflowError::new_err(e.to_string()))
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_input() -> TypeInfo {
                Self::type_output()
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
int_convert_u64_or_i64!(i64, ffi::PyLong_FromLongLong, ffi::PyLong_AsLongLong, false);

#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
int_fits_c_long!(isize);
#[cfg(any(target_pointer_width = "32", target_os = "windows"))]
int_fits_larger_int!(isize, i64);

int_fits_larger_int!(usize, u64);

// u64 has a manual implementation as it never fits into signed long
int_convert_u64_or_i64!(
    u64,
    ffi::PyLong_FromUnsignedLongLong,
    ffi::PyLong_AsUnsignedLongLong,
    true
);

#[cfg(not(Py_LIMITED_API))]
mod fast_128bit_int_conversion {
    use super::*;

    // for 128bit Integers
    macro_rules! int_convert_128 {
        ($rust_type: ty, $is_signed: expr) => {
            impl ToPyObject for $rust_type {
                #[inline]
                fn to_object(&self, py: Python<'_>) -> PyObject {
                    (*self).into_py(py)
                }
            }
            impl IntoPy<PyObject> for $rust_type {
                fn into_py(self, py: Python<'_>) -> PyObject {
                    // Always use little endian
                    let bytes = self.to_le_bytes();
                    unsafe {
                        PyObject::from_owned_ptr(
                            py,
                            ffi::_PyLong_FromByteArray(
                                bytes.as_ptr() as *const std::os::raw::c_uchar,
                                bytes.len(),
                                1,
                                $is_signed,
                            ),
                        )
                    }
                }

                #[cfg(feature = "experimental-inspect")]
                fn type_output() -> TypeInfo {
                    TypeInfo::builtin("int")
                }
            }

            impl FromPyObject<'_> for $rust_type {
                fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<$rust_type> {
                    let num = unsafe {
                        PyObject::from_owned_ptr_or_err(ob.py(), ffi::PyNumber_Index(ob.as_ptr()))?
                    };
                    let mut buffer = [0; std::mem::size_of::<$rust_type>()];
                    crate::err::error_on_minusone(ob.py(), unsafe {
                        ffi::_PyLong_AsByteArray(
                            num.as_ptr() as *mut ffi::PyLongObject,
                            buffer.as_mut_ptr(),
                            buffer.len(),
                            1,
                            $is_signed,
                        )
                    })?;
                    Ok(<$rust_type>::from_le_bytes(buffer))
                }

                #[cfg(feature = "experimental-inspect")]
                fn type_input() -> TypeInfo {
                    Self::type_output()
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
                fn to_object(&self, py: Python<'_>) -> PyObject {
                    (*self).into_py(py)
                }
            }

            impl IntoPy<PyObject> for $rust_type {
                fn into_py(self, py: Python<'_>) -> PyObject {
                    let lower = (self as u64).into_py(py);
                    let upper = ((self >> SHIFT) as $half_type).into_py(py);
                    let shift = SHIFT.into_py(py);
                    unsafe {
                        let shifted = PyObject::from_owned_ptr(
                            py,
                            ffi::PyNumber_Lshift(upper.as_ptr(), shift.as_ptr()),
                        );
                        PyObject::from_owned_ptr(
                            py,
                            ffi::PyNumber_Or(shifted.as_ptr(), lower.as_ptr()),
                        )
                    }
                }

                #[cfg(feature = "experimental-inspect")]
                fn type_output() -> TypeInfo {
                    TypeInfo::builtin("int")
                }
            }

            impl FromPyObject<'_> for $rust_type {
                fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<$rust_type> {
                    let py = ob.py();
                    unsafe {
                        let lower = err_if_invalid_value(
                            py,
                            -1 as _,
                            ffi::PyLong_AsUnsignedLongLongMask(ob.as_ptr()),
                        )? as $rust_type;
                        let shift = SHIFT.into_py(py);
                        let shifted = PyObject::from_owned_ptr_or_err(
                            py,
                            ffi::PyNumber_Rshift(ob.as_ptr(), shift.as_ptr()),
                        )?;
                        let upper: $half_type = shifted.extract(py)?;
                        Ok((<$rust_type>::from(upper) << SHIFT) | lower)
                    }
                }

                #[cfg(feature = "experimental-inspect")]
                fn type_input() -> TypeInfo {
                    Self::type_output()
                }
            }
        };
    }

    int_convert_128!(i128, i64);
    int_convert_128!(u128, u64);
}

fn err_if_invalid_value<T: PartialEq>(
    py: Python<'_>,
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

macro_rules! nonzero_int_impl {
    ($nonzero_type:ty, $primitive_type:ty) => {
        impl ToPyObject for $nonzero_type {
            fn to_object(&self, py: Python<'_>) -> PyObject {
                self.get().to_object(py)
            }
        }

        impl IntoPy<PyObject> for $nonzero_type {
            fn into_py(self, py: Python<'_>) -> PyObject {
                self.get().into_py(py)
            }
        }

        impl FromPyObject<'_> for $nonzero_type {
            fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
                let val: $primitive_type = obj.extract()?;
                <$nonzero_type>::try_from(val)
                    .map_err(|_| exceptions::PyValueError::new_err("invalid zero value"))
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_input() -> TypeInfo {
                <$primitive_type>::type_input()
            }
        }
    };
}

nonzero_int_impl!(NonZeroI8, i8);
nonzero_int_impl!(NonZeroI16, i16);
nonzero_int_impl!(NonZeroI32, i32);
nonzero_int_impl!(NonZeroI64, i64);
nonzero_int_impl!(NonZeroI128, i128);
nonzero_int_impl!(NonZeroIsize, isize);
nonzero_int_impl!(NonZeroU8, u8);
nonzero_int_impl!(NonZeroU16, u16);
nonzero_int_impl!(NonZeroU32, u32);
nonzero_int_impl!(NonZeroU64, u64);
nonzero_int_impl!(NonZeroU128, u128);
nonzero_int_impl!(NonZeroUsize, usize);

#[cfg(test)]
mod test_128bit_integers {
    use super::*;
    #[cfg(not(target_arch = "wasm32"))]
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

        #[test]
        fn test_nonzero_i128_roundtrip(
            x in any::<i128>()
                .prop_filter("Values must not be 0", |x| x != &0)
                .prop_map(|x| NonZeroI128::new(x).unwrap())
        ) {
            Python::with_gil(|py| {
                let x_py = x.into_py(py);
                let locals = PyDict::new(py);
                locals.set_item("x_py", x_py.clone_ref(py)).unwrap();
                py.run(&format!("assert x_py == {}", x), None, Some(locals)).unwrap();
                let roundtripped: NonZeroI128 = x_py.extract(py).unwrap();
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

        #[test]
        fn test_nonzero_u128_roundtrip(
            x in any::<u128>()
                .prop_filter("Values must not be 0", |x| x != &0)
                .prop_map(|x| NonZeroU128::new(x).unwrap())
        ) {
            Python::with_gil(|py| {
                let x_py = x.into_py(py);
                let locals = PyDict::new(py);
                locals.set_item("x_py", x_py.clone_ref(py)).unwrap();
                py.run(&format!("assert x_py == {}", x), None, Some(locals)).unwrap();
                let roundtripped: NonZeroU128 = x_py.extract(py).unwrap();
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

    #[test]
    fn test_nonzero_i128_max() {
        Python::with_gil(|py| {
            let v = NonZeroI128::new(std::i128::MAX).unwrap();
            let obj = v.to_object(py);
            assert_eq!(v, obj.extract::<NonZeroI128>(py).unwrap());
            assert_eq!(
                NonZeroU128::new(v.get() as u128).unwrap(),
                obj.extract::<NonZeroU128>(py).unwrap()
            );
            assert!(obj.extract::<NonZeroU64>(py).is_err());
        })
    }

    #[test]
    fn test_nonzero_i128_min() {
        Python::with_gil(|py| {
            let v = NonZeroI128::new(std::i128::MIN).unwrap();
            let obj = v.to_object(py);
            assert_eq!(v, obj.extract::<NonZeroI128>(py).unwrap());
            assert!(obj.extract::<NonZeroI64>(py).is_err());
            assert!(obj.extract::<NonZeroU128>(py).is_err());
        })
    }

    #[test]
    fn test_nonzero_u128_max() {
        Python::with_gil(|py| {
            let v = NonZeroU128::new(std::u128::MAX).unwrap();
            let obj = v.to_object(py);
            assert_eq!(v, obj.extract::<NonZeroU128>(py).unwrap());
            assert!(obj.extract::<NonZeroI128>(py).is_err());
        })
    }

    #[test]
    fn test_nonzero_i128_overflow() {
        Python::with_gil(|py| {
            let obj = py.eval("(1 << 130) * -1", None, None).unwrap();
            let err = obj.extract::<NonZeroI128>().unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyOverflowError>(py));
        })
    }

    #[test]
    fn test_nonzero_u128_overflow() {
        Python::with_gil(|py| {
            let obj = py.eval("1 << 130", None, None).unwrap();
            let err = obj.extract::<NonZeroU128>().unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyOverflowError>(py));
        })
    }

    #[test]
    fn test_nonzero_i128_zero_value() {
        Python::with_gil(|py| {
            let obj = py.eval("0", None, None).unwrap();
            let err = obj.extract::<NonZeroI128>().unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyValueError>(py));
        })
    }

    #[test]
    fn test_nonzero_u128_zero_value() {
        Python::with_gil(|py| {
            let obj = py.eval("0", None, None).unwrap();
            let err = obj.extract::<NonZeroU128>().unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyValueError>(py));
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Python;
    use crate::ToPyObject;
    use std::num::*;

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
                    Python::with_gil(|py| {
                    let obj = ("123").to_object(py);
                    let err = obj.extract::<$t>(py).unwrap_err();
                    assert!(err.is_instance_of::<exceptions::PyTypeError>(py));
                    });
                }

                #[test]
                fn from_py_float_type_error() {
                    Python::with_gil(|py| {
                    let obj = (12.3).to_object(py);
                    let err = obj.extract::<$t>(py).unwrap_err();
                    assert!(err.is_instance_of::<exceptions::PyTypeError>(py));});
                }

                #[test]
                fn to_py_object_and_back() {
                    Python::with_gil(|py| {
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

    #[test]
    fn test_nonzero_u32_max() {
        Python::with_gil(|py| {
            let v = NonZeroU32::new(std::u32::MAX).unwrap();
            let obj = v.to_object(py);
            assert_eq!(v, obj.extract::<NonZeroU32>(py).unwrap());
            assert_eq!(NonZeroU64::from(v), obj.extract::<NonZeroU64>(py).unwrap());
            assert!(obj.extract::<NonZeroI32>(py).is_err());
        });
    }

    #[test]
    fn test_nonzero_i64_max() {
        Python::with_gil(|py| {
            let v = NonZeroI64::new(std::i64::MAX).unwrap();
            let obj = v.to_object(py);
            assert_eq!(v, obj.extract::<NonZeroI64>(py).unwrap());
            assert_eq!(
                NonZeroU64::new(v.get() as u64).unwrap(),
                obj.extract::<NonZeroU64>(py).unwrap()
            );
            assert!(obj.extract::<NonZeroU32>(py).is_err());
        });
    }

    #[test]
    fn test_nonzero_i64_min() {
        Python::with_gil(|py| {
            let v = NonZeroI64::new(std::i64::MIN).unwrap();
            let obj = v.to_object(py);
            assert_eq!(v, obj.extract::<NonZeroI64>(py).unwrap());
            assert!(obj.extract::<NonZeroI32>(py).is_err());
            assert!(obj.extract::<NonZeroU64>(py).is_err());
        });
    }

    #[test]
    fn test_nonzero_u64_max() {
        Python::with_gil(|py| {
            let v = NonZeroU64::new(std::u64::MAX).unwrap();
            let obj = v.to_object(py);
            assert_eq!(v, obj.extract::<NonZeroU64>(py).unwrap());
            assert!(obj.extract::<NonZeroI64>(py).is_err());
        });
    }

    macro_rules! test_nonzero_common (
        ($test_mod_name:ident, $t:ty) => (
            mod $test_mod_name {
                use crate::exceptions;
                use crate::ToPyObject;
                use crate::Python;
                use std::num::*;

                #[test]
                fn from_py_string_type_error() {
                    Python::with_gil(|py| {
                    let obj = ("123").to_object(py);
                    let err = obj.extract::<$t>(py).unwrap_err();
                    assert!(err.is_instance_of::<exceptions::PyTypeError>(py));
                    });
                }

                #[test]
                fn from_py_float_type_error() {
                    Python::with_gil(|py| {
                    let obj = (12.3).to_object(py);
                    let err = obj.extract::<$t>(py).unwrap_err();
                    assert!(err.is_instance_of::<exceptions::PyTypeError>(py));});
                }

                #[test]
                fn to_py_object_and_back() {
                    Python::with_gil(|py| {
                    let val = <$t>::new(123).unwrap();
                    let obj = val.to_object(py);
                    assert_eq!(obj.extract::<$t>(py).unwrap(), val);});
                }
            }
        )
    );

    test_nonzero_common!(nonzero_i8, NonZeroI8);
    test_nonzero_common!(nonzero_u8, NonZeroU8);
    test_nonzero_common!(nonzero_i16, NonZeroI16);
    test_nonzero_common!(nonzero_u16, NonZeroU16);
    test_nonzero_common!(nonzero_i32, NonZeroI32);
    test_nonzero_common!(nonzero_u32, NonZeroU32);
    test_nonzero_common!(nonzero_i64, NonZeroI64);
    test_nonzero_common!(nonzero_u64, NonZeroU64);
    test_nonzero_common!(nonzero_isize, NonZeroIsize);
    test_nonzero_common!(nonzero_usize, NonZeroUsize);
    test_nonzero_common!(nonzero_i128, NonZeroI128);
    test_nonzero_common!(nonzero_u128, NonZeroU128);

    #[test]
    fn test_i64_bool() {
        Python::with_gil(|py| {
            let obj = true.to_object(py);
            assert_eq!(1, obj.extract::<i64>(py).unwrap());
            let obj = false.to_object(py);
            assert_eq!(0, obj.extract::<i64>(py).unwrap());
        })
    }

    #[test]
    fn test_i64_f64() {
        Python::with_gil(|py| {
            let obj = 12.34f64.to_object(py);
            let err = obj.extract::<i64>(py).unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyTypeError>(py));
            // with no remainder
            let obj = 12f64.to_object(py);
            let err = obj.extract::<i64>(py).unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyTypeError>(py));
        })
    }
}
