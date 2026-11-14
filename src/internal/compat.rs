//! Compatibility shims for functions added in Rust versions above MSRV.

/// Replacement for `usize::cast_signed` which was added in Rust 1.87.0.
pub(crate) const fn cast_signed(value: usize) -> isize {
    #[allow(clippy::cast_possible_wrap)]
    {
        value as isize
    }
}

/// Replacement for `isize::cast_unsigned` which was added in Rust 1.87.0.
pub(crate) const fn cast_unsigned(value: isize) -> usize {
    value as usize
}

/// Similar to [`cast_signed`] above, for `u32::cast_signed`.
pub(crate) const fn u32_cast_signed(value: u32) -> i32 {
    #[allow(clippy::cast_possible_wrap)]
    {
        value as i32
    }
}

/// Similar to [`cast_signed`] above, for `u128::cast_signed`.
pub(crate) const fn u128_cast_signed(value: u128) -> i128 {
    #[allow(clippy::cast_possible_wrap)]
    {
        value as i128
    }
}

/// Replacement for `i32::unchecked_neg` which was added in Rust 1.93.0.
#[cfg(any(all(Py_3_14, not(Py_LIMITED_API)), Py_3_15))]
pub(crate) const fn i32_unchecked_neg(value: i32) -> i32 {
    #[allow(clippy::cast_possible_wrap)]
    {
        -value
    }
}

/// Replacement for currently unstable `usize::checked_cast_signed`.
pub(crate) const fn checked_cast_signed(index: usize) -> Option<isize> {
    if index > isize::MAX as usize {
        None
    } else {
        Some(cast_signed(index))
    }
}

/// Replacement for currently unstable `usize::saturating_cast_signed`.
pub(crate) const fn saturating_cast_signed(index: usize) -> isize {
    if index > isize::MAX as usize {
        isize::MAX
    } else {
        cast_signed(index)
    }
}
