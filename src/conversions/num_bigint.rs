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
//! ```rust
//! use num_bigint::BigInt;
//! use pyo3::prelude::*;
//!
//! #[pyfunction]
//! fn add_one(n: BigInt) -> BigInt {
//!     n + 1
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

use num_bigint::{BigInt, BigUint};
use std::os::raw::{c_int, c_uchar};

#[cfg(not(Py_LIMITED_API))]
unsafe fn extract(ob: &PyLong, buffer: &mut [c_uchar], is_signed: c_int) -> PyResult<()> {
    crate::err::error_on_minusone(
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

#[cfg(Py_LIMITED_API)]
unsafe fn extract(ob: &PyLong, buffer: &mut [c_uchar], is_signed: c_int) -> PyResult<()> {
    use crate::intern;
    let py = ob.py();
    let kwargs = if is_signed != 0 {
        let kwargs = PyDict::new(py);
        kwargs.set_item(intern!(py, "signed"), true)?;
        Some(kwargs)
    } else {
        None
    };
    let bytes_obj = ob
        .getattr(intern!(py, "to_bytes"))?
        .call((buffer.len(), "little"), kwargs)?;
    let bytes: &PyBytes = bytes_obj.downcast_unchecked();
    buffer.copy_from_slice(bytes.as_bytes());
    Ok(())
}

macro_rules! bigint_conversion {
    ($rust_ty: ty, $is_signed: expr, $to_bytes: path, $from_bytes: path) => {
        #[cfg_attr(docsrs, doc(cfg(feature = "num-bigint")))]
        impl ToPyObject for $rust_ty {
            #[cfg(not(Py_LIMITED_API))]
            fn to_object(&self, py: Python<'_>) -> PyObject {
                let bytes = $to_bytes(self);
                unsafe {
                    let obj = ffi::_PyLong_FromByteArray(
                        bytes.as_ptr() as *const c_uchar,
                        bytes.len(),
                        1,
                        $is_signed,
                    );
                    PyObject::from_owned_ptr(py, obj)
                }
            }

            #[cfg(Py_LIMITED_API)]
            fn to_object(&self, py: Python<'_>) -> PyObject {
                let bytes = $to_bytes(self);
                let bytes_obj = PyBytes::new(py, &bytes);
                let kwargs = if $is_signed > 0 {
                    let kwargs = PyDict::new(py);
                    kwargs.set_item(crate::intern!(py, "signed"), true).unwrap();
                    Some(kwargs)
                } else {
                    None
                };
                py.get_type::<PyLong>()
                    .call_method("from_bytes", (bytes_obj, "little"), kwargs)
                    .expect("int.from_bytes() failed during to_object()") // FIXME: #1813 or similar
                    .into()
            }
        }

        #[cfg_attr(docsrs, doc(cfg(feature = "num-bigint")))]
        impl IntoPy<PyObject> for $rust_ty {
            fn into_py(self, py: Python<'_>) -> PyObject {
                self.to_object(py)
            }
        }

        #[cfg_attr(docsrs, doc(cfg(feature = "num-bigint")))]
        impl<'source> FromPyObject<'source> for $rust_ty {
            fn extract(ob: &'source PyAny) -> PyResult<$rust_ty> {
                let py = ob.py();
                unsafe {
                    let num: Py<PyLong> =
                        Py::from_owned_ptr_or_err(py, ffi::PyNumber_Index(ob.as_ptr()))?;

                    let n_bytes = {
                        cfg_if::cfg_if! {
                            if #[cfg(not(Py_LIMITED_API))] {
                                // fast path
                                let n_bits = ffi::_PyLong_NumBits(num.as_ptr());
                                if n_bits == (-1isize as usize) {
                                    return Err(crate::PyErr::fetch(py));
                                } else if n_bits == 0 {
                                    0
                                } else {
                                    (n_bits - 1 + $is_signed) / 8 + 1
                                }
                            } else {
                                // slow path
                                let n_bits_obj = num.getattr(py, crate::intern!(py, "bit_length"))?.call0(py)?;
                                let n_bits_int: &PyLong = n_bits_obj.downcast_unchecked(py);
                                let n_bits = n_bits_int.extract::<usize>()?;
                                if n_bits == 0 {
                                    0
                                } else {
                                    (n_bits - 1 + $is_signed) / 8 + 1
                                }
                            }
                        }
                    };

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
mod tests {
    use super::*;
    use crate::types::{PyDict, PyModule};
    use indoc::indoc;

    fn python_fib(py: Python<'_>) -> &PyModule {
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
        Python::with_gil(|py| {
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
                FromPyObject::extract(fib.getattr("fib").unwrap().call1((400,)).unwrap()).unwrap();
            assert_eq!(rs_result, py_result);
            // Checks if Python Long -> Rust BigUint conversion is correct if N is large
            let rs_result: BigUint = rust_fib(2000);
            let py_result: BigUint =
                FromPyObject::extract(fib.getattr("fib").unwrap().call1((2000,)).unwrap()).unwrap();
            assert_eq!(rs_result, py_result);
        });
    }

    #[test]
    fn convert_bigint() {
        Python::with_gil(|py| {
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
                FromPyObject::extract(fib.getattr("fib_neg").unwrap().call1((400,)).unwrap())
                    .unwrap();
            assert_eq!(rs_result, py_result);
            // Checks if Python Long -> Rust BigInt conversion is correct if N is large
            let rs_result = rust_fib::<BigInt>(2000) * -1;
            let py_result: BigInt =
                FromPyObject::extract(fib.getattr("fib_neg").unwrap().call1((2000,)).unwrap())
                    .unwrap();
            assert_eq!(rs_result, py_result);
        })
    }

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

    #[test]
    fn convert_index_class() {
        Python::with_gil(|py| {
            let index = python_index_class(py);
            let locals = PyDict::new(py);
            locals.set_item("index", index).unwrap();
            let ob = py.eval("index.C(10)", None, Some(locals)).unwrap();
            let _: BigInt = FromPyObject::extract(ob).unwrap();
        });
    }

    #[test]
    fn handle_zero() {
        Python::with_gil(|py| {
            let fib = python_fib(py);
            let zero: BigInt =
                FromPyObject::extract(fib.getattr("fib").unwrap().call1((0,)).unwrap()).unwrap();
            assert_eq!(zero, BigInt::from(0));
        })
    }

    /// `OverflowError` on converting Python int to BigInt, see issue #629
    #[test]
    fn check_overflow() {
        Python::with_gil(|py| {
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
        });
    }
}
