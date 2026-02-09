use crate::conversion::IntoPyObject;
use crate::inspect::PyStaticExpr;

/// Seals `PyReturnType` so that types outside PyO3 cannot implement it.
mod return_type {
    use crate::{impl_::introspection::PyReturnType, IntoPyObject};

    pub trait Sealed {}

    impl<'a, T: IntoPyObject<'a>> Sealed for T {}
    impl<T: PyReturnType, E> Sealed for Result<T, E> {}
}

/// Trait to guess a function Python return type
///
/// It is useful to properly get the return type `T` when the Rust implementation returns e.g. `PyResult<T>`
pub trait PyReturnType: return_type::Sealed {
    /// The function return type
    const OUTPUT_TYPE: PyStaticExpr;
}

impl<'a, T: IntoPyObject<'a>> PyReturnType for T {
    const OUTPUT_TYPE: PyStaticExpr = T::OUTPUT_TYPE;
}

impl<T: PyReturnType, E> PyReturnType for Result<T, E> {
    const OUTPUT_TYPE: PyStaticExpr = T::OUTPUT_TYPE;
}

#[repr(C)]
pub struct SerializedIntrospectionFragment<const LEN: usize> {
    pub length: u32,
    pub fragment: [u8; LEN],
}
