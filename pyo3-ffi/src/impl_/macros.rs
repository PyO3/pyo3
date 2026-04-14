macro_rules! extern_libpython_items {
    () => {};
    (
        $(#[$attrs:meta])*
        $vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?;
        $($rest:tt)*
    ) => {
        #[cfg_attr(
            all(windows, target_arch = "x86", not(any(PyPy, GraalPy))),
            link_name = stringify!($name)
        )]
        $(#[$attrs])*
        $vis fn $name($($args)*) $(-> $ret)?;

        extern_libpython_items! { $($rest)* }
    };
    (
        $(#[$attrs:meta])*
        $vis:vis static mut $name:ident: $ty:ty;
        $($rest:tt)*
    ) => {
        #[cfg_attr(
            all(windows, target_arch = "x86", not(any(PyPy, GraalPy))),
            link_name = stringify!($name)
        )]
        $(#[$attrs])*
        $vis static mut $name: $ty;

        extern_libpython_items! { $($rest)* }
    };
    (
        $(#[$attrs:meta])*
        $vis:vis static $name:ident: $ty:ty;
        $($rest:tt)*
    ) => {
        #[cfg_attr(
            all(windows, target_arch = "x86", not(any(PyPy, GraalPy))),
            link_name = stringify!($name)
        )]
        $(#[$attrs])*
        $vis static $name: $ty;

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
            // Python 3.8 - 3.15
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
    (@impl $abi:literal { $($body:tt)* } $($dll:literal),* $(,)?) => {
        $(
            #[cfg_attr(all(windows, pyo3_dll = $dll), link(name = $dll, kind = "raw-dylib"))]
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
