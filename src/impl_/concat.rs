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
    assert!(out_idx == out.len(), "Output buffer size mismatch");
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
        assert!(out_idx == out.len(), "Output buffer size mismatch");
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
    assert!(out_idx == out.len(), "Output buffer size mismatch");
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
        assert!(out_idx == out.len(), "Output buffer size mismatch");
    }
    out
}
