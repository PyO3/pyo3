#![cfg(feature = "macros")]

//! Functionality which is not only not supported on MSRV,
//! but can't even be cfg-ed out on MSRV because the compiler doesn't support
//! the syntax.

#[rustversion::since(1.54)]
mod requires_1_54 {

    include!("not_msrv/requires_1_54.rs");
}
