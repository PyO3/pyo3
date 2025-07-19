use crate::conversion::IntoPyObject;

/// Trait to guess a function Python return type
///
/// It is useful to properly get the return type `T` when the Rust implementation returns e.g. `PyResult<T>`
pub trait PyReturnType {
    /// The function return type
    const OUTPUT_TYPE: &'static str;
}

impl<'a, T: IntoPyObject<'a>> PyReturnType for T {
    const OUTPUT_TYPE: &'static str = T::OUTPUT_TYPE;
}

impl<T: PyReturnType, E> PyReturnType for Result<T, E> {
    const OUTPUT_TYPE: &'static str = T::OUTPUT_TYPE;
}
