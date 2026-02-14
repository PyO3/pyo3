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
                let remainer = c % 16;
                output[output_i] = if remainer >= 10 {
                    b'a' + remainer - 10
                } else {
                    b'0' + remainer
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
