use crate::conversion::IntoPyObject;
use crate::impl_::pyclass::PyClassImpl;
use crate::inspect::PyStaticExpr;
use crate::pycell::impl_::PyClassObjectLayout;

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

#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be converted to a Python object",
    label = "required by `#[pyo3(get)]` to create a readable property from a field of type `{Self}`",
    note = "implement `IntoPyObject` for `&{Self}` or `IntoPyObject + Clone` for `{Self}` to define the conversion"
)]
pub trait PyIntoPyObjectMaybeRefType<const REF_IMPL_EXISTS: bool> {
    const OUTPUT_TYPE: PyStaticExpr;
}

impl<'a, 'py, T: 'a> PyIntoPyObjectMaybeRefType<true> for T
where
    &'a T: IntoPyObject<'py>,
{
    const OUTPUT_TYPE: PyStaticExpr = <&T as IntoPyObject<'_>>::OUTPUT_TYPE;
}

impl<'py, T: IntoPyObject<'py>> PyIntoPyObjectMaybeRefType<false> for T {
    const OUTPUT_TYPE: PyStaticExpr = <T as IntoPyObject<'_>>::OUTPUT_TYPE;
}

/// Whether `T` is laid out differently from its base class, making it a
/// [disjoint base](https://peps.python.org/pep-0800/) at runtime.
pub const fn is_disjoint_base<T: PyClassImpl>() -> bool {
    <T::Layout as PyClassObjectLayout<T>>::IS_DISJOINT_BASE
}

#[repr(C)]
pub struct SerializedIntrospectionFragment<const LEN: usize> {
    pub length: u32,
    pub fragment: [u8; LEN],
}

/// Escapes a string to be valid JSON. Does not add quotes around it
///
/// Returns the number of written bytes
pub const fn escape_json_string(input: &str, output: &mut [u8]) -> usize {
    let input = input.as_bytes();
    let mut input_i = 0;
    let mut output_i = 0;
    while input_i < input.len() {
        match input[input_i] {
            b'\\' => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b'\\';
                output_i += 1;
            }
            b'"' => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b'"';
                output_i += 1;
            }
            0x08 => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b'b';
                output_i += 1;
            }
            0x0C => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b'f';
                output_i += 1;
            }
            b'\n' => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b'n';
                output_i += 1;
            }
            b'\r' => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b'r';
                output_i += 1;
            }
            b'\t' => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b't';
                output_i += 1;
            }
            c @ 0..32 => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b'u';
                output_i += 1;
                output[output_i] = b'0';
                output_i += 1;
                output[output_i] = b'0';
                output_i += 1;
                output[output_i] = b'0' + (c / 16);
                output_i += 1;
                let remainder = c % 16;
                output[output_i] = if remainder >= 10 {
                    b'a' + remainder - 10
                } else {
                    b'0' + remainder
                };
                output_i += 1;
            }
            c => {
                output[output_i] = c;
                output_i += 1;
            }
        }
        input_i += 1;
    }
    output_i
}

/// Number of bytes written by [`escape_json_string`]
pub const fn escaped_json_string_len(input: &str) -> usize {
    let input = input.as_bytes();
    let mut len = 0;
    let mut i = 0;
    while i < input.len() {
        len += match input[i] {
            b'\\' | b'"' | 0x08 | 0x0C | b'\n' | b'\r' | b'\t' => 2,
            0..32 => 6,
            _ => 1,
        };
        i += 1;
    }
    len
}

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {
    use super::*;
    use crate::prelude::*;
    use crate::PyTypeInfo;

    #[pyclass(crate = "crate", frozen, subclass)]
    struct Empty;

    #[pyclass(crate = "crate", frozen, subclass)]
    struct WithField(#[allow(dead_code)] u64);

    #[pyclass(crate = "crate", subclass)]
    struct Mutable;

    #[pyclass(crate = "crate", extends = Empty, frozen)]
    struct EmptyChild;

    #[pyclass(crate = "crate", subclass, frozen)]
    struct SizedBase(#[allow(dead_code)] u64);

    /// Adds no data, but is aligned more strictly than its base, so instances still grow.
    #[pyclass(crate = "crate", extends = SizedBase, frozen)]
    #[repr(align(32))]
    struct OverAligned;

    /// The condition `mypy.stubtest` checks, from the runtime type objects.
    fn runtime_is_disjoint_base<T: PyTypeInfo>(py: Python<'_>) -> bool {
        let attr =
            |obj: &Bound<'_, PyAny>, name| obj.getattr(name).unwrap().extract::<isize>().unwrap();
        let ty = T::type_object(py).into_any();
        let base = ty.getattr("__base__").unwrap();
        attr(&ty, "__basicsize__") != attr(&base, "__basicsize__")
            || attr(&ty, "__itemsize__") != attr(&base, "__itemsize__")
    }

    #[test]
    fn is_disjoint_base_matches_runtime_layout() {
        Python::attach(|py| {
            assert_eq!(
                is_disjoint_base::<Empty>(),
                runtime_is_disjoint_base::<Empty>(py)
            );
            assert_eq!(
                is_disjoint_base::<WithField>(),
                runtime_is_disjoint_base::<WithField>(py)
            );
            assert_eq!(
                is_disjoint_base::<Mutable>(),
                runtime_is_disjoint_base::<Mutable>(py)
            );
            assert_eq!(
                is_disjoint_base::<EmptyChild>(),
                runtime_is_disjoint_base::<EmptyChild>(py)
            );
            assert_eq!(
                is_disjoint_base::<SizedBase>(),
                runtime_is_disjoint_base::<SizedBase>(py)
            );
            // Empty contents, so only the padding makes this disjoint.
            assert_eq!(
                core::mem::size_of::<crate::pycell::impl_::PyClassObjectContents<OverAligned>>(),
                0
            );
            assert!(runtime_is_disjoint_base::<OverAligned>(py));
            assert_eq!(
                is_disjoint_base::<OverAligned>(),
                runtime_is_disjoint_base::<OverAligned>(py)
            );
            // The cases must not all agree by accident.
            assert!(is_disjoint_base::<WithField>());
            assert!(!is_disjoint_base::<EmptyChild>());
        });
    }
}
