use std::{ffi::CStr, process::exit};

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
            let pyo3_ffi_offset = memoffset::offset_of!(pyo3_ffi::$struct_name, $field);
            #[allow(clippy::used_underscore_binding)]
            let bindgen_offset = memoffset::offset_of!(bindings::$struct_name, $bindgen_field);

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

    if failed {
        exit(1);
    } else {
        exit(0);
    }
}

#[allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    improper_ctypes,
    clippy::all,
    // clippy fails with lots of errors if this is not set specifically
    clippy::used_underscore_binding
)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
