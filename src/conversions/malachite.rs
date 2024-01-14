#![cfg(feature = "malachite")]
//!  Conversions to and from [malachite](https://docs.rs/malachite)’s [`Integer`] and [`Natural`] types.
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
//! Note that you must use compatible versions of malachite and PyO3.
//! The required malachite version may vary based on the version of PyO3.
//! You must not use [`32_bit_limbs`] feature of malachite.
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
    ffi, types::*, FromPyObject, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject,
};

use malachite::{Natural, Integer};
use malachite::num::basic::traits::Zero;


#[cfg_attr(docsrs, doc(cfg(feature = "malachite")))]
impl<'source> FromPyObject<'source> for Integer {
    fn extract(ob: &'source PyAny) -> PyResult<Integer> {
        // get the Python interpreter
        let py = ob.py();

        // get PyLong object
        let num =
            if let Ok(long) = ob.downcast::<PyLong>() {
                long
            } else {
                let num_owned: Py<PyLong> = unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Index(ob.as_ptr()))? };
                num_owned.as_ref(py)
            };

        // check if number is zero, and if so, return zero
        let n_bits = int_n_bits(num)?;
        if n_bits == 0 {
            return Ok(Integer::ZERO);
        }

        // the number of bytes needed to store the integer padded to 64-bit limbs
        let n_bytes = (n_bits + 63) / 64;

        #[cfg(not(Py_LIMITED_API))]
        {
            let limbs_64 = int_to_u64_vec(num, n_bytes, true)?;
            Ok(Integer::from_owned_twos_complement_limbs_asc(limbs_64))
        }

        #[cfg(Py_LIMITED_API)]
        {
            let bytes = int_to_py_bytes(num, n_bytes, true)?.as_bytes();
            let n_limbs_64 = n_bytes / 8;  // the number of 64-bit limbs needed to store the integer
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
        let num =
            if let Ok(long) = ob.downcast::<PyLong>() {
                long
            } else {
                let num_owned: Py<PyLong> = unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Index(ob.as_ptr()))? };
                num_owned.as_ref(py)
            };

        // check if number is zero, and if so, return zero
        let n_bits = int_n_bits(num)?;
        if n_bits == 0 {
            return Ok(Natural::ZERO);
        }

        // the number of bytes needed to store the integer padded to 64-bit limbs
        let n_bytes = (n_bits + 63) / 64;

        #[cfg(not(Py_LIMITED_API))]
        {
            let limbs_64 = int_to_u64_vec(num, n_bytes, false)?;
            Ok(Natural::from_owned_limbs_asc(limbs_64))
        }
        #[cfg(Py_LIMITED_API)]
        {
            let bytes = int_to_py_bytes(num, n_bytes, false)?.as_bytes();
            let n_limbs_64 = n_bytes / 8;  // the number of 64-bit limbs needed to store the integer
            let mut limbs_64 = Vec::with_capacity(n_limbs_64);
            for i in (0..n_bytes).step_by(8) {
                limbs_64.push(u64::from_le_bytes(bytes[i..(i + 8)].try_into().unwrap()));
            }
            Ok(Natural::from_owned_limbs_asc(limbs_64))
        }
    }
}


/// Converts a Python integer to a Vec of u64s.
/// Takes number of limbs to convert to.
/// IF `is_signed` is true, the integer is treated as signed, and two's complement is returned.
#[cfg(not(Py_LIMITED_API))]
#[inline]
fn int_to_u64_vec(long: &PyLong, n_digits: usize, is_signed: bool) -> PyResult<Vec<u64>> {
    let mut buffer = Vec::with_capacity(n_digits);
    unsafe {
        crate::err::error_on_minusone(
            long.py(),
            ffi::_PyLong_AsByteArray(
                long.as_ptr().cast(),  // ptr to PyLong object
                buffer.as_mut_ptr() as *mut u8,  // ptr to first byte of buffer
                n_digits * 8,  // 8 bytes per u64
                1,  // little endian
                is_signed.into(),  // signed flag
            ),
        )?;
        buffer.set_len(n_digits)  // set buffer length to the number of digits
    };
    buffer
        .iter_mut()
        .for_each(|chunk| *chunk = u64::from_le(*chunk));

    Ok(buffer)
}


/// Converts a Python integer to a Python bytes object.
/// Takes number of bytes to convert to (can be calculated from the number of bits in the integer).
/// IF `is_signed` is true, the integer is treated as signed, and two's complement is returned.
#[cfg(Py_LIMITED_API)]
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
            return Err(crate::PyErr::fetch(py));
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
