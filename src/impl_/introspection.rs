use crate::conversion::IntoPyObject;
use crate::inspect::PyStaticExpr;

/// Trait to guess a function Python return type
///
/// It is useful to properly get the return type `T` when the Rust implementation returns e.g. `PyResult<T>`
pub trait PyReturnType {
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
