use crate::conversion::IntoPyObject;
use crate::inspect::TypeHint;

/// Trait to guess a function Python return type
///
/// It is useful to properly get the return type `T` when the Rust implementation returns e.g. `PyResult<T>`
pub trait PyReturnType {
    /// The function return type
    const OUTPUT_TYPE: TypeHint;
}

impl<'a, T: IntoPyObject<'a>> PyReturnType for T {
    const OUTPUT_TYPE: TypeHint = T::OUTPUT_TYPE;
}

impl<T: PyReturnType, E> PyReturnType for Result<T, E> {
    const OUTPUT_TYPE: TypeHint = T::OUTPUT_TYPE;
}

// TODO: convert it in a macro to build the full JSON for the type hint
#[doc(hidden)]
#[macro_export]
macro_rules! type_hint_json {
    ($hint:expr) => {{
        const HINT: $crate::inspect::TypeHint = $hint;
        const PARTS_LEN: usize = 3 + 4 * HINT.imports.len();
        const PARTS: [&[u8]; PARTS_LEN] = {
            let mut args: [&[u8]; PARTS_LEN] = [b""; PARTS_LEN];
            args[0] = b"{\"annotation\":\"";
            args[1] = HINT.annotation.as_bytes();
            if HINT.imports.is_empty() {
                args[2] = b"\",\"imports\":[]}"
            } else {
                args[2] = b"\",\"imports\":[{\"module\":\"";
                let mut i = 0;
                while i < HINT.imports.len() {
                    if i > 0 {
                        args[4 * i + 2] = b"\"},{\"module\":\"";
                    }
                    args[4 * i + 3] = HINT.imports[i].module.as_bytes();
                    args[4 * i + 4] = b"\",\"name\":\"";
                    args[4 * i + 5] = HINT.imports[i].name.as_bytes();
                    i += 1;
                }
                args[4 * i + 2] = b"\"}]}";
            }
            args
        };
        &$crate::impl_::concat::combine_to_array::<{ $crate::impl_::concat::combined_len(&PARTS) }>(
            &PARTS,
        )
    }};
}

#[repr(C)]
pub struct SerializedIntrospectionFragment<const LEN: usize> {
    pub length: u32,
    pub fragment: [u8; LEN],
}
