use std::{
    ffi::{CStr, FromBytesWithNulError},
    marker::PhantomData,
};

use crate::{
    impl_::{concat::combine_bytes, pyclass::PyClassImpl},
    PyClass, PyTypeInfo,
};

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

impl<ClassT: PyClass, const HAS_NEW_TEXT_SIGNATURE: bool>
    PyClassDocGenerator<ClassT, HAS_NEW_TEXT_SIGNATURE>
{
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<ClassT: PyClass + PyClassNewTextSignature> PyClassDocGenerator<ClassT, true> {
    pub const DOC_PIECES: &[&[u8]] = &[
        <ClassT as PyTypeInfo>::NAME.as_bytes(),
        ClassT::TEXT_SIGNATURE.as_bytes(),
        b"\n--\n\n",
        <ClassT as PyClassImpl>::RAW_DOC.to_bytes_with_nul(),
    ];
}

impl<ClassT: PyClass> PyClassDocGenerator<ClassT, false> {
    pub const DOC_PIECES: &[&[u8]] = &[<ClassT as PyClassImpl>::RAW_DOC.to_bytes_with_nul()];
}

/// Combines the pieces of documentation as collected by the `PyClassDocGenerator`.
pub const fn create_doc<const LEN: usize>(pieces: &[&[u8]]) -> [u8; LEN] {
    let mut buf = [0u8; LEN];
    combine_bytes(pieces, &mut buf);
    buf
}

/// Casts bytes to a CStr, ensuring they are valid.
pub const fn doc_bytes_as_cstr(bytes: &'static [u8]) -> &'static ::std::ffi::CStr {
    match CStr::from_bytes_with_nul(bytes) {
        Ok(cstr) => cstr,
        // This case may happen if the user provides an invalid docstring
        Err(FromBytesWithNulError::InteriorNul { .. }) => panic!("pyclass doc contains nul bytes"),
        // This case shouldn't happen using the macro machinery as long as `PyClassDocGenerator`
        // uses the RAW_DOC as the final piece, which is nul terminated.
        Err(FromBytesWithNulError::NotNulTerminated) => {
            panic!("pyclass doc expected to be nul terminated")
        }
    }
}
