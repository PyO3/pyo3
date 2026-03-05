use crate::conversion::private::Reference;
use crate::conversion::{FromPyObjectSequence, IntoPyObject};
use crate::ffi_ptr_ext::FfiPtrExt;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::PyStaticExpr;
use crate::py_result_ext::PyResultExt;
#[cfg(feature = "experimental-inspect")]
use crate::type_object::PyTypeInfo;
use crate::types::{PyByteArray, PyByteArrayMethods, PyBytes, PyInt};
use crate::{exceptions, ffi, Borrowed, Bound, FromPyObject, PyAny, PyErr, PyResult, Python};
use std::convert::Infallible;
use std::ffi::c_long;
use std::mem::MaybeUninit;
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};

use super::array::invalid_sequence_length;

macro_rules! int_fits_larger_int {
    ($rust_type:ty, $larger_type:ty) => {
        impl<'py> IntoPyObject<'py> for $rust_type {
            type Target = PyInt;
            type Output = Bound<'py, Self::Target>;
            type Error = Infallible;

            #[cfg(feature = "experimental-inspect")]
            const OUTPUT_TYPE: PyStaticExpr = <$larger_type>::OUTPUT_TYPE;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                (self as $larger_type).into_pyobject(py)
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_output() -> TypeInfo {
                <$larger_type>::type_output()
            }
        }

        impl<'py> IntoPyObject<'py> for &$rust_type {
            type Target = PyInt;
            type Output = Bound<'py, Self::Target>;
            type Error = Infallible;

            #[cfg(feature = "experimental-inspect")]
            const OUTPUT_TYPE: PyStaticExpr = <$larger_type>::OUTPUT_TYPE;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                (*self).into_pyobject(py)
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_output() -> TypeInfo {
                <$larger_type>::type_output()
            }
        }

        impl FromPyObject<'_, '_> for $rust_type {
            type Error = PyErr;

            #[cfg(feature = "experimental-inspect")]
            const INPUT_TYPE: PyStaticExpr = <$larger_type>::INPUT_TYPE;

            fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
                let val: $larger_type = obj.extract()?;
                <$rust_type>::try_from(val)
                    .map_err(|e| exceptions::PyOverflowError::new_err(e.to_string()))
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_input() -> TypeInfo {
                <$larger_type>::type_input()
            }
        }
    };
}

macro_rules! extract_int {
    ($obj:ident, $error_val:expr, $pylong_as:expr) => {
        extract_int!($obj, $error_val, $pylong_as, false)
    };

    ($obj:ident, $error_val:expr, $pylong_as:expr, $force_index_call: literal) => {
        // In python 3.8+ `PyLong_AsLong` and friends takes care of calling `PyNumber_Index`,
        // however 3.8 & 3.9 do lossy conversion of floats, hence we only use the
        // simplest logic for 3.10+ where that was fixed - python/cpython#82180.
        // `PyLong_AsUnsignedLongLong` does not call `PyNumber_Index`, hence the `force_index_call` argument
        // See https://github.com/PyO3/pyo3/pull/3742 for details
        if cfg!(Py_3_10) && !$force_index_call {
            err_if_invalid_value($obj.py(), $error_val, unsafe { $pylong_as($obj.as_ptr()) })
        } else if let Ok(long) = $obj.cast::<crate::types::PyInt>() {
            // fast path - checking for subclass of `int` just checks a bit in the type $object
            err_if_invalid_value($obj.py(), $error_val, unsafe { $pylong_as(long.as_ptr()) })
        } else {
            unsafe {
                let num = nb_index(&$obj)?;
                err_if_invalid_value($obj.py(), $error_val, $pylong_as(num.as_ptr()))
            }
        }
    };
}

macro_rules! int_convert_u64_or_i64 {
    ($rust_type:ty, $pylong_from_ll_or_ull:expr, $pylong_as_ll_or_ull:expr, $force_index_call:literal) => {
        impl<'py> IntoPyObject<'py> for $rust_type {
            type Target = PyInt;
            type Output = Bound<'py, Self::Target>;
            type Error = Infallible;

            #[cfg(feature = "experimental-inspect")]
            const OUTPUT_TYPE: PyStaticExpr = PyInt::TYPE_HINT;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                unsafe {
                    Ok($pylong_from_ll_or_ull(self)
                        .assume_owned(py)
                        .cast_into_unchecked())
                }
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_output() -> TypeInfo {
                TypeInfo::builtin("int")
            }
        }
        impl<'py> IntoPyObject<'py> for &$rust_type {
            type Target = PyInt;
            type Output = Bound<'py, Self::Target>;
            type Error = Infallible;

            #[cfg(feature = "experimental-inspect")]
            const OUTPUT_TYPE: PyStaticExpr = <$rust_type>::OUTPUT_TYPE;

            #[inline]
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                (*self).into_pyobject(py)
            }
        }
        impl FromPyObject<'_, '_> for $rust_type {
            type Error = PyErr;

            #[cfg(feature = "experimental-inspect")]
            const INPUT_TYPE: PyStaticExpr = PyInt::TYPE_HINT;

            fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<$rust_type, Self::Error> {
                extract_int!(obj, !0, $pylong_as_ll_or_ull, $force_index_call)
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_input() -> TypeInfo {
                Self::type_output()
            }
        }
    };
}

macro_rules! int_fits_c_long {
    ($rust_type:ty) => {
        impl<'py> IntoPyObject<'py> for $rust_type {
            type Target = PyInt;
            type Output = Bound<'py, Self::Target>;
            type Error = Infallible;

            #[cfg(feature = "experimental-inspect")]
            const OUTPUT_TYPE: PyStaticExpr = PyInt::TYPE_HINT;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                unsafe {
                    Ok(ffi::PyLong_FromLong(self as c_long)
                        .assume_owned(py)
                        .cast_into_unchecked())
                }
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_output() -> TypeInfo {
                TypeInfo::builtin("int")
            }
        }

        impl<'py> IntoPyObject<'py> for &$rust_type {
            type Target = PyInt;
            type Output = Bound<'py, Self::Target>;
            type Error = Infallible;

            #[cfg(feature = "experimental-inspect")]
            const OUTPUT_TYPE: PyStaticExpr = <$rust_type>::OUTPUT_TYPE;

            #[inline]
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                (*self).into_pyobject(py)
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_output() -> TypeInfo {
                TypeInfo::builtin("int")
            }
        }

        impl<'py> FromPyObject<'_, 'py> for $rust_type {
            type Error = PyErr;

            #[cfg(feature = "experimental-inspect")]
            const INPUT_TYPE: PyStaticExpr = PyInt::TYPE_HINT;

            fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
                let val: c_long = extract_int!(obj, -1, ffi::PyLong_AsLong)?;
                <$rust_type>::try_from(val)
                    .map_err(|e| exceptions::PyOverflowError::new_err(e.to_string()))
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_input() -> TypeInfo {
                Self::type_output()
            }
        }
    };
}

impl<'py> IntoPyObject<'py> for u8 {
    type Target = PyInt;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = PyInt::TYPE_HINT;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        unsafe {
            Ok(ffi::PyLong_FromLong(self as c_long)
                .assume_owned(py)
                .cast_into_unchecked())
        }
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("int")
    }

    #[inline]
    fn owned_sequence_into_pyobject<I>(
        iter: I,
        py: Python<'py>,
        _: crate::conversion::private::Token,
    ) -> Result<Bound<'py, PyAny>, PyErr>
    where
        I: AsRef<[u8]>,
    {
        Ok(PyBytes::new(py, iter.as_ref()).into_any())
    }

    #[cfg(feature = "experimental-inspect")]
    const SEQUENCE_OUTPUT_TYPE: PyStaticExpr = PyBytes::TYPE_HINT;
}

impl<'py> IntoPyObject<'py> for &'_ u8 {
    type Target = PyInt;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = u8::OUTPUT_TYPE;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        u8::into_pyobject(*self, py)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("int")
    }

    #[inline]
    fn borrowed_sequence_into_pyobject<I>(
        iter: I,
        py: Python<'py>,
        _: crate::conversion::private::Token,
    ) -> Result<Bound<'py, PyAny>, PyErr>
    where
        // I: AsRef<[u8]>, but the compiler needs it expressed via the trait for some reason
        I: AsRef<[<Self as Reference>::BaseType]>,
    {
        Ok(PyBytes::new(py, iter.as_ref()).into_any())
    }

    #[cfg(feature = "experimental-inspect")]
    const SEQUENCE_OUTPUT_TYPE: PyStaticExpr = PyBytes::TYPE_HINT;
}

impl<'py> FromPyObject<'_, 'py> for u8 {
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = PyInt::TYPE_HINT;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        let val: c_long = extract_int!(obj, -1, ffi::PyLong_AsLong)?;
        u8::try_from(val).map_err(|e| exceptions::PyOverflowError::new_err(e.to_string()))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        Self::type_output()
    }

    #[inline]
    fn sequence_extractor(
        obj: Borrowed<'_, 'py, PyAny>,
        _: crate::conversion::private::Token,
    ) -> Option<impl FromPyObjectSequence<Target = u8>> {
        if let Ok(bytes) = obj.cast::<PyBytes>() {
            Some(BytesSequenceExtractor::Bytes(bytes))
        } else if let Ok(byte_array) = obj.cast::<PyByteArray>() {
            Some(BytesSequenceExtractor::ByteArray(byte_array))
        } else {
            None
        }
    }
}

pub(crate) enum BytesSequenceExtractor<'a, 'py> {
    Bytes(Borrowed<'a, 'py, PyBytes>),
    ByteArray(Borrowed<'a, 'py, PyByteArray>),
}

impl BytesSequenceExtractor<'_, '_> {
    fn fill_slice(&self, out: &mut [MaybeUninit<u8>]) -> PyResult<()> {
        let mut copy_slice = |slice: &[u8]| {
            if slice.len() != out.len() {
                return Err(invalid_sequence_length(out.len(), slice.len()));
            }
            // Safety: `slice` and `out` are guaranteed not to overlap due to `&mut` reference on `out`.
            unsafe {
                std::ptr::copy_nonoverlapping(slice.as_ptr(), out.as_mut_ptr().cast(), out.len())
            };
            Ok(())
        };

        match self {
            BytesSequenceExtractor::Bytes(b) => copy_slice(b.as_bytes()),
            BytesSequenceExtractor::ByteArray(b) => {
                crate::sync::critical_section::with_critical_section(b, || {
                    // Safety: b is protected by a critical section
                    copy_slice(unsafe { b.as_bytes() })
                })
            }
        }
    }
}

impl FromPyObjectSequence for BytesSequenceExtractor<'_, '_> {
    type Target = u8;

    fn to_vec(&self) -> Vec<Self::Target> {
        match self {
            BytesSequenceExtractor::Bytes(b) => b.as_bytes().to_vec(),
            BytesSequenceExtractor::ByteArray(b) => b.to_vec(),
        }
    }

    fn to_array<const N: usize>(&self) -> PyResult<[u8; N]> {
        let mut out: MaybeUninit<[u8; N]> = MaybeUninit::uninit();

        // Safety: `[u8; N]` has the same layout as `[MaybeUninit<u8>; N]`
        let slice = unsafe {
            std::slice::from_raw_parts_mut(out.as_mut_ptr().cast::<MaybeUninit<u8>>(), N)
        };

        self.fill_slice(slice)?;

        // Safety: `out` is fully initialized
        Ok(unsafe { out.assume_init() })
    }
}

int_fits_c_long!(i8);
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
int_convert_u64_or_i64!(i64, ffi::PyLong_FromLongLong, ffi::PyLong_AsLongLong, false);

#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
int_fits_c_long!(isize);
#[cfg(any(target_pointer_width = "32", target_os = "windows"))]
int_fits_larger_int!(isize, i64);

int_fits_larger_int!(usize, u64);

// u64 has a manual implementation as it never fits into signed long
int_convert_u64_or_i64!(
    u64,
    ffi::PyLong_FromUnsignedLongLong,
    ffi::PyLong_AsUnsignedLongLong,
    true
);

#[cfg(not(Py_LIMITED_API))]
mod fast_128bit_int_conversion {
    use super::*;

    // for 128bit Integers
    macro_rules! int_convert_128 {
        ($rust_type: ty, $is_signed: literal) => {
            impl<'py> IntoPyObject<'py> for $rust_type {
                type Target = PyInt;
                type Output = Bound<'py, Self::Target>;
                type Error = Infallible;

                #[cfg(feature = "experimental-inspect")]
                const OUTPUT_TYPE: PyStaticExpr = PyInt::TYPE_HINT;

                fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                    #[cfg(Py_3_13)]
                    {
                        let bytes = self.to_ne_bytes();
                        Ok(int_from_ne_bytes::<{ $is_signed }>(py, &bytes))
                    }
                    #[cfg(not(Py_3_13))]
                    {
                        let bytes = self.to_le_bytes();
                        Ok(int_from_le_bytes::<{ $is_signed }>(py, &bytes))
                    }
                }

                #[cfg(feature = "experimental-inspect")]
                fn type_output() -> TypeInfo {
                    TypeInfo::builtin("int")
                }
            }

            impl<'py> IntoPyObject<'py> for &$rust_type {
                type Target = PyInt;
                type Output = Bound<'py, Self::Target>;
                type Error = Infallible;

                #[cfg(feature = "experimental-inspect")]
                const OUTPUT_TYPE: PyStaticExpr = <$rust_type>::OUTPUT_TYPE;

                #[inline]
                fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                    (*self).into_pyobject(py)
                }

                #[cfg(feature = "experimental-inspect")]
                fn type_output() -> TypeInfo {
                    TypeInfo::builtin("int")
                }
            }

            impl FromPyObject<'_, '_> for $rust_type {
                type Error = PyErr;

                #[cfg(feature = "experimental-inspect")]
                const INPUT_TYPE: PyStaticExpr = PyInt::TYPE_HINT;

                fn extract(ob: Borrowed<'_, '_, PyAny>) -> Result<$rust_type, Self::Error> {
                    let num = nb_index(&ob)?;
                    let mut buffer = [0u8; std::mem::size_of::<$rust_type>()];
                    #[cfg(not(Py_3_13))]
                    {
                        crate::err::error_on_minusone(ob.py(), unsafe {
                            ffi::_PyLong_AsByteArray(
                                num.as_ptr() as *mut ffi::PyLongObject,
                                buffer.as_mut_ptr(),
                                buffer.len(),
                                1,
                                $is_signed.into(),
                            )
                        })?;
                        Ok(<$rust_type>::from_le_bytes(buffer))
                    }
                    #[cfg(Py_3_13)]
                    {
                        let mut flags = ffi::Py_ASNATIVEBYTES_NATIVE_ENDIAN;
                        if !$is_signed {
                            flags |= ffi::Py_ASNATIVEBYTES_UNSIGNED_BUFFER
                                | ffi::Py_ASNATIVEBYTES_REJECT_NEGATIVE;
                        }
                        let actual_size: usize = unsafe {
                            ffi::PyLong_AsNativeBytes(
                                num.as_ptr(),
                                buffer.as_mut_ptr().cast(),
                                buffer
                                    .len()
                                    .try_into()
                                    .expect("length of buffer fits in Py_ssize_t"),
                                flags,
                            )
                        }
                        .try_into()
                        .map_err(|_| PyErr::fetch(ob.py()))?;
                        if actual_size as usize > buffer.len() {
                            return Err(crate::exceptions::PyOverflowError::new_err(
                                "Python int larger than 128 bits",
                            ));
                        }
                        Ok(<$rust_type>::from_ne_bytes(buffer))
                    }
                }

                #[cfg(feature = "experimental-inspect")]
                fn type_input() -> TypeInfo {
                    Self::type_output()
                }
            }
        };
    }

    int_convert_128!(i128, true);
    int_convert_128!(u128, false);
}

#[cfg(all(not(Py_LIMITED_API), not(Py_3_13)))]
pub(crate) fn int_from_le_bytes<'py, const IS_SIGNED: bool>(
    py: Python<'py>,
    bytes: &[u8],
) -> Bound<'py, PyInt> {
    unsafe {
        ffi::_PyLong_FromByteArray(bytes.as_ptr().cast(), bytes.len(), 1, IS_SIGNED.into())
            .assume_owned(py)
            .cast_into_unchecked()
    }
}

#[cfg(all(Py_3_13, not(Py_LIMITED_API)))]
pub(crate) fn int_from_ne_bytes<'py, const IS_SIGNED: bool>(
    py: Python<'py>,
    bytes: &[u8],
) -> Bound<'py, PyInt> {
    let flags = if IS_SIGNED {
        ffi::Py_ASNATIVEBYTES_NATIVE_ENDIAN
    } else {
        ffi::Py_ASNATIVEBYTES_NATIVE_ENDIAN | ffi::Py_ASNATIVEBYTES_UNSIGNED_BUFFER
    };
    unsafe {
        ffi::PyLong_FromNativeBytes(bytes.as_ptr().cast(), bytes.len(), flags)
            .assume_owned(py)
            .cast_into_unchecked()
    }
}

pub(crate) fn nb_index<'py>(obj: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyInt>> {
    // SAFETY: PyNumber_Index returns a new reference or NULL on error
    unsafe { ffi::PyNumber_Index(obj.as_ptr()).assume_owned_or_err(obj.py()) }.cast_into()
}

// For ABI3 we implement the conversion manually.
#[cfg(Py_LIMITED_API)]
mod slow_128bit_int_conversion {
    use super::*;
    use crate::types::any::PyAnyMethods as _;
    const SHIFT: usize = 64;

    // for 128bit Integers
    macro_rules! int_convert_128 {
        ($rust_type: ty, $half_type: ty) => {
            impl<'py> IntoPyObject<'py> for $rust_type {
                type Target = PyInt;
                type Output = Bound<'py, Self::Target>;
                type Error = Infallible;

                #[cfg(feature = "experimental-inspect")]
                const OUTPUT_TYPE: PyStaticExpr = PyInt::TYPE_HINT;

                fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                    let lower = (self as u64).into_pyobject(py)?;
                    let upper = ((self >> SHIFT) as $half_type).into_pyobject(py)?;
                    let shift = SHIFT.into_pyobject(py)?;
                    unsafe {
                        let shifted =
                            ffi::PyNumber_Lshift(upper.as_ptr(), shift.as_ptr()).assume_owned(py);

                        Ok(ffi::PyNumber_Or(shifted.as_ptr(), lower.as_ptr())
                            .assume_owned(py)
                            .cast_into_unchecked())
                    }
                }

                #[cfg(feature = "experimental-inspect")]
                fn type_output() -> TypeInfo {
                    TypeInfo::builtin("int")
                }
            }

            impl<'py> IntoPyObject<'py> for &$rust_type {
                type Target = PyInt;
                type Output = Bound<'py, Self::Target>;
                type Error = Infallible;

                #[cfg(feature = "experimental-inspect")]
                const OUTPUT_TYPE: PyStaticExpr = <$rust_type>::OUTPUT_TYPE;

                #[inline]
                fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                    (*self).into_pyobject(py)
                }

                #[cfg(feature = "experimental-inspect")]
                fn type_output() -> TypeInfo {
                    TypeInfo::builtin("int")
                }
            }

            impl FromPyObject<'_, '_> for $rust_type {
                type Error = PyErr;

                #[cfg(feature = "experimental-inspect")]
                const INPUT_TYPE: PyStaticExpr = PyInt::TYPE_HINT;

                fn extract(ob: Borrowed<'_, '_, PyAny>) -> Result<$rust_type, Self::Error> {
                    let py = ob.py();
                    unsafe {
                        let lower = err_if_invalid_value(
                            py,
                            -1 as _,
                            ffi::PyLong_AsUnsignedLongLongMask(ob.as_ptr()),
                        )? as $rust_type;
                        let shift = SHIFT.into_pyobject(py)?;
                        let shifted = Bound::from_owned_ptr_or_err(
                            py,
                            ffi::PyNumber_Rshift(ob.as_ptr(), shift.as_ptr()),
                        )?;
                        let upper: $half_type = shifted.extract()?;
                        Ok((<$rust_type>::from(upper) << SHIFT) | lower)
                    }
                }

                #[cfg(feature = "experimental-inspect")]
                fn type_input() -> TypeInfo {
                    Self::type_output()
                }
            }
        };
    }

    int_convert_128!(i128, i64);
    int_convert_128!(u128, u64);
}

fn err_if_invalid_value<T: PartialEq>(
    py: Python<'_>,
    invalid_value: T,
    actual_value: T,
) -> PyResult<T> {
    if actual_value == invalid_value {
        if let Some(err) = PyErr::take(py) {
            return Err(err);
        }
    }

    Ok(actual_value)
}

macro_rules! nonzero_int_impl {
    ($nonzero_type:ty, $primitive_type:ty) => {
        impl<'py> IntoPyObject<'py> for $nonzero_type {
            type Target = PyInt;
            type Output = Bound<'py, Self::Target>;
            type Error = Infallible;

            #[cfg(feature = "experimental-inspect")]
            const OUTPUT_TYPE: PyStaticExpr = PyInt::TYPE_HINT;

            #[inline]
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                self.get().into_pyobject(py)
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_output() -> TypeInfo {
                TypeInfo::builtin("int")
            }
        }

        impl<'py> IntoPyObject<'py> for &$nonzero_type {
            type Target = PyInt;
            type Output = Bound<'py, Self::Target>;
            type Error = Infallible;

            #[cfg(feature = "experimental-inspect")]
            const OUTPUT_TYPE: PyStaticExpr = <$nonzero_type>::OUTPUT_TYPE;

            #[inline]
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                (*self).into_pyobject(py)
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_output() -> TypeInfo {
                TypeInfo::builtin("int")
            }
        }

        impl FromPyObject<'_, '_> for $nonzero_type {
            type Error = PyErr;

            #[cfg(feature = "experimental-inspect")]
            const INPUT_TYPE: PyStaticExpr = <$primitive_type>::INPUT_TYPE;

            fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
                let val: $primitive_type = obj.extract()?;
                <$nonzero_type>::try_from(val)
                    .map_err(|_| exceptions::PyValueError::new_err("invalid zero value"))
            }

            #[cfg(feature = "experimental-inspect")]
            fn type_input() -> TypeInfo {
                <$primitive_type>::type_input()
            }
        }
    };
}

nonzero_int_impl!(NonZeroI8, i8);
nonzero_int_impl!(NonZeroI16, i16);
nonzero_int_impl!(NonZeroI32, i32);
nonzero_int_impl!(NonZeroI64, i64);
nonzero_int_impl!(NonZeroI128, i128);
nonzero_int_impl!(NonZeroIsize, isize);
nonzero_int_impl!(NonZeroU8, u8);
nonzero_int_impl!(NonZeroU16, u16);
nonzero_int_impl!(NonZeroU32, u32);
nonzero_int_impl!(NonZeroU64, u64);
nonzero_int_impl!(NonZeroU128, u128);
nonzero_int_impl!(NonZeroUsize, usize);

#[cfg(test)]
mod test_128bit_integers {
    use super::*;
    use crate::types::PyAnyMethods;

    #[cfg(not(target_arch = "wasm32"))]
    use crate::types::PyDict;

    #[cfg(not(target_arch = "wasm32"))]
    use crate::types::dict::PyDictMethods;

    #[cfg(not(target_arch = "wasm32"))]
    use proptest::prelude::*;

    #[cfg(not(target_arch = "wasm32"))]
    use std::ffi::CString;

    #[cfg(not(target_arch = "wasm32"))]
    proptest! {
        #[test]
        fn test_i128_roundtrip(x: i128) {
            Python::attach(|py| {
                let x_py = x.into_pyobject(py).unwrap();
                let locals = PyDict::new(py);
                locals.set_item("x_py", &x_py).unwrap();
                py.run(&CString::new(format!("assert x_py == {x}")).unwrap(), None, Some(&locals)).unwrap();
                let roundtripped: i128 = x_py.extract().unwrap();
                assert_eq!(x, roundtripped);
            })
        }

        #[test]
        fn test_nonzero_i128_roundtrip(
            x in any::<i128>()
                .prop_filter("Values must not be 0", |x| x != &0)
                .prop_map(|x| NonZeroI128::new(x).unwrap())
        ) {
            Python::attach(|py| {
                let x_py = x.into_pyobject(py).unwrap();
                let locals = PyDict::new(py);
                locals.set_item("x_py", &x_py).unwrap();
                py.run(&CString::new(format!("assert x_py == {x}")).unwrap(), None, Some(&locals)).unwrap();
                let roundtripped: NonZeroI128 = x_py.extract().unwrap();
                assert_eq!(x, roundtripped);
            })
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    proptest! {
        #[test]
        fn test_u128_roundtrip(x: u128) {
            Python::attach(|py| {
                let x_py = x.into_pyobject(py).unwrap();
                let locals = PyDict::new(py);
                locals.set_item("x_py", &x_py).unwrap();
                py.run(&CString::new(format!("assert x_py == {x}")).unwrap(), None, Some(&locals)).unwrap();
                let roundtripped: u128 = x_py.extract().unwrap();
                assert_eq!(x, roundtripped);
            })
        }

        #[test]
        fn test_nonzero_u128_roundtrip(
            x in any::<u128>()
                .prop_filter("Values must not be 0", |x| x != &0)
                .prop_map(|x| NonZeroU128::new(x).unwrap())
        ) {
            Python::attach(|py| {
                let x_py = x.into_pyobject(py).unwrap();
                let locals = PyDict::new(py);
                locals.set_item("x_py", &x_py).unwrap();
                py.run(&CString::new(format!("assert x_py == {x}")).unwrap(), None, Some(&locals)).unwrap();
                let roundtripped: NonZeroU128 = x_py.extract().unwrap();
                assert_eq!(x, roundtripped);
            })
        }
    }

    #[test]
    fn test_i128_max() {
        Python::attach(|py| {
            let v = i128::MAX;
            let obj = v.into_pyobject(py).unwrap();
            assert_eq!(v, obj.extract::<i128>().unwrap());
            assert_eq!(v as u128, obj.extract::<u128>().unwrap());
            assert!(obj.extract::<u64>().is_err());
        })
    }

    #[test]
    fn test_i128_min() {
        Python::attach(|py| {
            let v = i128::MIN;
            let obj = v.into_pyobject(py).unwrap();
            assert_eq!(v, obj.extract::<i128>().unwrap());
            assert!(obj.extract::<i64>().is_err());
            assert!(obj.extract::<u128>().is_err());
        })
    }

    #[test]
    fn test_u128_max() {
        Python::attach(|py| {
            let v = u128::MAX;
            let obj = v.into_pyobject(py).unwrap();
            assert_eq!(v, obj.extract::<u128>().unwrap());
            assert!(obj.extract::<i128>().is_err());
        })
    }

    #[test]
    fn test_i128_overflow() {
        Python::attach(|py| {
            let obj = py.eval(c"(1 << 130) * -1", None, None).unwrap();
            let err = obj.extract::<i128>().unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyOverflowError>(py));
        })
    }

    #[test]
    fn test_u128_overflow() {
        Python::attach(|py| {
            let obj = py.eval(c"1 << 130", None, None).unwrap();
            let err = obj.extract::<u128>().unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyOverflowError>(py));
        })
    }

    #[test]
    fn test_nonzero_i128_max() {
        Python::attach(|py| {
            let v = NonZeroI128::new(i128::MAX).unwrap();
            let obj = v.into_pyobject(py).unwrap();
            assert_eq!(v, obj.extract::<NonZeroI128>().unwrap());
            assert_eq!(
                NonZeroU128::new(v.get() as u128).unwrap(),
                obj.extract::<NonZeroU128>().unwrap()
            );
            assert!(obj.extract::<NonZeroU64>().is_err());
        })
    }

    #[test]
    fn test_nonzero_i128_min() {
        Python::attach(|py| {
            let v = NonZeroI128::new(i128::MIN).unwrap();
            let obj = v.into_pyobject(py).unwrap();
            assert_eq!(v, obj.extract::<NonZeroI128>().unwrap());
            assert!(obj.extract::<NonZeroI64>().is_err());
            assert!(obj.extract::<NonZeroU128>().is_err());
        })
    }

    #[test]
    fn test_nonzero_u128_max() {
        Python::attach(|py| {
            let v = NonZeroU128::new(u128::MAX).unwrap();
            let obj = v.into_pyobject(py).unwrap();
            assert_eq!(v, obj.extract::<NonZeroU128>().unwrap());
            assert!(obj.extract::<NonZeroI128>().is_err());
        })
    }

    #[test]
    fn test_nonzero_i128_overflow() {
        Python::attach(|py| {
            let obj = py.eval(c"(1 << 130) * -1", None, None).unwrap();
            let err = obj.extract::<NonZeroI128>().unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyOverflowError>(py));
        })
    }

    #[test]
    fn test_nonzero_u128_overflow() {
        Python::attach(|py| {
            let obj = py.eval(c"1 << 130", None, None).unwrap();
            let err = obj.extract::<NonZeroU128>().unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyOverflowError>(py));
        })
    }

    #[test]
    fn test_nonzero_i128_zero_value() {
        Python::attach(|py| {
            let obj = py.eval(c"0", None, None).unwrap();
            let err = obj.extract::<NonZeroI128>().unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyValueError>(py));
        })
    }

    #[test]
    fn test_nonzero_u128_zero_value() {
        Python::attach(|py| {
            let obj = py.eval(c"0", None, None).unwrap();
            let err = obj.extract::<NonZeroU128>().unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyValueError>(py));
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::types::PyAnyMethods;
    use crate::{IntoPyObject, Python};
    use std::num::*;

    #[test]
    fn test_u32_max() {
        Python::attach(|py| {
            let v = u32::MAX;
            let obj = v.into_pyobject(py).unwrap();
            assert_eq!(v, obj.extract::<u32>().unwrap());
            assert_eq!(u64::from(v), obj.extract::<u64>().unwrap());
            assert!(obj.extract::<i32>().is_err());
        });
    }

    #[test]
    fn test_i64_max() {
        Python::attach(|py| {
            let v = i64::MAX;
            let obj = v.into_pyobject(py).unwrap();
            assert_eq!(v, obj.extract::<i64>().unwrap());
            assert_eq!(v as u64, obj.extract::<u64>().unwrap());
            assert!(obj.extract::<u32>().is_err());
        });
    }

    #[test]
    fn test_i64_min() {
        Python::attach(|py| {
            let v = i64::MIN;
            let obj = v.into_pyobject(py).unwrap();
            assert_eq!(v, obj.extract::<i64>().unwrap());
            assert!(obj.extract::<i32>().is_err());
            assert!(obj.extract::<u64>().is_err());
        });
    }

    #[test]
    fn test_u64_max() {
        Python::attach(|py| {
            let v = u64::MAX;
            let obj = v.into_pyobject(py).unwrap();
            assert_eq!(v, obj.extract::<u64>().unwrap());
            assert!(obj.extract::<i64>().is_err());
        });
    }

    macro_rules! test_common (
        ($test_mod_name:ident, $t:ty) => (
            mod $test_mod_name {
                use crate::exceptions;
                use crate::conversion::IntoPyObject;
                use crate::types::PyAnyMethods;
                use crate::Python;

                #[test]
                fn from_py_string_type_error() {
                    Python::attach(|py| {
                    let obj = ("123").into_pyobject(py).unwrap();
                    let err = obj.extract::<$t>().unwrap_err();
                    assert!(err.is_instance_of::<exceptions::PyTypeError>(py));
                    });
                }

                #[test]
                fn from_py_float_type_error() {
                    Python::attach(|py| {
                    let obj = (12.3f64).into_pyobject(py).unwrap();
                    let err = obj.extract::<$t>().unwrap_err();
                    assert!(err.is_instance_of::<exceptions::PyTypeError>(py));});
                }

                #[test]
                fn to_py_object_and_back() {
                    Python::attach(|py| {
                    let val = 123 as $t;
                    let obj = val.into_pyobject(py).unwrap();
                    assert_eq!(obj.extract::<$t>().unwrap(), val as $t);});
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
    test_common!(i128, i128);
    test_common!(u128, u128);

    #[test]
    fn test_nonzero_u32_max() {
        Python::attach(|py| {
            let v = NonZeroU32::new(u32::MAX).unwrap();
            let obj = v.into_pyobject(py).unwrap();
            assert_eq!(v, obj.extract::<NonZeroU32>().unwrap());
            assert_eq!(NonZeroU64::from(v), obj.extract::<NonZeroU64>().unwrap());
            assert!(obj.extract::<NonZeroI32>().is_err());
        });
    }

    #[test]
    fn test_nonzero_i64_max() {
        Python::attach(|py| {
            let v = NonZeroI64::new(i64::MAX).unwrap();
            let obj = v.into_pyobject(py).unwrap();
            assert_eq!(v, obj.extract::<NonZeroI64>().unwrap());
            assert_eq!(
                NonZeroU64::new(v.get() as u64).unwrap(),
                obj.extract::<NonZeroU64>().unwrap()
            );
            assert!(obj.extract::<NonZeroU32>().is_err());
        });
    }

    #[test]
    fn test_nonzero_i64_min() {
        Python::attach(|py| {
            let v = NonZeroI64::new(i64::MIN).unwrap();
            let obj = v.into_pyobject(py).unwrap();
            assert_eq!(v, obj.extract::<NonZeroI64>().unwrap());
            assert!(obj.extract::<NonZeroI32>().is_err());
            assert!(obj.extract::<NonZeroU64>().is_err());
        });
    }

    #[test]
    fn test_nonzero_u64_max() {
        Python::attach(|py| {
            let v = NonZeroU64::new(u64::MAX).unwrap();
            let obj = v.into_pyobject(py).unwrap();
            assert_eq!(v, obj.extract::<NonZeroU64>().unwrap());
            assert!(obj.extract::<NonZeroI64>().is_err());
        });
    }

    macro_rules! test_nonzero_common (
        ($test_mod_name:ident, $t:ty) => (
            mod $test_mod_name {
                use crate::exceptions;
                use crate::conversion::IntoPyObject;
                use crate::types::PyAnyMethods;
                use crate::Python;
                use std::num::*;

                #[test]
                fn from_py_string_type_error() {
                    Python::attach(|py| {
                    let obj = ("123").into_pyobject(py).unwrap();
                    let err = obj.extract::<$t>().unwrap_err();
                    assert!(err.is_instance_of::<exceptions::PyTypeError>(py));
                    });
                }

                #[test]
                fn from_py_float_type_error() {
                    Python::attach(|py| {
                    let obj = (12.3f64).into_pyobject(py).unwrap();
                    let err = obj.extract::<$t>().unwrap_err();
                    assert!(err.is_instance_of::<exceptions::PyTypeError>(py));});
                }

                #[test]
                fn to_py_object_and_back() {
                    Python::attach(|py| {
                    let val = <$t>::new(123).unwrap();
                    let obj = val.into_pyobject(py).unwrap();
                    assert_eq!(obj.extract::<$t>().unwrap(), val);});
                }
            }
        )
    );

    test_nonzero_common!(nonzero_i8, NonZeroI8);
    test_nonzero_common!(nonzero_u8, NonZeroU8);
    test_nonzero_common!(nonzero_i16, NonZeroI16);
    test_nonzero_common!(nonzero_u16, NonZeroU16);
    test_nonzero_common!(nonzero_i32, NonZeroI32);
    test_nonzero_common!(nonzero_u32, NonZeroU32);
    test_nonzero_common!(nonzero_i64, NonZeroI64);
    test_nonzero_common!(nonzero_u64, NonZeroU64);
    test_nonzero_common!(nonzero_isize, NonZeroIsize);
    test_nonzero_common!(nonzero_usize, NonZeroUsize);
    test_nonzero_common!(nonzero_i128, NonZeroI128);
    test_nonzero_common!(nonzero_u128, NonZeroU128);

    #[test]
    fn test_i64_bool() {
        Python::attach(|py| {
            let obj = true.into_pyobject(py).unwrap();
            assert_eq!(1, obj.extract::<i64>().unwrap());
            let obj = false.into_pyobject(py).unwrap();
            assert_eq!(0, obj.extract::<i64>().unwrap());
        })
    }

    #[test]
    fn test_i64_f64() {
        Python::attach(|py| {
            let obj = 12.34f64.into_pyobject(py).unwrap();
            let err = obj.extract::<i64>().unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyTypeError>(py));
            // with no remainder
            let obj = 12f64.into_pyobject(py).unwrap();
            let err = obj.extract::<i64>().unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyTypeError>(py));
        })
    }
}
