use crate::conversion::AsPyPointer;
use crate::err::PyErr;
use crate::instance::PyNativeType;
use crate::types::PyString;
use crate::{ffi, PyAny, PyResult};

/// Computes the "repr" representation of obj.
///
/// This is equivalent to the Python expression `repr(obj)`.
pub fn repr(obj: &PyAny) -> PyResult<&PyString> {
    unsafe {
        obj.py()
            .from_owned_ptr_or_err(ffi::PyObject_Repr(obj.as_ptr()))
    }
}

/// Computes the "str" representation of obj.
///
/// This is equivalent to the Python expression `str(obj)`.
pub fn str(obj: &PyAny) -> PyResult<&PyString> {
    unsafe {
        obj.py()
            .from_owned_ptr_or_err(ffi::PyObject_Str(obj.as_ptr()))
    }
}

/// Retrieves the hash code of the object.
///
/// This is equivalent to the Python expression `hash(obi)`.
pub fn hash(obj: &PyAny) -> PyResult<isize> {
    let v = unsafe { ffi::PyObject_Hash(obj.as_ptr()) };
    if v == -1 {
        Err(PyErr::fetch(obj.py()))
    } else {
        Ok(v)
    }
}

/// Returns the length of the sequence or mapping.
///
/// This is equivalent to the Python expression `len(obj)`.
pub fn len(obj: &PyAny) -> PyResult<usize> {
    let v = unsafe { ffi::PyObject_Size(obj.as_ptr()) };
    if v == -1 {
        Err(PyErr::fetch(obj.py()))
    } else {
        Ok(v as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PyList;
    use crate::Python;

    #[test]
    fn test_list_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let list = PyList::new(py, &[1, 2, 3]);
        assert_eq!(len(list).unwrap(), 3);
    }
}
