// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::{
    exceptions, ffi, AsPyPointer, FromPyObject, IntoPy, PyAny, PyErr, PyNativeType, PyObject,
    PyResult, Python, ToPyObject,
};
use std::convert::TryFrom;
use std::i64;
use std::os::raw::c_long;

fn err_if_invalid_value<T: PartialEq>(
    py: Python,
    invalid_value: T,
    actual_value: T,
) -> PyResult<T> {
    if actual_value == invalid_value && PyErr::occurred(py) {
        Err(PyErr::fetch(py))
    } else {
        Ok(actual_value)
    }
}

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

pyobject_native_var_type!(PyLong, ffi::PyLong_Type, ffi::PyLong_Check);

macro_rules! int_fits_c_long {
    ($rust_type:ty) => {
        impl ToPyObject for $rust_type {
            #![cfg_attr(feature = "cargo-clippy", allow(clippy::cast_lossless))]
            fn to_object(&self, py: Python) -> PyObject {
                unsafe { PyObject::from_owned_ptr(py, ffi::PyLong_FromLong(*self as c_long)) }
            }
        }
        impl IntoPy<PyObject> for $rust_type {
            #![cfg_attr(feature = "cargo-clippy", allow(clippy::cast_lossless))]
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

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
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
                        if ok == -1 {
                            Err(PyErr::fetch(ob.py()))
                        } else {
                            Ok(<$rust_type>::from_le_bytes(buffer))
                        }
                    }
                }
            }
        };
    }

    int_convert_128!(i128, 1);
    int_convert_128!(u128, 0);
}

// For ABI3 and PyPy, we implement the conversion manually.
#[cfg(any(Py_LIMITED_API, PyPy))]
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
                        let shifted = PyObject::from_owned_ptr(
                            py,
                            ffi::PyNumber_Rshift(ob.as_ptr(), SHIFT.into_py(py).as_ptr()),
                        );
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

#[cfg(test)]
mod test_128bit_intergers {
    use super::*;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_i128_roundtrip(x: i128) {
            Python::with_gil(|py| {
                let x_py = x.into_py(py);
                crate::py_run!(py, x_py, &format!("assert x_py == {}", x));
                let roundtripped: i128 = x_py.extract(py).unwrap();
                assert_eq!(x, roundtripped);
            })
        }
    }

    proptest! {
        #[test]
        fn test_u128_roundtrip(x: u128) {
            Python::with_gil(|py| {
                let x_py = x.into_py(py);
                crate::py_run!(py, x_py, &format!("assert x_py == {}", x));
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
            assert!(err.is_instance::<crate::exceptions::PyOverflowError>(py));
        })
    }

    #[test]
    fn test_u128_overflow() {
        Python::with_gil(|py| {
            let obj = py.eval("1 << 130", None, None).unwrap();
            let err = obj.extract::<u128>().unwrap_err();
            assert!(err.is_instance::<crate::exceptions::PyOverflowError>(py));
        })
    }
}

#[cfg(all(feature = "num-bigint", not(any(Py_LIMITED_API, PyPy))))]
mod bigint_conversion {
    use super::*;
    use crate::{err, Py};
    use num_bigint::{BigInt, BigUint};
    use std::os::raw::{c_int, c_uchar};

    #[cfg(not(all(windows, PyPy)))]
    unsafe fn extract(ob: &PyLong, buffer: &mut [c_uchar], is_signed: c_int) -> PyResult<()> {
        err::error_on_minusone(
            ob.py(),
            ffi::_PyLong_AsByteArray(
                ob.as_ptr() as *mut ffi::PyLongObject,
                buffer.as_mut_ptr(),
                buffer.len(),
                1,
                is_signed,
            ),
        )
    }

    macro_rules! bigint_conversion {
        ($rust_ty: ty, $is_signed: expr, $to_bytes: path, $from_bytes: path) => {
            impl ToPyObject for $rust_ty {
                fn to_object(&self, py: Python) -> PyObject {
                    unsafe {
                        let bytes = $to_bytes(self);
                        let obj = ffi::_PyLong_FromByteArray(
                            bytes.as_ptr() as *const c_uchar,
                            bytes.len(),
                            1,
                            $is_signed,
                        );
                        PyObject::from_owned_ptr(py, obj)
                    }
                }
            }
            impl IntoPy<PyObject> for $rust_ty {
                fn into_py(self, py: Python) -> PyObject {
                    self.to_object(py)
                }
            }
            impl<'source> FromPyObject<'source> for $rust_ty {
                fn extract(ob: &'source PyAny) -> PyResult<$rust_ty> {
                    let py = ob.py();
                    unsafe {
                        let num = ffi::PyNumber_Index(ob.as_ptr());
                        if num.is_null() {
                            return Err(PyErr::fetch(py));
                        }
                        let n_bits = ffi::_PyLong_NumBits(num);
                        let n_bytes = if n_bits < 0 {
                            return Err(PyErr::fetch(py));
                        } else if n_bits == 0 {
                            0
                        } else {
                            (n_bits as usize - 1 + $is_signed) / 8 + 1
                        };
                        let num: Py<PyLong> = Py::from_owned_ptr(py, num);
                        if n_bytes <= 128 {
                            let mut buffer = [0; 128];
                            extract(num.as_ref(py), &mut buffer[..n_bytes], $is_signed)?;
                            Ok($from_bytes(&buffer[..n_bytes]))
                        } else {
                            let mut buffer = vec![0; n_bytes];
                            extract(num.as_ref(py), &mut buffer, $is_signed)?;
                            Ok($from_bytes(&buffer))
                        }
                    }
                }
            }
        };
    }
    bigint_conversion!(BigUint, 0, BigUint::to_bytes_le, BigUint::from_bytes_le);
    bigint_conversion!(
        BigInt,
        1,
        BigInt::to_signed_bytes_le,
        BigInt::from_signed_bytes_le
    );

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::types::{PyDict, PyModule};
        use indoc::indoc;

        fn python_fib(py: Python) -> &PyModule {
            let fib_code = indoc!(
                r#"
                def fib(n):
                    f0, f1 = 0, 1
                    for _ in range(n):
                        f0, f1 = f1, f0 + f1
                    return f0

                def fib_neg(n):
                    return -fib(n)
        "#
            );
            PyModule::from_code(py, fib_code, "fib.py", "fib").unwrap()
        }

        fn rust_fib<T>(n: usize) -> T
        where
            T: From<u16>,
            for<'a> &'a T: std::ops::Add<Output = T>,
        {
            let mut f0: T = T::from(0);
            let mut f1: T = T::from(1);
            for _ in 0..n {
                let f2 = &f0 + &f1;
                f0 = std::mem::replace(&mut f1, f2);
            }
            f0
        }

        #[test]
        fn convert_biguint() {
            let gil = Python::acquire_gil();
            let py = gil.python();
            let rs_result: BigUint = rust_fib(400);
            let fib = python_fib(py);
            let locals = PyDict::new(py);
            locals.set_item("rs_result", &rs_result).unwrap();
            locals.set_item("fib", fib).unwrap();
            // Checks if Rust BigUint -> Python Long conversion is correct
            py.run("assert fib.fib(400) == rs_result", None, Some(locals))
                .unwrap();
            // Checks if Python Long -> Rust BigUint conversion is correct if N is small
            let py_result: BigUint =
                FromPyObject::extract(fib.call1("fib", (400,)).unwrap()).unwrap();
            assert_eq!(rs_result, py_result);
            // Checks if Python Long -> Rust BigUint conversion is correct if N is large
            let rs_result: BigUint = rust_fib(2000);
            let py_result: BigUint =
                FromPyObject::extract(fib.call1("fib", (2000,)).unwrap()).unwrap();
            assert_eq!(rs_result, py_result);
        }

        #[test]
        fn convert_bigint() {
            let gil = Python::acquire_gil();
            let py = gil.python();
            let rs_result = rust_fib::<BigInt>(400) * -1;
            let fib = python_fib(py);
            let locals = PyDict::new(py);
            locals.set_item("rs_result", &rs_result).unwrap();
            locals.set_item("fib", fib).unwrap();
            // Checks if Rust BigInt -> Python Long conversion is correct
            py.run("assert fib.fib_neg(400) == rs_result", None, Some(locals))
                .unwrap();
            // Checks if Python Long -> Rust BigInt conversion is correct if N is small
            let py_result: BigInt =
                FromPyObject::extract(fib.call1("fib_neg", (400,)).unwrap()).unwrap();
            assert_eq!(rs_result, py_result);
            // Checks if Python Long -> Rust BigInt conversion is correct if N is large
            let rs_result = rust_fib::<BigInt>(2000) * -1;
            let py_result: BigInt =
                FromPyObject::extract(fib.call1("fib_neg", (2000,)).unwrap()).unwrap();
            assert_eq!(rs_result, py_result);
        }

        fn python_index_class(py: Python) -> &PyModule {
            let index_code = indoc!(
                r#"
                class C:
                    def __init__(self, x):
                        self.x = x
                    def __index__(self):
                        return self.x
                "#
            );
            PyModule::from_code(py, index_code, "index.py", "index").unwrap()
        }

        #[test]
        fn convert_index_class() {
            let gil = Python::acquire_gil();
            let py = gil.python();
            let index = python_index_class(py);
            let locals = PyDict::new(py);
            locals.set_item("index", index).unwrap();
            let ob = py.eval("index.C(10)", None, Some(locals)).unwrap();
            let _: BigInt = FromPyObject::extract(ob).unwrap();
        }

        #[test]
        fn handle_zero() {
            let gil = Python::acquire_gil();
            let py = gil.python();
            let fib = python_fib(py);
            let zero: BigInt = FromPyObject::extract(fib.call1("fib", (0,)).unwrap()).unwrap();
            assert_eq!(zero, BigInt::from(0));
        }

        /// `OverflowError` on converting Python int to BigInt, see issue #629
        #[test]
        fn check_overflow() {
            let gil = Python::acquire_gil();
            let py = gil.python();
            macro_rules! test {
                ($T:ty, $value:expr, $py:expr) => {
                    let value = $value;
                    println!("{}: {}", stringify!($T), value);
                    let python_value = value.clone().to_object(py);
                    let roundtrip_value = python_value.extract::<$T>(py).unwrap();
                    assert_eq!(value, roundtrip_value);
                };
            }
            for i in 0..=256usize {
                // test a lot of values to help catch other bugs too
                test!(BigInt, BigInt::from(i), py);
                test!(BigUint, BigUint::from(i), py);
                test!(BigInt, -BigInt::from(i), py);
                test!(BigInt, BigInt::from(1) << i, py);
                test!(BigUint, BigUint::from(1u32) << i, py);
                test!(BigInt, -BigInt::from(1) << i, py);
                test!(BigInt, (BigInt::from(1) << i) + 1u32, py);
                test!(BigUint, (BigUint::from(1u32) << i) + 1u32, py);
                test!(BigInt, (-BigInt::from(1) << i) + 1u32, py);
                test!(BigInt, (BigInt::from(1) << i) - 1u32, py);
                test!(BigUint, (BigUint::from(1u32) << i) - 1u32, py);
                test!(BigInt, (-BigInt::from(1) << i) - 1u32, py);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Python;
    use crate::ToPyObject;

    #[test]
    fn test_u32_max() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = std::u32::MAX;
        let obj = v.to_object(py);
        assert_eq!(v, obj.extract::<u32>(py).unwrap());
        assert_eq!(u64::from(v), obj.extract::<u64>(py).unwrap());
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

    macro_rules! test_common (
        ($test_mod_name:ident, $t:ty) => (
            mod $test_mod_name {
                use crate::exceptions;
                use crate::ToPyObject;
                use crate::Python;

                #[test]
                fn from_py_string_type_error() {
                    let gil = Python::acquire_gil();
                    let py = gil.python();

                    let obj = ("123").to_object(py);
                    let err = obj.extract::<$t>(py).unwrap_err();
                    assert!(err.is_instance::<exceptions::PyTypeError>(py));
                }

                #[test]
                fn from_py_float_type_error() {
                    let gil = Python::acquire_gil();
                    let py = gil.python();

                    let obj = (12.3).to_object(py);
                    let err = obj.extract::<$t>(py).unwrap_err();
                    assert!(err.is_instance::<exceptions::PyTypeError>(py));
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
    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    test_common!(i128, i128);
    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    test_common!(u128, u128);
}
