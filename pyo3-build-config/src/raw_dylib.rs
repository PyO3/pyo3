//! Generation of the `pyo3_libpython_link!` macro used for raw-dylib linking
//! against libpython on Windows.
//!
//! The macro definition is generated at build time (by `pyo3-ffi`'s build
//! script, or by `pyo3-ffi/meson.build`) with the detected Python shared
//! library name baked into a `#[link(name = "...", kind = "raw-dylib")]`
//! attribute, so that `extern_libpython!` in `pyo3-ffi` can link to libpython
//! without the need for import libraries (`.lib` files).

/// Returns the `import_name_type` attribute value for the given target
/// architecture.
///
/// On x86, Python DLLs export undecorated symbol names (no leading
/// underscore), but the default for raw-dylib on x86 is fully-decorated
/// (cdecl adds a `_` prefix). `import_name_type = "undecorated"` is required
/// to match, and it is only valid on x86.
pub fn libpython_import_name_type(target_arch: &str) -> &'static str {
    if target_arch == "x86" {
        ", import_name_type = \"undecorated\""
    } else {
        ""
    }
}

/// Generates the content of the `pyo3_libpython_link!` macro definition file.
///
/// The macro wraps its body in an `unsafe extern $abi` block with a
/// `#[link(name = "{lib_name}", kind = "raw-dylib"{import_name_type})]`
/// attribute applied.
pub fn generate_libpython_link_macro_content(lib_name: &str, import_name_type: &str) -> String {
    let link_line =
        format!("        #[link(name = \"{lib_name}\", kind = \"raw-dylib\"{import_name_type})]");
    [
        "// @generated - do not edit.",
        "// Defines the `#[link(..., kind = \"raw-dylib\")]` attribute for the",
        "// auto-detected Python shared library, so that `extern_libpython!` can",
        "// link to it without the need for import libraries (.lib files).",
        "macro_rules! pyo3_libpython_link {",
        "    ($abi:literal { $($body:tt)* }) => {",
        &link_line,
        "        unsafe extern $abi { $($body)* }",
        "    };",
        "}",
        "",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::libpython_import_name_type;

    #[test]
    fn import_name_type_only_on_x86() {
        assert_eq!(
            libpython_import_name_type("x86"),
            ", import_name_type = \"undecorated\""
        );
        assert_eq!(libpython_import_name_type("x86_64"), "");
        assert_eq!(libpython_import_name_type("aarch64"), "");
        assert_eq!(libpython_import_name_type("arm64"), "");
    }
}
