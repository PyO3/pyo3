#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::{type_hint_subscript, PyStaticExpr};
use crate::{
    conversion::{FromPyObject, FromPyObjectOwned, FromPyObjectSequence, IntoPyObject},
    exceptions::PyTypeError,
    ffi,
    types::{PyAnyMethods, PySequence, PyString},
    Borrowed, CastError, PyResult, PyTypeInfo,
};
use crate::{Bound, PyAny, PyErr, Python};

impl<'py, T> IntoPyObject<'py> for Vec<T>
where
    T: IntoPyObject<'py>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = T::SEQUENCE_OUTPUT_TYPE;

    /// Turns [`Vec<u8>`] into [`PyBytes`], all other `T`s will be turned into a [`PyList`]
    ///
    /// [`PyBytes`]: crate::types::PyBytes
    /// [`PyList`]: crate::types::PyList
    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        T::owned_sequence_into_pyobject(self, py, crate::conversion::private::Token)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::list_of(T::type_output())
    }
}

impl<'a, 'py, T> IntoPyObject<'py> for &'a Vec<T>
where
    &'a T: IntoPyObject<'py>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = <&[T]>::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        // NB: we could actually not cast to `PyAny`, which would be nice for
        // `&Vec<u8>`, but that'd be inconsistent with the `IntoPyObject` impl
        // above which always returns a `PyAny` for `Vec<T>`.
        self.as_slice().into_pyobject(py).map(Bound::into_any)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::list_of(<&T>::type_output())
    }
}

impl<'py, T> FromPyObject<'_, 'py> for Vec<T>
where
    T: FromPyObjectOwned<'py>,
{
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = type_hint_subscript!(PySequence::TYPE_HINT, T::INPUT_TYPE);

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        if let Some(extractor) = T::sequence_extractor(obj, crate::conversion::private::Token) {
            return Ok(extractor.to_vec());
        }

        if obj.is_instance_of::<PyString>() {
            return Err(PyTypeError::new_err("Can't extract `str` to `Vec`"));
        }

        extract_sequence(obj)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        TypeInfo::sequence_of(T::type_input())
    }
}

fn extract_sequence<'py, T>(obj: Borrowed<'_, 'py, PyAny>) -> PyResult<Vec<T>>
where
    T: FromPyObjectOwned<'py>,
{
    // Types that pass `PySequence_Check` usually implement enough of the sequence protocol
    // to support this function and if not, we will only fail extraction safely.
    let seq = unsafe {
        if ffi::PySequence_Check(obj.as_ptr()) != 0 {
            obj.cast_unchecked::<PySequence>()
        } else {
            return Err(CastError::new(obj, PySequence::type_object(obj.py()).into_any()).into());
        }
    };

    let mut v = Vec::with_capacity(seq.len().unwrap_or(0));
    for item in seq.try_iter()? {
        v.push(item?.extract::<T>().map_err(Into::into)?);
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use crate::conversion::IntoPyObject;
    use crate::types::{PyAnyMethods, PyBytes, PyBytesMethods, PyList};
    use crate::Python;

    #[test]
    fn test_vec_intopyobject_impl() {
        Python::attach(|py| {
            let bytes: Vec<u8> = b"foobar".to_vec();
            let obj = bytes.clone().into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyBytes>());
            let obj = obj.cast_into::<PyBytes>().unwrap();
            assert_eq!(obj.as_bytes(), &bytes);

            let nums: Vec<u16> = vec![0, 1, 2, 3];
            let obj = nums.into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyList>());
        });
    }

    #[test]
    fn test_vec_reference_intopyobject_impl() {
        Python::attach(|py| {
            let bytes: Vec<u8> = b"foobar".to_vec();
            let obj = (&bytes).into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyBytes>());
            let obj = obj.cast_into::<PyBytes>().unwrap();
            assert_eq!(obj.as_bytes(), &bytes);

            let nums: Vec<u16> = vec![0, 1, 2, 3];
            let obj = (&nums).into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyList>());
        });
    }

    #[test]
    fn test_strings_cannot_be_extracted_to_vec() {
        Python::attach(|py| {
            let v = "London Calling";
            let ob = v.into_pyobject(py).unwrap();

            assert!(ob.extract::<Vec<String>>().is_err());
            assert!(ob.extract::<Vec<char>>().is_err());
        });
    }

    #[test]
    fn test_extract_bytes_to_vec() {
        Python::attach(|py| {
            let v: Vec<u8> = PyBytes::new(py, b"abc").extract().unwrap();
            assert_eq!(v, b"abc");
        });
    }

    #[test]
    fn test_extract_tuple_to_vec() {
        Python::attach(|py| {
            let v: Vec<i32> = py.eval(c"(1, 2)", None, None).unwrap().extract().unwrap();
            assert_eq!(v, [1, 2]);
        });
    }

    #[test]
    fn test_extract_range_to_vec() {
        Python::attach(|py| {
            let v: Vec<i32> = py
                .eval(c"range(1, 5)", None, None)
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(v, [1, 2, 3, 4]);
        });
    }

    #[test]
    fn test_extract_bytearray_to_vec() {
        Python::attach(|py| {
            let v: Vec<u8> = py
                .eval(c"bytearray(b'abc')", None, None)
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(v, b"abc");
        });
    }
}
