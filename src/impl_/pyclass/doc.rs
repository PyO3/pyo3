use std::{ffi::CStr, marker::PhantomData};

use crate::{impl_::pyclass::PyClassImpl, PyClass, PyTypeInfo};

/// Trait implemented by classes with a known text signature for instantiation.
///
/// This is implemented by the `#[pymethods]` macro when handling expansion for a
/// `#[new]` method.
pub trait PyClassNewTextSignature {
    const TEXT_SIGNATURE: &'static str;
}

/// Type which uses specialization on impl blocks to facilitate generating the documentation for a
/// `#[pyclass]` type.
///
/// At the moment, this is only used to help lift the `TEXT_SIGNATURE` constant to compile time
/// providing a base case and a specialized implementation when the signature is known at compile time.
///
/// In the future when const eval is more advanced, it will probably be possible to format the whole
/// class docstring at compile time as part of this type instead of in macro expansion.
pub struct PyClassDocGenerator<
    ClassT: PyClass,
    // switch to determine if a signature for class instantiation is known
    const HAS_NEW_TEXT_SIGNATURE: bool,
>(PhantomData<ClassT>);

impl<ClassT: PyClass + PyClassNewTextSignature> PyClassDocGenerator<ClassT, true> {
    pub const DOC_PIECES: &'static [&'static [u8]] = &[
        <ClassT as PyTypeInfo>::NAME.as_bytes(),
        ClassT::TEXT_SIGNATURE.as_bytes(),
        b"\n--\n\n",
        <ClassT as PyClassImpl>::RAW_DOC.to_bytes_with_nul(),
    ];
}

impl<ClassT: PyClass> PyClassDocGenerator<ClassT, false> {
    pub const DOC_PIECES: &'static [&'static [u8]] =
        &[<ClassT as PyClassImpl>::RAW_DOC.to_bytes_with_nul()];
}

/// Casts bytes to a CStr, ensuring they are valid.
pub const fn doc_bytes_as_cstr(bytes: &'static [u8]) -> &'static ::std::ffi::CStr {
    match CStr::from_bytes_with_nul(bytes) {
        Ok(cstr) => cstr,
        #[cfg(not(from_bytes_with_nul_error))] // MSRV 1.86
        Err(_) => panic!("invalid pyclass doc"),
        #[cfg(from_bytes_with_nul_error)]
        // This case may happen if the user provides an invalid docstring
        Err(std::ffi::FromBytesWithNulError::InteriorNul { .. }) => {
            panic!("pyclass doc contains nul bytes")
        }
        // This case shouldn't happen using the macro machinery as long as `PyClassDocGenerator`
        // uses the RAW_DOC as the final piece, which is nul terminated.
        #[cfg(from_bytes_with_nul_error)]
        Err(std::ffi::FromBytesWithNulError::NotNulTerminated) => {
            panic!("pyclass doc expected to be nul terminated")
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::ffi;

    use super::*;

    #[test]
    #[cfg(feature = "macros")]
    fn test_doc_generator() {
        use crate::impl_::concat::{combine_to_array, combined_len};

        /// A dummy class with signature.
        #[crate::pyclass(crate = "crate")]
        struct MyClass;

        #[crate::pymethods(crate = "crate")]
        impl MyClass {
            #[new]
            fn new(x: i32, y: i32) -> Self {
                let _ = (x, y); // suppress unused variable warnings
                MyClass
            }
        }

        // simulate what the macro is doing
        const PIECES: &[&[u8]] = PyClassDocGenerator::<MyClass, true>::DOC_PIECES;
        assert_eq!(
            &combine_to_array::<{ combined_len(PIECES) }>(PIECES),
            b"MyClass(x, y)\n--\n\nA dummy class with signature.\0"
        );

        // simulate if the macro detected no text signature
        const PIECES_WITHOUT_SIGNATURE: &[&[u8]] =
            PyClassDocGenerator::<MyClass, false>::DOC_PIECES;
        assert_eq!(
            &combine_to_array::<{ combined_len(PIECES_WITHOUT_SIGNATURE) }>(
                PIECES_WITHOUT_SIGNATURE
            ),
            b"A dummy class with signature.\0"
        );
    }

    #[test]
    fn test_doc_bytes_as_cstr() {
        let cstr = doc_bytes_as_cstr(b"MyClass\0");
        assert_eq!(cstr, ffi::c_str!("MyClass"));
    }

    #[test]
    #[cfg(from_bytes_with_nul_error)]
    #[should_panic(expected = "pyclass doc contains nul bytes")]
    fn test_doc_bytes_as_cstr_central_nul() {
        doc_bytes_as_cstr(b"MyClass\0Foo");
    }

    #[test]
    #[cfg(from_bytes_with_nul_error)]
    #[should_panic(expected = "pyclass doc expected to be nul terminated")]
    fn test_doc_bytes_as_cstr_not_nul_terminated() {
        doc_bytes_as_cstr(b"MyClass");
    }
}
