#![cfg(feature = "smallvec")]

//!  Conversions to and from [smallvec](https://docs.rs/smallvec/).
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! # change * to the latest versions
//! smallvec = "*"
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"smallvec\"] }")]
//! ```
//!
//! Note that you must use compatible versions of smallvec and PyO3.
//! The required smallvec version may vary based on the version of PyO3.
use crate::conversion::{FromPyObjectOwned, IntoPyObject};
use crate::exceptions::PyTypeError;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::PyStaticExpr;
use crate::types::any::PyAnyMethods;
use crate::types::{PySequence, PyString};
use crate::{
    err::CastError, ffi, Borrowed, Bound, FromPyObject, PyAny, PyErr, PyResult, PyTypeInfo, Python,
};
use smallvec::{Array, SmallVec};

impl<'py, A> IntoPyObject<'py> for SmallVec<A>
where
    A: Array,
    A::Item: IntoPyObject<'py>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = A::Item::SEQUENCE_OUTPUT_TYPE;

    /// Turns [`SmallVec<u8>`] into [`PyBytes`], all other `T`s will be turned into a [`PyList`]
    ///
    /// [`PyBytes`]: crate::types::PyBytes
    /// [`PyList`]: crate::types::PyList
    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        <A::Item>::owned_sequence_into_pyobject(self, py, crate::conversion::private::Token)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::list_of(A::Item::type_output())
    }
}

impl<'a, 'py, A> IntoPyObject<'py> for &'a SmallVec<A>
where
    A: Array,
    &'a A::Item: IntoPyObject<'py>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.as_slice().into_pyobject(py)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::list_of(<&A::Item>::type_output())
    }
}

impl<'py, A> FromPyObject<'_, 'py> for SmallVec<A>
where
    A: Array,
    A::Item: FromPyObjectOwned<'py>,
{
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if obj.is_instance_of::<PyString>() {
            return Err(PyTypeError::new_err("Can't extract `str` to `SmallVec`"));
        }
        extract_sequence(obj)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        TypeInfo::sequence_of(A::Item::type_input())
    }
}

fn extract_sequence<'py, A>(obj: Borrowed<'_, 'py, PyAny>) -> PyResult<SmallVec<A>>
where
    A: Array,
    A::Item: FromPyObjectOwned<'py>,
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

    let mut sv = SmallVec::with_capacity(seq.len().unwrap_or(0));
    for item in seq.try_iter()? {
        sv.push(item?.extract::<A::Item>().map_err(Into::into)?);
    }
    Ok(sv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PyBytes, PyBytesMethods, PyDict, PyList};

    #[test]
    fn test_smallvec_from_py_object() {
        Python::attach(|py| {
            let l = PyList::new(py, [1, 2, 3, 4, 5]).unwrap();
            let sv: SmallVec<[u64; 8]> = l.extract().unwrap();
            assert_eq!(sv.as_slice(), [1, 2, 3, 4, 5]);
        });
    }

    #[test]
    fn test_smallvec_from_py_object_fails() {
        Python::attach(|py| {
            let dict = PyDict::new(py);
            let sv: PyResult<SmallVec<[u64; 8]>> = dict.extract();
            assert_eq!(
                sv.unwrap_err().to_string(),
                "TypeError: 'dict' object is not an instance of 'Sequence'"
            );
        });
    }

    #[test]
    fn test_smallvec_into_pyobject() {
        Python::attach(|py| {
            let sv: SmallVec<[u64; 8]> = [1, 2, 3, 4, 5].iter().cloned().collect();
            let hso = sv.into_pyobject(py).unwrap();
            let l = PyList::new(py, [1, 2, 3, 4, 5]).unwrap();
            assert!(l.eq(hso).unwrap());
        });
    }

    #[test]
    fn test_smallvec_intopyobject_impl() {
        Python::attach(|py| {
            let bytes: SmallVec<[u8; 8]> = [1, 2, 3, 4, 5].iter().cloned().collect();
            let obj = bytes.clone().into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyBytes>());
            let obj = obj.cast_into::<PyBytes>().unwrap();
            assert_eq!(obj.as_bytes(), &*bytes);

            let nums: SmallVec<[u16; 8]> = [1, 2, 3, 4, 5].iter().cloned().collect();
            let obj = nums.into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyList>());
        });
    }
}
