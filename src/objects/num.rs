use libc::{c_long, c_double};
use std;
use python::{Python, PythonObject, ToPythonPointer};
use err::{self, PyResult, PyErr};
use super::object::PyObject;
use super::exc;
use ffi::{self, Py_ssize_t};
use conversion::{ToPyObject, FromPyObject};

pyobject_newtype!(PyInt, PyInt_Check, PyInt_Type);
pyobject_newtype!(PyLong, PyLong_Check, PyLong_Type);
pyobject_newtype!(PyFloat, PyFloat_Check, PyFloat_Type);

impl <'p> PyInt<'p> {
    /// Creates a new python `int` object.
    pub fn new(py: Python<'p>, val: c_long) -> PyInt<'p> {
        unsafe {
            err::cast_from_owned_ptr_or_panic(py, ffi::PyInt_FromLong(val))
        }
    }

    /// Gets the value of this integer.
    pub fn value(&self) -> c_long {
        unsafe { ffi::PyInt_AS_LONG(self.as_ptr()) }
    }
}


impl <'p> PyFloat<'p> {
    /// Creates a new python `float` object.
    pub fn new(py: Python<'p>, val: c_double) -> PyFloat<'p> {
        unsafe {
            err::cast_from_owned_ptr_or_panic(py, ffi::PyFloat_FromDouble(val))
        }
    }

    /// Gets the value of this float.
    pub fn value(&self) -> c_double {
        unsafe { ffi::PyFloat_AS_DOUBLE(self.as_ptr()) }
    }
}

macro_rules! int_fits_c_long(
    ($rust_type:ty) => (
        impl <'p> ToPyObject<'p> for $rust_type {
            type ObjectType = PyInt<'p>;

            fn to_py_object(&self, py: Python<'p>) -> PyResult<'p, PyInt<'p>> {
                unsafe {
                    Ok(try!(err::result_from_owned_ptr(py,
                        ffi::PyInt_FromLong(*self as c_long))).unchecked_cast_into::<PyInt>())
                }
            }
        }

        impl <'p, 's> FromPyObject<'p, 's> for $rust_type {
            fn from_py_object(s: &'s PyObject<'p>) -> PyResult<'p, $rust_type> {
                let py = s.python();
                let val = unsafe { ffi::PyInt_AsLong(s.as_ptr()) };
                if val == -1 && PyErr::occurred(py) {
                    return Err(PyErr::fetch(py));
                }
                match std::num::cast::<c_long, $rust_type>(val) {
                    Some(v) => Ok(v),
                    None => Err(overflow_error(py))
                }
            }
        }
    )
);


macro_rules! int_fits_larger_int(
    ($rust_type:ty, $larger_type:ty) => (
        impl <'p> ToPyObject<'p> for $rust_type {
            type ObjectType = <$larger_type as ToPyObject<'p>>::ObjectType;

            #[inline]
            fn to_py_object(&self, py: Python<'p>) -> PyResult<'p, <$larger_type as ToPyObject<'p>>::ObjectType> {
                (*self as $larger_type).to_py_object(py)
            }
        }

        impl <'p, 's> FromPyObject<'p, 's> for $rust_type {
            fn from_py_object(s: &'s PyObject<'p>) -> PyResult<'p, $rust_type> {
                let py = s.python();
                let val = try!(s.extract::<$larger_type>());
                match std::num::cast::<$larger_type, $rust_type>(val) {
                    Some(v) => Ok(v),
                    None => Err(overflow_error(py))
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
// TODO: manual implementation for i64 on systems with 32-bit long

// u64 has a manual implementation as it never fits into signed long

#[cfg(all(target_pointer_width="64", not(target_os="windows")))]
int_fits_c_long!(isize);
#[cfg(any(target_pointer_width="32", target_os="windows"))]
int_fits_larger_int!(isize, i64);

int_fits_larger_int!(usize, u64);

impl <'p> ToPyObject<'p> for u64 {
    type ObjectType = PyObject<'p>;

    fn to_py_object(&self, py: Python<'p>) -> PyResult<'p, PyObject<'p>> {
        unsafe {
            let ptr = match std::num::cast::<u64, c_long>(*self) {
                Some(v) => ffi::PyInt_FromLong(v),
                None => ffi::PyLong_FromUnsignedLongLong(*self)
            };
            err::result_from_owned_ptr(py, ptr)
        }
    }
}

fn pylong_as_u64<'p>(obj: &PyObject<'p>) -> PyResult<'p, u64> {
    let py = obj.python();
    let v = unsafe { ffi::PyLong_AsUnsignedLongLong(obj.as_ptr()) };
    if v == !0 && PyErr::occurred(py) {
        Err(PyErr::fetch(py))
    } else {
        Ok(v)
    }
}

impl <'p, 's> FromPyObject<'p, 's> for u64 {
    fn from_py_object(s: &'s PyObject<'p>) -> PyResult<'p, u64> {
        let py = s.python();
        let ptr = s.as_ptr();
        unsafe {
            if ffi::PyLong_Check(ptr) != 0 {
                pylong_as_u64(s)
            } else if ffi::PyInt_Check(ptr) != 0 {
                match std::num::cast::<c_long, u64>(ffi::PyInt_AS_LONG(ptr)) {
                    Some(v) => Ok(v),
                    None => Err(overflow_error(py))
                }
            } else {
                let num = try!(err::result_from_owned_ptr(py, ffi::PyNumber_Long(ptr)));
                pylong_as_u64(&num)
            }
        }
    }
}

impl <'p> ToPyObject<'p> for f64 {
    type ObjectType = PyFloat<'p>;

    fn to_py_object(&self, py: Python<'p>) -> PyResult<'p, PyFloat<'p>> {
       Ok(PyFloat::new(py, *self))
    }
}

impl <'p, 's> FromPyObject<'p, 's> for f64 {
    fn from_py_object(s: &'s PyObject<'p>) -> PyResult<'p, f64> {
        let py = s.python();
        let v = unsafe { ffi::PyFloat_AsDouble(s.as_ptr()) };
        if v == -1.0 && PyErr::occurred(py) {
            Err(PyErr::fetch(py))
        } else {
            Ok(v)
        }
    }
}

fn overflow_error(py: Python) -> PyErr {
    PyErr::new_lazy_init(py.get_type::<exc::OverflowError>(), None)
}

impl <'p> ToPyObject<'p> for f32 {
    type ObjectType = PyFloat<'p>;

    fn to_py_object(&self, py: Python<'p>) -> PyResult<'p, PyFloat<'p>> {
       Ok(PyFloat::new(py, *self as f64))
    }
}

impl <'p, 's> FromPyObject<'p, 's> for f32 {
    fn from_py_object(s: &'s PyObject<'p>) -> PyResult<'p, f32> {
        Ok(try!(s.extract::<f64>()) as f32)
    }
}

#[cfg(test)]
mod test {
    use std;
    use python::{Python, PythonObject};
    use conversion::ToPyObject;

    macro_rules! num_to_py_object_and_back (
        ($func_name:ident, $t1:ty, $t2:ty) => (
            #[test]
            fn $func_name() {
                let gil = Python::acquire_gil();
                let py = gil.python();
                let val = 123 as $t1;
                let obj = val.to_py_object(py).unwrap().into_object();
                assert_eq!(obj.extract::<$t2>().unwrap(), val as $t2);
            }
        )
    );

    num_to_py_object_and_back!(to_from_f64, f64, f64);
    num_to_py_object_and_back!(to_from_f32, f32, f32);
    num_to_py_object_and_back!(to_from_i8,   i8,  i8);
    num_to_py_object_and_back!(to_from_u8,   u8,  u8);
    num_to_py_object_and_back!(to_from_i16, i16, i16);
    num_to_py_object_and_back!(to_from_u16, u16, u16);
    num_to_py_object_and_back!(to_from_i32, i32, i32);
    num_to_py_object_and_back!(to_from_u32, u32, u32);
    num_to_py_object_and_back!(to_from_i64, i64, i64);
    num_to_py_object_and_back!(to_from_u64, u64, u64);
    num_to_py_object_and_back!(to_from_isize, isize, isize);
    num_to_py_object_and_back!(to_from_usize, usize, usize);
    num_to_py_object_and_back!(float_to_i32, f64, i32);
    num_to_py_object_and_back!(float_to_u32, f64, u32);
    num_to_py_object_and_back!(float_to_i64, f64, i64);
    num_to_py_object_and_back!(float_to_u64, f64, u64);
    num_to_py_object_and_back!(int_to_float, i32, f64);

    #[test]
    fn test_u32_max() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = std::u32::MAX;
        let obj = v.to_py_object(py).unwrap().into_object();
        assert_eq!(v, obj.extract::<u32>().unwrap());
        assert_eq!(v as u64, obj.extract::<u64>().unwrap());
        assert!(obj.extract::<i32>().is_err());
    }

    #[test]
    fn test_u64_max() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = std::u64::MAX;
        let obj = v.to_py_object(py).unwrap().into_object();
        println!("{:?}", obj);
        assert_eq!(v, obj.extract::<u64>().unwrap());
        assert!(obj.extract::<i64>().is_err());
    }
}

