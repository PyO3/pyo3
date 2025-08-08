/// `concat!` but working with constants
#[macro_export]
#[doc(hidden)]
macro_rules! const_concat {
    ($e:expr) => {{
        $e
    }};
    ($l:expr, $($r:expr),+ $(,)?) => {{
        const PIECES: &[&str] = &[$l, $($r),+];
        const RAW_BYTES: &[u8] = &$crate::impl_::concat::combine_to_array::<{
            $crate::impl_::concat::combined_len(PIECES)
        }>(PIECES);
        // Safety: `RAW_BYTES` is combined from valid &str pieces
        unsafe { ::std::str::from_utf8_unchecked(RAW_BYTES) }
    }}
}

pub use const_concat;

/// Calculates the total byte length of all string pieces in the array.
///
/// This is a useful utility in order to determine the size needed for the constant
/// `combine` function.
pub const fn combined_len(pieces: &[&str]) -> usize {
    let mut len = 0;
    let mut pieces_idx = 0;
    while pieces_idx < pieces.len() {
        len += pieces[pieces_idx].len();
        pieces_idx += 1;
    }
    len
}

/// Combines all string pieces into a single byte array.
///
/// `out` should be a buffer at the correct size of `combined_len(pieces)`, else this will panic.
#[cfg(mut_ref_in_const_fn)] // requires MSRV 1.83
pub const fn combine(pieces: &[&str], out: &mut [u8]) {
    let mut out_idx = 0;
    let mut pieces_idx = 0;
    while pieces_idx < pieces.len() {
        let piece = pieces[pieces_idx].as_bytes();
        let mut piece_idx = 0;
        while piece_idx < piece.len() {
            out[out_idx] = piece[piece_idx];
            out_idx += 1;
            piece_idx += 1;
        }
        pieces_idx += 1;
    }
    assert!(out_idx == out.len(), "output buffer too large");
}

/// Wrapper around combine which has a const generic parameter, this is going to be more codegen
/// at compile time (?)
///
/// Unfortunately the `&mut [u8]` buffer needs MSRV 1.83
pub const fn combine_to_array<const LEN: usize>(pieces: &[&str]) -> [u8; LEN] {
    let mut out: [u8; LEN] = [0u8; LEN];
    #[cfg(mut_ref_in_const_fn)]
    combine(pieces, &mut out);
    #[cfg(not(mut_ref_in_const_fn))] // inlined here for higher code
    {
        let mut out_idx = 0;
        let mut pieces_idx = 0;
        while pieces_idx < pieces.len() {
            let piece = pieces[pieces_idx].as_bytes();
            let mut piece_idx = 0;
            while piece_idx < piece.len() {
                out[out_idx] = piece[piece_idx];
                out_idx += 1;
                piece_idx += 1;
            }
            pieces_idx += 1;
        }
        assert!(out_idx == out.len(), "output buffer too large");
    }
    out
}

/// Calculates the total byte length of all byte pieces in the array.
///
/// This is a useful utility in order to determine the size needed for the constant
/// `combine_bytes` function.
pub const fn combined_len_bytes(pieces: &[&[u8]]) -> usize {
    let mut len = 0;
    let mut pieces_idx = 0;
    while pieces_idx < pieces.len() {
        len += pieces[pieces_idx].len();
        pieces_idx += 1;
    }
    len
}

/// Combines all bytes pieces into a single byte array.
///
/// `out` should be a buffer at the correct size of `combined_len(pieces)`, else this will panic.
#[cfg(mut_ref_in_const_fn)] // requires MSRV 1.83
pub const fn combine_bytes(pieces: &[&[u8]], out: &mut [u8]) {
    let mut out_idx = 0;
    let mut pieces_idx = 0;
    while pieces_idx < pieces.len() {
        let piece = pieces[pieces_idx];
        let mut piece_idx = 0;
        while piece_idx < piece.len() {
            out[out_idx] = piece[piece_idx];
            out_idx += 1;
            piece_idx += 1;
        }
        pieces_idx += 1;
    }
    assert!(out_idx == out.len(), "output buffer too large");
}

/// Wrapper around `combine_bytes` which has a const generic parameter, this is going to be more codegen
/// at compile time (?)
///
/// Unfortunately the `&mut [u8]` buffer needs MSRV 1.83
pub const fn combine_bytes_to_array<const LEN: usize>(pieces: &[&[u8]]) -> [u8; LEN] {
    let mut out: [u8; LEN] = [0u8; LEN];
    #[cfg(mut_ref_in_const_fn)]
    combine_bytes(pieces, &mut out);
    #[cfg(not(mut_ref_in_const_fn))] // inlined here for higher code
    {
        let mut out_idx = 0;
        let mut pieces_idx = 0;
        while pieces_idx < pieces.len() {
            let piece = pieces[pieces_idx];
            let mut piece_idx = 0;
            while piece_idx < piece.len() {
                out[out_idx] = piece[piece_idx];
                out_idx += 1;
                piece_idx += 1;
            }
            pieces_idx += 1;
        }
        assert!(out_idx == out.len(), "output buffer too large");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combined_len() {
        let pieces = ["foo", "bar", "baz"];
        assert_eq!(combined_len(&pieces), 9);
        let empty: [&str; 0] = [];
        assert_eq!(combined_len(&empty), 0);
    }

    #[test]
    fn test_combine_to_array() {
        let pieces = ["foo", "bar"];
        let combined = combine_to_array::<6>(&pieces);
        assert_eq!(&combined, b"foobar");
    }

    #[test]
    fn test_const_concat_macro() {
        const RESULT: &str = const_concat!("foo", "bar", "baz");
        assert_eq!(RESULT, "foobarbaz");
        const SINGLE: &str = const_concat!("abc");
        assert_eq!(SINGLE, "abc");
    }

    #[test]
    fn test_combined_len_bytes() {
        let pieces: [&[u8]; 3] = [b"foo", b"bar", b"baz"];
        assert_eq!(combined_len_bytes(&pieces), 9);
        let empty: [&[u8]; 0] = [];
        assert_eq!(combined_len_bytes(&empty), 0);
    }

    #[test]
    fn test_combine_bytes_to_array() {
        let pieces: [&[u8]; 2] = [b"foo", b"bar"];
        let combined = combine_bytes_to_array::<6>(&pieces);
        assert_eq!(&combined, b"foobar");
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_combine_to_array_buffer_too_small() {
        let pieces = ["foo", "bar"];
        // Intentionally wrong length
        let _ = combine_to_array::<5>(&pieces);
    }

    #[test]
    #[should_panic(expected = "output buffer too large")]
    fn test_combine_to_array_buffer_too_big() {
        let pieces = ["foo", "bar"];
        // Intentionally wrong length
        let _ = combine_to_array::<10>(&pieces);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_combine_bytes_to_array_buffer_too_small() {
        let pieces: [&[u8]; 2] = [b"foo", b"bar"];
        // Intentionally wrong length
        let _ = combine_bytes_to_array::<5>(&pieces);
    }

    #[test]
    #[should_panic(expected = "output buffer too large")]
    fn test_combine_bytes_to_array_buffer_too_big() {
        let pieces: [&[u8]; 2] = [b"foo", b"bar"];
        // Intentionally wrong length
        let _ = combine_bytes_to_array::<10>(&pieces);
    }
}
