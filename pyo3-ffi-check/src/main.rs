use std::{ffi::CStr, process::exit};

use pyo3_ffi_check_definitions::{bindgen as bindings, pyo3_ffi};

fn main() {
    println!(
        "comparing pyo3-ffi against headers generated for {}",
        CStr::from_bytes_with_nul(bindings::PY_VERSION)
            .unwrap()
            .to_string_lossy()
    );

    let mut failed = false;

    macro_rules! check_struct {
        ($name:ident) => {{
            let pyo3_ffi_size = std::mem::size_of::<pyo3_ffi::$name>();
            let bindgen_size = std::mem::size_of::<bindings::$name>();

            let pyo3_ffi_align = std::mem::align_of::<pyo3_ffi::$name>();
            let bindgen_align = std::mem::align_of::<bindings::$name>();

            // Check if sizes differ, but ignore zero-sized types (probably "opaque" in pyo3-ffi)
            if pyo3_ffi_size == 0 {
                println!(
                    "warning: ignoring zero-sized pyo3_ffi type {}",
                    stringify!($name),
                );
            } else if pyo3_ffi_size != bindgen_size {
                failed = true;
                println!(
                    "error: size of {} differs between pyo3_ffi ({}) and bindgen ({})",
                    stringify!($name),
                    pyo3_ffi_size,
                    bindgen_size
                );
            } else if pyo3_ffi_align != bindgen_align {
                failed = true;
                println!(
                    "error: alignment of {} differs between pyo3_ffi ({}) and bindgen ({})",
                    stringify!($name),
                    pyo3_ffi_align,
                    bindgen_align
                );
            }

            pyo3_ffi_check_macro::for_all_fields!($name, check_field);
        }};
    }

    macro_rules! check_field {
        ($struct_name:ident, $field:ident, $bindgen_field:ident) => {{
            // some struct fields are deprecated but still present in the ABI
            #[allow(clippy::used_underscore_binding, deprecated)]
            let pyo3_ffi_offset = std::mem::offset_of!(pyo3_ffi::$struct_name, $field);
            #[allow(clippy::used_underscore_binding)]
            let bindgen_offset = std::mem::offset_of!(bindings::$struct_name, $bindgen_field);

            if pyo3_ffi_offset != bindgen_offset {
                failed = true;
                println!(
                    "error: field offset of {}.{} differs between pyo3_ffi ({}) and bindgen ({})",
                    stringify!($struct_name),
                    stringify!($field),
                    pyo3_ffi_offset,
                    bindgen_offset
                );
            }
        }};
    }

    pyo3_ffi_check_macro::for_all_structs!(check_struct);

    struct ReturnTypeSniffer<T> {
        _marker: std::marker::PhantomData<T>,
    }

    impl<T> ReturnTypeSniffer<T> {
        /// Uses type inference of return value from a closure to sniff return type `T` without needing
        /// to ever call the function.
        fn new(_: impl FnOnce() -> T) -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }

        fn check_compatible<U>(
            symbol: &str,
            _pyo3_ffi_type: &ReturnTypeSniffer<T>,
            _bindgen_type: &ReturnTypeSniffer<U>,
        ) -> bool {
            let mut compatible = true;
            if std::mem::size_of::<T>() != std::mem::size_of::<U>() {
                compatible = false;
                println!(
                    "error: {symbol} return type size differs (pyo3-ffi type is {}, bindgen type is {})",
                    std::any::type_name::<T>(),
                    std::any::type_name::<U>(),
                );
            }

            if std::mem::align_of::<T>() != std::mem::align_of::<U>() {
                compatible = false;
                println!(
                    "error: {symbol} return type alignment differs (pyo3-ffi type is {}, bindgen type is {})",
                    std::any::type_name::<T>(),
                    std::any::type_name::<U>(),
                );
            }
            compatible
        }
    }

    /// tt muncher macro replacing arg types with `todo!()` to
    /// allow to sniff return type
    macro_rules! todo_args {
        // entry point
        (($f:expr)($($args:tt)*)) => {
            todo_args!(@[($f)()] $($args)*)
        };
        // terminate when no more args
        (@[($f:expr)($($inferred:tt)*)]) => {
            // allow not expect because args might be empty
            #[allow(unreachable_code, reason = "args are todo!()")]
            ($f)($($inferred)*)
        };
        // consume comma
        (@[($f:expr)($($inferred:tt)*)] , $($args:tt)*) => {
            todo_args!(@[($f)($($inferred)* ,)] $($args)*)
        };
        // consume `_`
        (@[($f:expr)($($inferred:tt)*)] _ $($args:tt)*) => {
            todo_args!(@[($f)($($inferred)* todo!())] $($args)*)
        };
        // consume trailing `...`
        (@[($f:expr)($($inferred:tt)*)] ...) => {
            todo_args!(@[($f)($($inferred)*)])
        };
    }

    macro_rules! check_function {
        ($name:ident, [$($modifiers:tt)*] ($($arg_types:tt)*)) => {{
            // Check functions have the same number of arguments
            #[allow(deprecated)]
            { pyo3_ffi::$name as $($modifiers)* fn($($arg_types)*) -> _ };
            bindings::$name as $($modifiers)* fn($($arg_types)*) -> _;

            // TODO: can probably sniff arg types by binding sniffers for each argument position and then passing
            // those inside `todo_args!` to use type inference for each argument.

            // Check return types are compatible
            #[allow(deprecated)]
            let pyo3_ffi_return_type = ReturnTypeSniffer::new(|| unsafe { todo_args!((pyo3_ffi::$name)($($arg_types)*)) });
            let bindgen_return_type = ReturnTypeSniffer::new(|| unsafe { todo_args!((bindings::$name)($($arg_types)*)) });

            failed |= !ReturnTypeSniffer::check_compatible(stringify!($name), &pyo3_ffi_return_type, &bindgen_return_type);
        }};
        // case when the function is an inline function in the headers, in which case pyo3-ffi will use the
        // Rust abi and the extern symbol uses the C abi
        (@inline $name:ident, ($($arg_types:tt)*)) => {{
            // Check functions have the same number of arguments
            #[allow(deprecated)]
            { pyo3_ffi::$name as unsafe fn($($arg_types)*) -> _ };
            bindings::$name as unsafe extern "C" fn($($arg_types)*) -> _;

            // TODO: can probably sniff arg types by binding sniffers for each argument position and then passing
            // those inside `todo_args!` to use type inference for each argument.

            // Check return types are compatible
            #[allow(deprecated)]
            let pyo3_ffi_return_type = ReturnTypeSniffer::new(|| unsafe { todo_args!((pyo3_ffi::$name)($($arg_types)*)) });
            let bindgen_return_type = ReturnTypeSniffer::new(|| unsafe { todo_args!((bindings::$name)($($arg_types)*)) });

            failed |= !ReturnTypeSniffer::check_compatible(stringify!($name), &pyo3_ffi_return_type, &bindgen_return_type);
        }};
    }

    pyo3_ffi_check_macro::for_all_functions!(check_function);

    if failed {
        exit(1);
    } else {
        exit(0);
    }
}
