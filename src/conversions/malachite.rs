#![cfg(feature = "malachite")]
//! Conversions to and from [malachite]’s [`Integer`] and [`Natural`] types.
//!
//! This is useful for converting Python integers when they may not fit in Rust's built-in integer types.
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! malachite = "*"
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"malachite\"] }")]
//! ```
//!
//! If you use the `32_bit_limbs` feature of malachite, you must use the `malachite-32bit` feature of PyO3 instead.
//!
//! ## Examples
//!
//! Using [`Integer`] to correctly increment an arbitrary precision integer.
//! This is not possible with Rust's native integers if the Python integer is too large,
//! in which case it will fail its conversion and raise `OverflowError`.
//! ```rust
//! use malachite::Integer;
//! use malachite::num::basic::traits::One
//! use pyo3::prelude::*;
//!
//! #[pyfunction]
//! fn add_one(n: Integer) -> Integer {
//!     n + Integer::ONE
//! }
//!
//! #[pymodule]
//! fn my_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
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

use crate::{
    exceptions::PyValueError, ffi, types::*, FromPyObject, IntoPy, Py, PyErr, PyObject, PyResult,
    Python, ToPyObject,
};
use malachite::num::basic::traits::Zero;
use malachite::{Integer, Natural};

#[cfg_attr(docsrs, doc(cfg(feature = "malachite")))]
impl<'source> FromPyObject<'source> for Integer {
    fn extract(ob: &'source PyAny) -> PyResult<Integer> {
        // get the Python interpreter
        let py = ob.py();

        // get PyLong object
        let num_owned: Py<PyLong>;
        let num = if let Ok(long) = ob.downcast::<PyLong>() {
            long
        } else {
            num_owned = unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Index(ob.as_ptr()))? };
            num_owned.as_ref(py)
        };

        // check if number is zero, and if so, return zero
        let n_bits = int_n_bits(num)?;
        if n_bits == 0 {
            return Ok(Integer::ZERO);
        }

        // the number of bytes needed to store the integer
        let mut n_bytes = (n_bits + 7 + 1) >> 3; // +1 for the sign bit

        #[cfg(feature = "malachite-32bit")]
        {
            // convert the number of bytes to a multiple of 4, because of 32-bit limbs
            n_bytes = ((n_bytes + 7) >> 2) << 2;
        }
        #[cfg(not(feature = "malachite-32bit"))]
        {
            // convert the number of bytes to a multiple of 8, because of 64-bit limbs
            n_bytes = ((n_bytes + 7) >> 3) << 3;
        }

        #[cfg(not(Py_LIMITED_API))]
        {
            let limbs = int_to_limbs(num, n_bytes, true)?;
            Ok(Integer::from_owned_twos_complement_limbs_asc(limbs))
        }
        #[cfg(all(Py_LIMITED_API, feature = "malachite-32bit"))]
        {
            let bytes = int_to_py_bytes(num, n_bytes, true)?.as_bytes();
            let n_limbs_32 = n_bytes >> 2; // the number of 32-bit limbs needed to store the integer
            let mut limbs_32 = Vec::with_capacity(n_limbs_32);
            for i in (0..n_bytes).step_by(4) {
                limbs_32.push(u32::from_le_bytes(bytes[i..(i + 4)].try_into().unwrap()));
            }
            Ok(Integer::from_owned_twos_complement_limbs_asc(limbs_32))
        }
        #[cfg(all(Py_LIMITED_API, not(feature = "malachite-32bit")))]
        {
            let bytes = int_to_py_bytes(num, n_bytes, true)?.as_bytes();
            let n_limbs_64 = n_bytes >> 3; // the number of 64-bit limbs needed to store the integer
            let mut limbs_64 = Vec::with_capacity(n_limbs_64);
            for i in (0..n_bytes).step_by(8) {
                limbs_64.push(u64::from_le_bytes(bytes[i..(i + 8)].try_into().unwrap()));
            }
            Ok(Integer::from_owned_twos_complement_limbs_asc(limbs_64))
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "malachite")))]
impl<'source> FromPyObject<'source> for Natural {
    fn extract(ob: &'source PyAny) -> PyResult<Natural> {
        // get the Python interpreter
        let py = ob.py();

        // get PyLong object
        let num_owned: Py<PyLong>;
        let num = if let Ok(long) = ob.downcast::<PyLong>() {
            long
        } else {
            num_owned = unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Index(ob.as_ptr()))? };
            num_owned.as_ref(py)
        };

        // check if number is negative, and if so, raise TypeError
        if num.lt(0)? {
            return Err(PyErr::new::<PyValueError, _>(
                "expected non-negative integer",
            ));
        }

        // check if number is zero, and if so, return zero
        let n_bits = int_n_bits(num)?;
        if n_bits == 0 {
            return Ok(Natural::ZERO);
        }

        // the number of bytes needed to store the integer
        let mut n_bytes = (n_bits + 7) >> 3;

        #[cfg(feature = "malachite-32bit")]
        {
            // convert the number of bytes to a multiple of 4, because of 32-bit limbs
            n_bytes = ((n_bytes + 7) >> 2) << 2;
        }
        #[cfg(not(feature = "malachite-32bit"))]
        {
            // convert the number of bytes to a multiple of 8, because of 64-bit limbs
            n_bytes = ((n_bytes + 7) >> 3) << 3;
        }

        #[cfg(not(Py_LIMITED_API))]
        {
            let limbs = int_to_limbs(num, n_bytes, false)?;
            Ok(Natural::from_owned_limbs_asc(limbs))
        }
        #[cfg(all(Py_LIMITED_API, feature = "malachite-32bit"))]
        {
            let bytes = int_to_py_bytes(num, n_bytes, false)?.as_bytes();
            let n_limbs_32 = n_bytes >> 2; // the number of 32-bit limbs needed to store the integer
            let mut limbs_32 = Vec::with_capacity(n_limbs_32);
            for i in (0..n_bytes).step_by(4) {
                limbs_32.push(u32::from_le_bytes(bytes[i..(i + 4)].try_into().unwrap()));
            }
            Ok(Natural::from_owned_limbs_asc(limbs_32))
        }
        #[cfg(all(Py_LIMITED_API, not(feature = "malachite-32bit")))]
        {
            let bytes = int_to_py_bytes(num, n_bytes, false)?.as_bytes();
            let n_limbs_64 = n_bytes >> 3; // the number of 64-bit limbs needed to store the integer
            let mut limbs_64 = Vec::with_capacity(n_limbs_64);
            for i in (0..n_bytes).step_by(8) {
                limbs_64.push(u64::from_le_bytes(bytes[i..(i + 8)].try_into().unwrap()));
            }
            Ok(Natural::from_owned_limbs_asc(limbs_64))
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "malachite")))]
impl ToPyObject for Integer {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        if self == &Integer::ZERO {
            return 0.to_object(py);
        }

        let bytes = limbs_to_bytes(
            self.twos_complement_limbs(),
            self.twos_complement_limb_count(),
        );

        #[cfg(not(Py_LIMITED_API))]
        unsafe {
            let obj = ffi::_PyLong_FromByteArray(
                bytes.as_ptr().cast(),
                bytes.len(),
                1,           // little endian
                true.into(), // signed
            );
            PyObject::from_owned_ptr(py, obj)
        }

        #[cfg(Py_LIMITED_API)]
        {
            let bytes_obj = PyBytes::new(py, &bytes);
            let kwargs = PyDict::new(py);
            kwargs.set_item(crate::intern!(py, "signed"), true).unwrap();
            let kwargs = Some(kwargs);
            py.get_type::<PyLong>()
                .call_method("from_bytes", (bytes_obj, "little"), kwargs)
                .expect("int.from_bytes() failed during to_object()")
                .into()
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "malachite")))]
impl ToPyObject for Natural {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        if self == &Natural::ZERO {
            return 0.to_object(py);
        }

        let bytes = limbs_to_bytes(self.limbs(), self.limb_count());

        #[cfg(not(Py_LIMITED_API))]
        unsafe {
            let obj = ffi::_PyLong_FromByteArray(
                bytes.as_ptr().cast(),
                bytes.len(),
                1,            // little endian
                false.into(), // unsigned
            );
            PyObject::from_owned_ptr(py, obj)
        }

        #[cfg(Py_LIMITED_API)]
        {
            let bytes_obj = PyBytes::new(py, &bytes);
            let kwargs = None;
            py.get_type::<PyLong>()
                .call_method("from_bytes", (bytes_obj, "little"), kwargs)
                .expect("int.from_bytes() failed during to_object()")
                .into()
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "malachite")))]
impl IntoPy<PyObject> for Integer {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "malachite")))]
impl IntoPy<PyObject> for Natural {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

/// Convert 32-bit limbs (little endian) used by malachite to bytes (little endian)
#[cfg(feature = "malachite-32bit")]
#[inline]
fn limbs_to_bytes(limbs: impl Iterator<Item = u32>, limb_count: u64) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((limb_count << 3) as usize);

    for limb in limbs {
        for byte in limb.to_le_bytes() {
            bytes.push(byte);
        }
    }

    bytes
}

/// Convert 64-bit limbs (little endian) used by malachite to bytes (little endian)
#[cfg(not(feature = "malachite-32bit"))]
#[inline]
fn limbs_to_bytes(limbs: impl Iterator<Item = u64>, limb_count: u64) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((limb_count << 3) as usize);

    for limb in limbs {
        for byte in limb.to_le_bytes() {
            bytes.push(byte);
        }
    }

    bytes
}

/// Converts a Python integer to a vector of 32-bit limbs (little endian).
/// Takes number of bytes to convert to. Multiple of 4.
/// If `is_signed` is true, the integer is treated as signed, and two's complement is returned.
#[cfg(all(not(Py_LIMITED_API), feature = "malachite-32bit"))]
#[inline]
fn int_to_limbs(long: &PyLong, n_bytes: usize, is_signed: bool) -> PyResult<Vec<u32>> {
    let mut buffer = Vec::with_capacity(n_bytes);
    unsafe {
        crate::err::error_on_minusone(
            long.py(),
            ffi::_PyLong_AsByteArray(
                long.as_ptr().cast(),           // ptr to PyLong object
                buffer.as_mut_ptr() as *mut u8, // ptr to first byte of buffer
                n_bytes << 2,                   // 4 bytes per u32
                1,                              // little endian
                is_signed.into(),               // signed flag
            ),
        )?;
        buffer.set_len(n_bytes) // set buffer length to the number of bytes
    };
    buffer
        .iter_mut()
        .for_each(|chunk| *chunk = u32::from_le(*chunk));

    Ok(buffer)
}

/// Converts a Python integer to a vector of 64-bit limbs (little endian).
/// Takes number of bytes to convert to. Multiple of 8.
/// If `is_signed` is true, the integer is treated as signed, and two's complement is returned.
#[cfg(all(not(Py_LIMITED_API), not(feature = "malachite-32bit")))]
#[inline]
fn int_to_limbs(long: &PyLong, n_bytes: usize, is_signed: bool) -> PyResult<Vec<u64>> {
    let mut buffer = Vec::with_capacity(n_bytes);
    unsafe {
        crate::err::error_on_minusone(
            long.py(),
            ffi::_PyLong_AsByteArray(
                long.as_ptr().cast(),           // ptr to PyLong object
                buffer.as_mut_ptr() as *mut u8, // ptr to first byte of buffer
                n_bytes << 3,                   // 8 bytes per u64
                1,                              // little endian
                is_signed.into(),               // signed flag
            ),
        )?;
        buffer.set_len(n_bytes) // set buffer length to the number of bytes
    };
    buffer
        .iter_mut()
        .for_each(|chunk| *chunk = u64::from_le(*chunk));

    Ok(buffer)
}

/// Converts a Python integer to a Python bytes object. Bytes are in little endian order.
/// Takes number of bytes to convert to (can be calculated from the number of bits in the integer).
/// If `is_signed` is true, the integer is treated as signed, and two's complement is returned.
#[cfg(Py_LIMITED_API)]
#[inline]
fn int_to_py_bytes(long: &PyLong, n_bytes: usize, is_signed: bool) -> PyResult<&PyBytes> {
    use crate::intern;

    // get the Python interpreter
    let py = long.py();

    // setup kwargs for to_bytes (only if signed)
    let kwargs = if is_signed {
        let kwargs = PyDict::new(py);
        kwargs.set_item(intern!(py, "signed"), true)?;
        Some(kwargs)
    } else {
        None
    };

    // call to_bytes
    let bytes = long.call_method(
        intern!(py, "to_bytes"),
        (n_bytes, intern!(py, "little")),
        kwargs,
    )?;

    // downcast to PyBytes
    Ok(bytes.downcast()?)
}

/// Returns the number of bits in the absolute value of the given integer.
/// The number of bits returned is the smallest number of bits that can represent the integer,
/// not the multiple of 8 (bytes) that it would take up in memory.
#[inline]
fn int_n_bits(long: &PyLong) -> PyResult<usize> {
    let py = long.py();

    #[cfg(not(Py_LIMITED_API))]
    {
        // fast path
        let n_bits = unsafe { ffi::_PyLong_NumBits(long.as_ptr()) };
        if n_bits == (-1isize as usize) {
            return Err(PyErr::fetch(py));
        }
        Ok(n_bits)
    }

    #[cfg(Py_LIMITED_API)]
    {
        // slow path
        long.call_method0(crate::intern!(py, "bit_length"))
            .and_then(PyAny::extract)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PyDict, PyModule};
    use indoc::indoc;

    /// Fibonacci sequence iterator (Rust)
    fn rust_fib<T>() -> impl Iterator<Item = T>
    where
        T: From<u8>,
        for<'a> &'a T: std::ops::Add<Output = T>,
    {
        let mut f0: T = T::from(1);
        let mut f1: T = T::from(1);
        std::iter::from_fn(move || {
            let f2 = &f0 + &f1;
            Some(std::mem::replace(&mut f0, std::mem::replace(&mut f1, f2)))
        })
    }

    /// Fibonacci sequence iterator (Python)
    fn python_fib(py: Python<'_>) -> impl Iterator<Item = PyObject> + '_ {
        let mut f0 = 1.to_object(py);
        let mut f1 = 1.to_object(py);
        std::iter::from_fn(move || {
            let f2 = f0.call_method1(py, "__add__", (f1.as_ref(py),)).unwrap();
            Some(std::mem::replace(&mut f0, std::mem::replace(&mut f1, f2)))
        })
    }

    /// Generate test python class
    fn python_index_class(py: Python<'_>) -> &PyModule {
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

    /// Test conversion to and from Natural
    /// Tests the first 2000 numbers in the fibonacci sequence
    #[test]
    fn convert_natural() {
        Python::with_gil(|py| {
            // check the first 2000 numbers in the fibonacci sequence
            for (py_result, rs_result) in python_fib(py).zip(rust_fib::<Natural>()).take(2000) {
                // Python -> Rust
                assert_eq!(py_result.extract::<Natural>(py).unwrap(), rs_result);
                // Rust -> Python
                assert!(py_result.as_ref(py).eq(rs_result).unwrap());
            }
        });
    }

    /// Test conversion to and from Integer
    /// Tests the first 2000 numbers in the fibonacci sequence and their negations
    #[test]
    fn convert_integer() {
        Python::with_gil(|py| {
            // check the first 2000 numbers in the fibonacci sequence
            for (py_result, rs_result) in python_fib(py).zip(rust_fib::<Integer>()).take(2000) {
                // Python -> Rust
                assert_eq!(py_result.extract::<Integer>(py).unwrap(), rs_result);
                // Rust -> Python
                assert!(py_result.as_ref(py).eq(&rs_result).unwrap());

                // negate
                let rs_result = rs_result * Integer::from(-1);
                let py_result = py_result.call_method0(py, "__neg__").unwrap();

                // Python -> Rust
                assert_eq!(py_result.extract::<Integer>(py).unwrap(), rs_result);
                // Rust -> Python
                assert!(py_result.as_ref(py).eq(rs_result).unwrap());
            }
        });
    }

    /// Test Python class conversion
    #[test]
    fn convert_index_class() {
        Python::with_gil(|py| {
            let index = python_index_class(py);
            let locals = PyDict::new(py);
            locals.set_item("index", index).unwrap();
            let ob = py.eval("index.C(10)", None, Some(locals)).unwrap();
            let integer: Integer = FromPyObject::extract(ob).unwrap();
            let natural: Natural = FromPyObject::extract(ob).unwrap();

            assert_eq!(integer, Integer::from(10));
            assert_eq!(natural, Natural::from(10_u8));

            let ob2 = py.eval("index.C(-10)", None, Some(locals)).unwrap();
            let integer2: Integer = FromPyObject::extract(ob2).unwrap();

            assert_eq!(integer2, Integer::from(-10));
        });
    }

    /// Test conversion to and from zero
    #[test]
    fn handle_zero() {
        Python::with_gil(|py| {
            // Python -> Rust
            let zero_integer: Integer = 0.to_object(py).extract(py).unwrap();
            let zero_natural: Natural = 0.to_object(py).extract(py).unwrap();
            assert_eq!(zero_integer, Integer::from(0));
            assert_eq!(zero_natural, Natural::from(0_u8));

            // Rust -> Python
            let zero_integer = zero_integer.to_object(py);
            let zero_natural = zero_natural.to_object(py);
            assert!(zero_integer.as_ref(py).eq(Integer::from(0)).unwrap());
            assert!(zero_natural.as_ref(py).eq(Natural::from(0_u8)).unwrap());
        })
    }

    /// Test for possible overflows
    #[test]
    fn check_overflow() {
        Python::with_gil(|py| {
            macro_rules! test {
                ($T:ty, $value:expr, $py:expr) => {
                    let value = $value;
                    println!("{}: {}", stringify!($T), value);
                    let python_value = value.clone().into_py(py);
                    let roundtrip_value = python_value.extract::<$T>(py).unwrap();
                    assert_eq!(value, roundtrip_value);
                };
            }

            for i in 0..=256usize {
                // test a lot of values to help catch other bugs too
                test!(Integer, Integer::from(i), py);
                test!(Natural, Natural::from(i), py);
                test!(Integer, -Integer::from(i), py);
                test!(Integer, Integer::from(1) << i, py);
                test!(Natural, Natural::from(1u32) << i, py);
                test!(Integer, -Integer::from(1) << i, py);
                test!(Integer, (Integer::from(1) << i) + Integer::from(1u32), py);
                test!(
                    Natural,
                    (Natural::from(1u32) << i) + Natural::from(1u32),
                    py
                );
                test!(Integer, (-Integer::from(1) << i) + Integer::from(1u32), py);
                test!(Integer, (Integer::from(1) << i) - Integer::from(1u32), py);
                test!(
                    Natural,
                    (Natural::from(1u32) << i) - Natural::from(1u32),
                    py
                );
                test!(Integer, (-Integer::from(1) << i) - Integer::from(1u32), py);
            }
        });
    }

    /// Test error when converting negative integer to Natural
    #[test]
    fn negative_natural() {
        Python::with_gil(|py| {
            let zero = 0.to_object(py);
            let minus_one = (-1).to_object(py);
            assert_eq!(zero.extract::<Natural>(py).unwrap(), Natural::ZERO);
            assert!(minus_one
                .extract::<Natural>(py)
                .unwrap_err()
                .get_type(py)
                .is(PyType::new::<PyValueError>(py)));
        });
    }
}
