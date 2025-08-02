/// `concat!` but working with constants
#[macro_export]
#[doc(hidden)]
macro_rules! const_concat {
    ($e:expr) => {{
        $e
    }};
    ($l:expr, $($r:expr),+ $(,)?) => {{
        const PIECES: &[&str] = &[$l, $($r),+];
        const LEN: usize = $crate::impl_::concat::combined_len(PIECES);
        const RAW_BYTES: [u8; LEN] = $crate::impl_::concat::combine::<LEN>(PIECES);
        // Safety: `RAW_BYTES` is combined from valid &str pieces
        unsafe { ::std::str::from_utf8_unchecked(&RAW_BYTES) }
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
/// `LEN` should be the result of `combined_len(pieces)`, else this will panic.
pub const fn combine<const LEN: usize>(pieces: &[&str]) -> [u8; LEN] {
    let mut out = [0u8; LEN];
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
    out
}
