//! This file re-exports various libc functions and types, and adds a custom libc implementation
//! for wasm32-unknown-unknown, since libc does not support wasm32-unknown-unknown.
//!
//! When compiled for wasm32-unknown-unknown, this is expected to be used in an
//! emscripten environment, and the definitions are chosen accordingly.

#![allow(non_camel_case_types)]
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod libc {
    extern "C" {
        pub fn atexit(cb: extern "C" fn()) -> c_int;
    }
    pub type c_char = i8;
    pub type c_int = i32;
    pub type c_ulong = u32;
    pub type c_void = std::ffi::c_void;
    pub type intptr_t = isize;
    pub type size_t = usize;
    pub type ssize_t = isize;
    pub type uintptr_t = usize;
    pub type wchar_t = u32;
    pub enum FILE {}
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub mod libc {
    pub use libc::atexit;
    pub use libc::c_char;
    pub use libc::c_int;
    pub use libc::c_ulong;
    pub use libc::c_void;
    pub use libc::intptr_t;
    pub use libc::size_t;
    pub use libc::ssize_t;
    pub use libc::uintptr_t;
    pub use libc::wchar_t;
    pub use libc::FILE;
}
