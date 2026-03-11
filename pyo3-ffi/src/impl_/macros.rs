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
            all(windows, target_arch = "x86", not(any(PyPy, GraalPy))),
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
        [_PyObject_CallMethod_SizeT]
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
        [_PySequence_IterSearch]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyStack_AsDict]
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
        [_PyEval_EvalFrameDefault]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyEval_RequestCodeExtraIndex]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyCode_GetExtra]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyCode_SetExtra]
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
        [_PyObject_GC_Calloc]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyObject_GC_Malloc]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_Py_GetAllocatedBlocks]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyRun_AnyFileObject]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyRun_InteractiveLoopObject]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyRun_SimpleFileObject]
        $(#[$attrs:meta])* $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?
    ) => {
        extern_libpython_cpython_private_fn! { $(#[$attrs])* $vis $name($($args)*) $(-> $ret)? }
    };
    (
        [_PyUnicode_CheckConsistency]
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
        [_PyLong_NumBits]
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
        [_PyErr_BadInternalCall]
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
        [_PySet_NextEntry]
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
/// The build script sets a `pyo3_dll` cfg value to the target DLL name (e.g. `python312`),
/// and this macro expands to the appropriate `#[link(name = "...", kind = "raw-dylib")]`
/// attribute for that DLL.
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
        extern_libpython!(@impl $abi { $($body)* }
            // abi3
            "python3", "python3_d",
            // Python 3.7 - 3.15
            "python37", "python37_d",
            "python38", "python38_d",
            "python39", "python39_d",
            "python310", "python310_d",
            "python311", "python311_d",
            "python312", "python312_d",
            "python313", "python313_d",
            "python314", "python314_d",
            "python315", "python315_d",
            // free-threaded builds (3.13+)
            "python313t", "python313t_d",
            "python314t", "python314t_d",
            "python315t", "python315t_d",
            // PyPy (DLL is libpypy3.X-c.dll, not pythonXY.dll)
            "libpypy3.11-c",
        );
    };
    // Internal: generate cfg_attr for each DLL name. One of these will be selected
    // by `pyo3-ffi`'s `build.rs`.
    //
    // On x86 Windows, Python DLLs export undecorated symbol names (no leading
    // underscore), but the default for raw-dylib on x86 is fully-decorated
    // (cdecl adds a `_` prefix). We use `import_name_type = "undecorated"` to
    // match. The `import_name_type` key is only valid on x86, so we need
    // separate cfg_attr arms per architecture.
    (@impl $abi:literal { $($body:tt)* } $($dll:literal),* $(,)?) => {
        $(
            #[cfg_attr(all(windows, target_arch = "x86", pyo3_dll = $dll),
                link(name = $dll, kind = "raw-dylib", import_name_type = "undecorated"))]
            #[cfg_attr(all(windows, not(target_arch = "x86"), pyo3_dll = $dll),
                link(name = $dll, kind = "raw-dylib"))]
        )*
        extern $abi {
            extern_libpython_items! { $($body)* }
        }
    };
    // Default ABI: "C"
    ($($body:tt)*) => {
        extern_libpython!("C" { $($body)* });
    };
}
