/// Calculates the total byte length of all byte pieces in the array.
///
/// This is a useful utility in order to determine the size needed for the constant
/// `combine` function.
pub const fn combined_len(pieces: &[&[u8]]) -> usize {
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
#[allow(clippy::incompatible_msrv)] // `split_at_mut` also requires MSRV 1.83
const fn combine(pieces: &[&[u8]], mut out: &mut [u8]) {
    let mut pieces_idx = 0;
    while pieces_idx < pieces.len() {
        let piece = pieces[pieces_idx];
        slice_copy_from_slice(out, piece);
        // using split_at_mut because range indexing not yet supported in const fn
        out = out.split_at_mut(piece.len()).1;
        pieces_idx += 1;
    }
    // should be no trailing buffer
    assert!(out.is_empty(), "output buffer too large");
}

/// Wrapper around `combine` which has a const generic parameter, this is going to be more codegen
/// at compile time (?)
///
/// Unfortunately the `&mut [u8]` buffer needs MSRV 1.83
pub const fn combine_to_array<const LEN: usize>(pieces: &[&[u8]]) -> [u8; LEN] {
    let mut out: [u8; LEN] = [0u8; LEN];
    #[cfg(mut_ref_in_const_fn)]
    combine(pieces, &mut out);
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

/// Replacement for `slice::copy_from_slice`, which is const from 1.87
#[cfg(mut_ref_in_const_fn)] // requires MSRV 1.83
const fn slice_copy_from_slice(out: &mut [u8], src: &[u8]) {
    let mut i = 0;
    while i < src.len() {
        out[i] = src[i];
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combined_len() {
        let pieces: [&[u8]; 3] = [b"foo", b"bar", b"baz"];
        assert_eq!(combined_len(&pieces), 9);
        let empty: [&[u8]; 0] = [];
        assert_eq!(combined_len(&empty), 0);
    }

    #[test]
    fn test_combine_to_array() {
        let pieces: [&[u8]; 2] = [b"foo", b"bar"];
        let combined = combine_to_array::<6>(&pieces);
        assert_eq!(&combined, b"foobar");
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_combine_to_array_buffer_too_small() {
        let pieces: [&[u8]; 2] = [b"foo", b"bar"];
        // Intentionally wrong length
        let _ = combine_to_array::<5>(&pieces);
    }

    #[test]
    #[should_panic(expected = "output buffer too large")]
    fn test_combine_to_array_buffer_too_big() {
        let pieces: [&[u8]; 2] = [b"foo", b"bar"];
        // Intentionally wrong length
        let _ = combine_to_array::<10>(&pieces);
    }
}
