#![cfg(feature = "num-bigint")]
//!  Conversions to and from [num-bigint](https://docs.rs/num-bigint)’s [`BigInt`] and [`BigUint`] types.
//!
//! This is useful for converting Python integers when they may not fit in Rust's built-in integer types.
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! num-bigint = "*"
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"num-bigint\"] }")]
//! ```
//!
//! Note that you must use compatible versions of num-bigint and PyO3.
//! The required num-bigint version may vary based on the version of PyO3.
//!
//! ## Examples
//!
//! Using [`BigInt`] to correctly increment an arbitrary precision integer.
//! This is not possible with Rust's native integers if the Python integer is too large,
//! in which case it will fail its conversion and raise `OverflowError`.
//! ```rust,no_run
//! use num_bigint::BigInt;
//! use pyo3::prelude::*;
//!
//! #[pyfunction]
//! fn add_one(n: BigInt) -> BigInt {
//!     n + 1
//! }
//!
//! #[pymodule]
//! fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
//!     m.add_function(wrap_pyfunction!(add_one, m)?)?;
//!     Ok(())
//! }
//! ```
//!
//! Python code:
//! ```python
//! from my_module import add_one
//!
//! n = 1 << 1337
//! value = add_one(n)
//!
//! assert n + 1 == value
//! ```

#[cfg(Py_LIMITED_API)]
use crate::types::{bytes::PyBytesMethods, PyBytes};
use crate::{
    conversion::IntoPyObject, ffi, instance::Bound, types::PyInt, FromPyObject, Py, PyAny, PyErr,
    PyResult, Python,
};

use num_bigint::{BigInt, BigUint};

#[cfg(not(Py_LIMITED_API))]
use num_bigint::Sign;

// for identical functionality between BigInt and BigUint
macro_rules! bigint_conversion {
    ($rust_ty: ty, $is_signed: literal, $to_bytes: path) => {
        #[cfg_attr(docsrs, doc(cfg(feature = "num-bigint")))]
        impl<'py> IntoPyObject<'py> for $rust_ty {
            type Target = PyInt;
            type Output = Bound<'py, Self::Target>;
            type Error = PyErr;

            #[inline]
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                (&self).into_pyobject(py)
            }
        }

        #[cfg_attr(docsrs, doc(cfg(feature = "num-bigint")))]
        impl<'py> IntoPyObject<'py> for &$rust_ty {
            type Target = PyInt;
            type Output = Bound<'py, Self::Target>;
            type Error = PyErr;

            #[cfg(not(Py_LIMITED_API))]
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                use crate::ffi_ptr_ext::FfiPtrExt;
                let bytes = $to_bytes(&self);
                unsafe {
                    Ok(ffi::_PyLong_FromByteArray(
                        bytes.as_ptr().cast(),
                        bytes.len(),
                        1,
                        $is_signed.into(),
                    )
                    .assume_owned(py)
                    .cast_into_unchecked())
                }
            }

            #[cfg(Py_LIMITED_API)]
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                use $crate::py_result_ext::PyResultExt;
                use $crate::types::any::PyAnyMethods;
                let bytes = $to_bytes(&self);
                let bytes_obj = PyBytes::new(py, &bytes);
                let kwargs = if $is_signed {
                    let kwargs = crate::types::PyDict::new(py);
                    kwargs.set_item(crate::intern!(py, "signed"), true)?;
                    Some(kwargs)
                } else {
                    None
                };
                unsafe {
                    py.get_type::<PyInt>()
                        .call_method("from_bytes", (bytes_obj, "little"), kwargs.as_ref())
                        .cast_into_unchecked()
                }
            }
        }
    };
}

bigint_conversion!(BigUint, false, BigUint::to_bytes_le);
bigint_conversion!(BigInt, true, BigInt::to_signed_bytes_le);

#[cfg_attr(docsrs, doc(cfg(feature = "num-bigint")))]
impl<'py> FromPyObject<'py> for BigInt {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<BigInt> {
        let py = ob.py();
        // fast path - checking for subclass of `int` just checks a bit in the type object
        let num_owned: Py<PyInt>;
        let num = if let Ok(long) = ob.cast::<PyInt>() {
            long
        } else {
            num_owned = unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Index(ob.as_ptr()))? };
            num_owned.bind(py)
        };
        #[cfg(not(Py_LIMITED_API))]
        {
            let mut buffer = int_to_u32_vec::<true>(num)?;
            let sign = if buffer.last().copied().is_some_and(|last| last >> 31 != 0) {
                // BigInt::new takes an unsigned array, so need to convert from two's complement
                // flip all bits, 'subtract' 1 (by adding one to the unsigned array)
                let mut elements = buffer.iter_mut();
                for element in elements.by_ref() {
                    *element = (!*element).wrapping_add(1);
                    if *element != 0 {
                        // if the element didn't wrap over, no need to keep adding further ...
                        break;
                    }
                }
                // ... so just two's complement the rest
                for element in elements {
                    *element = !*element;
                }
                Sign::Minus
            } else {
                Sign::Plus
            };
            Ok(BigInt::new(sign, buffer))
        }
        #[cfg(Py_LIMITED_API)]
        {
            let n_bits = int_n_bits(num)?;
            if n_bits == 0 {
                return Ok(BigInt::from(0isize));
            }
            let bytes = int_to_py_bytes(num, (n_bits + 8) / 8, true)?;
            Ok(BigInt::from_signed_bytes_le(bytes.as_bytes()))
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "num-bigint")))]
impl<'py> FromPyObject<'py> for BigUint {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<BigUint> {
        let py = ob.py();
        // fast path - checking for subclass of `int` just checks a bit in the type object
        let num_owned: Py<PyInt>;
        let num = if let Ok(long) = ob.cast::<PyInt>() {
            long
        } else {
            num_owned = unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Index(ob.as_ptr()))? };
            num_owned.bind(py)
        };
        #[cfg(not(Py_LIMITED_API))]
        {
            let buffer = int_to_u32_vec::<false>(num)?;
            Ok(BigUint::new(buffer))
        }
        #[cfg(Py_LIMITED_API)]
        {
            let n_bits = int_n_bits(num)?;
            if n_bits == 0 {
                return Ok(BigUint::from(0usize));
            }
            let bytes = int_to_py_bytes(num, n_bits.div_ceil(8), false)?;
            Ok(BigUint::from_bytes_le(bytes.as_bytes()))
        }
    }
}

#[cfg(not(any(Py_LIMITED_API, Py_3_13)))]
#[inline]
fn int_to_u32_vec<const SIGNED: bool>(long: &Bound<'_, PyInt>) -> PyResult<Vec<u32>> {
    let mut buffer = Vec::new();
    let n_bits = int_n_bits(long)?;
    if n_bits == 0 {
        return Ok(buffer);
    }
    let n_digits = if SIGNED {
        (n_bits + 32) / 32
    } else {
        n_bits.div_ceil(32)
    };
    buffer.reserve_exact(n_digits);
    unsafe {
        crate::err::error_on_minusone(
            long.py(),
            ffi::_PyLong_AsByteArray(
                long.as_ptr().cast(),
                buffer.as_mut_ptr() as *mut u8,
                n_digits * 4,
                1,
                SIGNED.into(),
            ),
        )?;
        buffer.set_len(n_digits)
    };
    buffer
        .iter_mut()
        .for_each(|chunk| *chunk = u32::from_le(*chunk));

    Ok(buffer)
}

#[cfg(all(not(Py_LIMITED_API), Py_3_13))]
#[inline]
fn int_to_u32_vec<const SIGNED: bool>(long: &Bound<'_, PyInt>) -> PyResult<Vec<u32>> {
    let mut buffer = Vec::new();
    let mut flags = ffi::Py_ASNATIVEBYTES_LITTLE_ENDIAN;
    if !SIGNED {
        flags |= ffi::Py_ASNATIVEBYTES_UNSIGNED_BUFFER | ffi::Py_ASNATIVEBYTES_REJECT_NEGATIVE;
    }
    let n_bytes =
        unsafe { ffi::PyLong_AsNativeBytes(long.as_ptr().cast(), std::ptr::null_mut(), 0, flags) };
    let n_bytes_unsigned: usize = n_bytes
        .try_into()
        .map_err(|_| crate::PyErr::fetch(long.py()))?;
    if n_bytes == 0 {
        return Ok(buffer);
    }
    let n_digits = n_bytes_unsigned.div_ceil(4);
    buffer.reserve_exact(n_digits);
    unsafe {
        ffi::PyLong_AsNativeBytes(
            long.as_ptr().cast(),
            buffer.as_mut_ptr().cast(),
            (n_digits * 4).try_into().unwrap(),
            flags,
        );
        buffer.set_len(n_digits);
    };
    buffer
        .iter_mut()
        .for_each(|chunk| *chunk = u32::from_le(*chunk));

    Ok(buffer)
}

#[cfg(Py_LIMITED_API)]
fn int_to_py_bytes<'py>(
    long: &Bound<'py, PyInt>,
    n_bytes: usize,
    is_signed: bool,
) -> PyResult<Bound<'py, PyBytes>> {
    use crate::intern;
    use crate::types::any::PyAnyMethods;
    let py = long.py();
    let kwargs = if is_signed {
        let kwargs = crate::types::PyDict::new(py);
        kwargs.set_item(intern!(py, "signed"), true)?;
        Some(kwargs)
    } else {
        None
    };
    let bytes = long.call_method(
        intern!(py, "to_bytes"),
        (n_bytes, intern!(py, "little")),
        kwargs.as_ref(),
    )?;
    Ok(bytes.cast_into()?)
}

#[inline]
#[cfg(any(not(Py_3_13), Py_LIMITED_API))]
fn int_n_bits(long: &Bound<'_, PyInt>) -> PyResult<usize> {
    let py = long.py();
    #[cfg(not(Py_LIMITED_API))]
    {
        // fast path
        let n_bits = unsafe { ffi::_PyLong_NumBits(long.as_ptr()) };
        if n_bits == (-1isize as usize) {
            return Err(crate::PyErr::fetch(py));
        }
        Ok(n_bits)
    }

    #[cfg(Py_LIMITED_API)]
    {
        use crate::types::any::PyAnyMethods;
        // slow path
        long.call_method0(crate::intern!(py, "bit_length"))
            .and_then(|any| any.extract())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::generate_unique_module_name;
    use crate::types::{PyAnyMethods as _, PyDict, PyModule};
    use indoc::indoc;
    use pyo3_ffi::c_str;

    fn rust_fib<T>() -> impl Iterator<Item = T>
    where
        T: From<u16>,
        for<'a> &'a T: std::ops::Add<Output = T>,
    {
        let mut f0: T = T::from(1);
        let mut f1: T = T::from(1);
        std::iter::from_fn(move || {
            let f2 = &f0 + &f1;
            Some(std::mem::replace(&mut f0, std::mem::replace(&mut f1, f2)))
        })
    }

    fn python_fib(py: Python<'_>) -> impl Iterator<Item = Bound<'_, PyAny>> + '_ {
        let mut f0 = 1i32.into_pyobject(py).unwrap().into_any();
        let mut f1 = 1i32.into_pyobject(py).unwrap().into_any();
        std::iter::from_fn(move || {
            let f2 = f0.call_method1("__add__", (&f1,)).unwrap();
            Some(std::mem::replace(&mut f0, std::mem::replace(&mut f1, f2)))
        })
    }

    #[test]
    fn convert_biguint() {
        Python::attach(|py| {
            // check the first 2000 numbers in the fibonacci sequence
            for (py_result, rs_result) in python_fib(py).zip(rust_fib::<BigUint>()).take(2000) {
                // Python -> Rust
                assert_eq!(py_result.extract::<BigUint>().unwrap(), rs_result);
                // Rust -> Python
                assert!(py_result.eq(rs_result).unwrap());
            }
        });
    }

    #[test]
    fn convert_bigint() {
        Python::attach(|py| {
            // check the first 2000 numbers in the fibonacci sequence
            for (py_result, rs_result) in python_fib(py).zip(rust_fib::<BigInt>()).take(2000) {
                // Python -> Rust
                assert_eq!(py_result.extract::<BigInt>().unwrap(), rs_result);
                // Rust -> Python
                assert!(py_result.eq(&rs_result).unwrap());

                // negate

                let rs_result = rs_result * -1;
                let py_result = py_result.call_method0("__neg__").unwrap();

                // Python -> Rust
                assert_eq!(py_result.extract::<BigInt>().unwrap(), rs_result);
                // Rust -> Python
                assert!(py_result.eq(rs_result).unwrap());
            }
        });
    }

    fn python_index_class(py: Python<'_>) -> Bound<'_, PyModule> {
        let index_code = c_str!(indoc!(
            r#"
                class C:
                    def __init__(self, x):
                        self.x = x
                    def __index__(self):
                        return self.x
                "#
        ));
        PyModule::from_code(
            py,
            index_code,
            c_str!("index.py"),
            &generate_unique_module_name("index"),
        )
        .unwrap()
    }

    #[test]
    fn convert_index_class() {
        Python::attach(|py| {
            let index = python_index_class(py);
            let locals = PyDict::new(py);
            locals.set_item("index", index).unwrap();
            let ob = py
                .eval(ffi::c_str!("index.C(10)"), None, Some(&locals))
                .unwrap();
            let _: BigInt = ob.extract().unwrap();
        });
    }

    #[test]
    fn handle_zero() {
        Python::attach(|py| {
            let zero: BigInt = 0i32.into_pyobject(py).unwrap().extract().unwrap();
            assert_eq!(zero, BigInt::from(0));
        })
    }

    /// `OverflowError` on converting Python int to BigInt, see issue #629
    #[test]
    fn check_overflow() {
        Python::attach(|py| {
            macro_rules! test {
                ($T:ty, $value:expr, $py:expr) => {
                    let value = $value;
                    println!("{}: {}", stringify!($T), value);
                    let python_value = value.clone().into_pyobject(py).unwrap();
                    let roundtrip_value = python_value.extract::<$T>().unwrap();
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
        });
    }
}
