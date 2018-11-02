//! common macros for num2.rs and num3.rs

use crate::err::{PyErr, PyResult};
use crate::python::Python;
use std::os::raw::c_int;

pub(super) fn err_if_invalid_value<T: PartialEq>(
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

#[macro_export]
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

        impl<'source> FromPyObject<'source> for $rust_type {
            fn extract(obj: &'source PyObjectRef) -> PyResult<Self> {
                let val = $crate::objectprotocol::ObjectProtocol::extract::<$larger_type>(obj)?;
                match cast::<$larger_type, $rust_type>(val) {
                    Some(v) => Ok(v),
                    None => Err(exceptions::OverflowError.into())
                }
            }
        }
    )
);

// for 128bit Integers
#[macro_export]
macro_rules! int_convert_bignum (
    ($rust_type: ty, $byte_size: expr, $is_little_endian: expr, $is_signed: expr) => (
        impl ToPyObject for $rust_type {
            #[inline]
            fn to_object(&self, py: Python) -> PyObject {
                self.into_object(py)
            }
        }
        impl IntoPyObject for $rust_type {
            fn into_object(self, py: Python) -> PyObject {
                unsafe {
                    let bytes = ::std::mem::transmute::<_, [c_uchar; $byte_size]>(self);
                    let obj = ffi::_PyLong_FromByteArray(
                        bytes.as_ptr() as *const c_uchar,
                        $byte_size,
                        $is_little_endian,
                        $is_signed,
                    );
                    PyObject::from_owned_ptr_or_panic(py, obj)
                }
            }
        }
        impl<'source> FromPyObject<'source> for $rust_type {
            fn extract(ob: &'source PyObjectRef) -> PyResult<$rust_type> {
                unsafe {
                    let num = ffi::PyNumber_Index(ob.as_ptr());
                    if num.is_null() {
                         return Err(PyErr::fetch(ob.py()));
                    }
                    let buffer: [c_uchar; $byte_size] = [0; $byte_size];
                    let ok = ffi::_PyLong_AsByteArray(
                        ob.as_ptr() as *mut ffi::PyLongObject,
                        buffer.as_ptr() as *const c_uchar,
                        $byte_size,
                        $is_little_endian,
                        $is_signed,
                    );
                    if ok == -1 {
                        Err(PyErr::fetch(ob.py()))
                    } else {
                        Ok(::std::mem::transmute::<_, $rust_type>(buffer))
                    }
                }
            }
        }
    )
);

// manual implementation for 128bit integers
#[cfg(target_endian = "little")]
pub(super) const IS_LITTLE_ENDIAN: c_int = 1;
#[cfg(not(target_endian = "little"))]
pub(super) const IS_LITTLE_ENDIAN: c_int = 0;

#[cfg(test)]
mod test {
    use crate::conversion::ToPyObject;
    use crate::python::Python;
    use std;

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

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_i128_max() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = std::i128::MAX;
        let obj = v.to_object(py);
        assert_eq!(v, obj.extract::<i128>(py).unwrap());
        assert_eq!(v as u128, obj.extract::<u128>(py).unwrap());
        assert!(obj.extract::<u64>(py).is_err());
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_i128_min() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = std::i128::MIN;
        let obj = v.to_object(py);
        assert_eq!(v, obj.extract::<i128>(py).unwrap());
        assert!(obj.extract::<i64>(py).is_err());
        assert!(obj.extract::<u128>(py).is_err());
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_u128_max() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = std::u128::MAX;
        let obj = v.to_object(py);
        assert_eq!(v, obj.extract::<u128>(py).unwrap());
        assert!(obj.extract::<i128>(py).is_err());
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_u128_overflow() {
        use crate::ffi;
        use crate::object::PyObject;
        use crate::types::exceptions;
        use std::os::raw::c_uchar;
        let gil = Python::acquire_gil();
        let py = gil.python();
        let overflow_bytes: [c_uchar; 20] = [255; 20];
        unsafe {
            let obj = ffi::_PyLong_FromByteArray(
                overflow_bytes.as_ptr() as *const c_uchar,
                20,
                super::IS_LITTLE_ENDIAN,
                0,
            );
            let obj = PyObject::from_owned_ptr_or_panic(py, obj);
            let err = obj.extract::<u128>(py).unwrap_err();
            assert!(err.is_instance::<exceptions::OverflowError>(py));
        }
    }
}
