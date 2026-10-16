// On x86 Windows, `raw-dylib` with `import_name_type = "undecorated"` removes the
// leading cdecl underscore from function names. This is expected behavior for
// `import_name_type = "undecorated"` (not a rustc bug): it strips the cdecl `_`
// prefix, which collides with symbols whose real names start with `_Py`.
// See https://doc.rust-lang.org/reference/items/external-blocks.html#the-import_name_type-key
//
// That matches ordinary `Py_*` exports, but it breaks CPython's internal `_Py*`
// function exports whose real DLL names already start with an underscore. For
// those functions, ask rustc for one extra underscore so that x86 undecoration
// lands back on CPython's export.
//
// Variables are intentionally excluded here: `import_name_type` does not affect
// variable imports, so `_Py_*` statics continue to work without any rewriting.
#[allow(unused_macros, reason = "used indirectly by extern_libpython_item!")]
macro_rules! extern_libpython_cpython_private_fn {
    ($(#[$attrs:meta])* $vis:vis $name:ident($($args:tt)*) $(-> $ret:ty)?) => {
        #[cfg_attr(
            all(windows, pyo3_use_raw_dylib, target_arch = "x86"),
            link_name = concat!("_", stringify!($name))
        )]
        $(#[$attrs])*
        $vis fn $name($($args)*) $(-> $ret)?;
    };
}

// Keep this list in sync with `_Py*` function imports declared through
// `extern_libpython!`. The x86 workaround only needs to apply to functions:
// statics keep their original names even when `import_name_type` is set. Match
// by name only here so the function signature stays in a single generic arm.
//
// TODO: reduce the number of `_Py*` exports from pyo3-ffi over time — the fewer
// CPython-private functions we expose, the smaller this workaround list becomes.
#[allow(unused_macros, reason = "used indirectly by extern_libpython_item!")]
macro_rules! extern_libpython_maybe_private_fn {
    (
        [_PyObject_CallFunction_SizeT]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyObject_MakeTpCall]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_Py_CheckFunctionResult]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyBytes_Resize]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyLong_AsByteArray]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyLong_FromByteArray]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyUnicode_Ready]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyUnicode_ToDecimalDigit]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyThreadState_UncheckedGet]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyInterpreterState_GetEvalFrameFunc]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyInterpreterState_SetEvalFrameFunc]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyObject_GC_New]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyObject_GC_NewVar]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyObject_GC_Resize]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyObject_New]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyObject_NewVar]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_Py_HashBytes]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_Py_DECREF_DecRefTotal]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_Py_Dealloc]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_Py_DecRef]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_Py_INCREF_IncRefTotal]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_Py_IncRef]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_Py_NegativeRefcount]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyErr_BadInternalCall]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [$name:ident]
        $(#[$attrs:meta])* $vis:vis fn $fn_name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        $(#[$attrs])*
        $vis fn $fn_name($($args)*) $(-> $ret)?;
    };
}

macro_rules! extern_libpython_item {
    ($(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?) => {
        extern_libpython_maybe_private_fn! {
            [$name]
            $(#[$attrs])*
            $vis fn $name($($args)*) $(-> $ret)?
        }
    };
    ($(#[$attrs:meta])* $vis:vis static mut $name:ident: $ty:ty) => {
        $(#[$attrs])*
        $vis static mut $name: $ty;
    };
    ($(#[$attrs:meta])* $vis:vis static $name:ident: $ty:ty) => {
        $(#[$attrs])*
        $vis static $name: $ty;
    };
}

macro_rules! extern_libpython_items {
    () => {};
    (
        $(#[$attrs:meta])*
        $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?;
        $($rest:tt)*
    ) => {
        extern_libpython_item! {
            $(#[$attrs])*
            $vis fn $name($($args)*) $(-> $ret)?
        }
        extern_libpython_items! { $($rest)* }
    };
    (
        $(#[$attrs:meta])*
        $vis:vis static mut $name:ident: $ty:ty;
        $($rest:tt)*
    ) => {
        extern_libpython_item! {
            $(#[$attrs])*
            $vis static mut $name: $ty
        }
        extern_libpython_items! { $($rest)* }
    };
    (
        $(#[$attrs:meta])*
        $vis:vis static $name:ident: $ty:ty;
        $($rest:tt)*
    ) => {
        extern_libpython_item! {
            $(#[$attrs])*
            $vis static $name: $ty
        }
        extern_libpython_items! { $($rest)* }
    };
}

/// Helper macro to declare `extern` blocks that link against libpython on Windows
/// using `raw-dylib`, eliminating the need for import libraries.
///
/// The build system auto-detects the target Python shared library at build time
/// and generates the `pyo3_libpython_link!` macro (included in the crate root),
/// which expands to the appropriate `#[link(name = "...", kind = "raw-dylib")]`
/// attribute for that library. Because the library name is resolved automatically
/// per build, any Python version or implementation (CPython, PyPy, GraalPy, ...)
/// is supported without changes to this crate's source.
///
/// # Usage
///
/// ```rust,ignore
/// // Default ABI "C" (most common):
/// extern_libpython! {
///     pub fn PyObject_Call(
///         callable: *mut PyObject,
///         args: *mut PyObject,
///         kwargs: *mut PyObject,
///     ) -> *mut PyObject;
/// }
///
/// // Explicit ABI:
/// extern_libpython! { "C-unwind" {
///     pub fn PyGILState_Ensure() -> PyGILState_STATE;
/// }}
/// ```
macro_rules! extern_libpython {
    // Explicit ABI
    ($abi:literal { $($body:tt)* }) => {
        extern_libpython!(@impl $abi { $($body)* });
    };
    // Internal: with raw-dylib linking on Windows, the generated
    // `pyo3_libpython_link!` macro supplies the `#[link(name = "...",
    // kind = "raw-dylib")]` attribute for the detected Python DLL.
    //
    // Otherwise (or on other platforms), a plain `unsafe extern` block is used.
    // On Windows the `link(name = "pythonXY")` attribute is an empty placeholder
    // resolved by the build script's `cargo:rustc-link-lib=pythonXY:` alias.
    (@impl $abi:literal { $($body:tt)* }) => {
        #[cfg(all(windows, pyo3_use_raw_dylib))]
        pyo3_libpython_link! {
            $abi { extern_libpython_items! { $($body)* } }
        }
        #[cfg(not(all(windows, pyo3_use_raw_dylib)))]
        #[cfg_attr(windows, link(name = "pythonXY"))]
        unsafe extern $abi {
            extern_libpython_items! { $($body)* }
        }
    };
    // Default ABI: "C"
    ($($body:tt)*) => {
        extern_libpython!("C" { $($body)* });
    };
}
