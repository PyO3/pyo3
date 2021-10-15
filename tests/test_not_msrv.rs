//! Functionality which is not only not supported on MSRV,
//! but can't even be cfg-ed out on MSRV because the compiler doesn't support
//! the syntax.

// TODO(#1782) rustversion attribute can't go on modules until Rust 1.42, so this
// funky dance has to happen...
mod requires_1_54 {
    #[rustversion::since(1.54)]
    include!("not_msrv/requires_1_54.rs");
}
