use crate::impl_::concat::slice_copy_from_slice;

/// This is a reimplementation of the `indoc` crate's unindent functionality:
///
/// 1. Count the leading spaces of each line, ignoring the first line and any lines that are empty or contain spaces only.
/// 2. Take the minimum.
/// 3. If the first line is empty i.e. the string begins with a newline, remove the first line.
/// 4. Remove the computed number of spaces from the beginning of each line.
const fn unindent_bytes(bytes: &mut [u8]) -> usize {
    // (1) + (2) - count leading spaces, take the minimum
    let Some(to_unindent) = get_minimum_leading_spaces(bytes) else {
        // all lines were empty, nothing to unindent
        return bytes.len();
    };

    // now copy from the original buffer, bringing values forward as needed
    let mut read_idx = 0;
    let mut write_idx = 0;

    // (3) - remove first line if it is empty
    match consume_eol(bytes, read_idx) {
        // skip empty first line
        Some(eol) => read_idx = eol,
        // copy non-empty first line as-is
        None => {
            (read_idx, write_idx) = copy_forward_until_eol(bytes, read_idx, write_idx);
        }
    };

    // (4) - unindent remaining lines
    while read_idx < bytes.len() {
        let leading_spaces = count_spaces(bytes, read_idx);

        if leading_spaces < to_unindent {
            read_idx += leading_spaces;
            assert!(
                consume_eol(bytes, read_idx).is_some(),
                "removed fewer spaces than expected on non-empty line"
            );
        } else {
            // leading_spaces may be equal to or larger than to_unindent, only need to unindent
            // the required amount, additional indentation is meaningful
            read_idx += to_unindent;
        }

        // copy remainder of line
        (read_idx, write_idx) = copy_forward_until_eol(bytes, read_idx, write_idx);
    }

    write_idx
}

/// Counts the minimum leading spaces of all non-empty lines except the first line.
///
/// Returns `None` if there are no non-empty lines except the first line.
const fn get_minimum_leading_spaces(bytes: &[u8]) -> Option<usize> {
    // scan for leading spaces (ignoring first line and empty lines)
    let mut i = 0;

    // skip first line
    i = advance_to_next_line(bytes, i);

    let mut to_unindent = None;

    // for remaining lines, count leading spaces
    while i < bytes.len() {
        let line_leading_spaces = count_spaces(bytes, i);
        i += line_leading_spaces;

        // line only had spaces, ignore for the count
        if let Some(eol) = consume_eol(bytes, i) {
            i = eol;
            continue;
        }

        // this line has content, consider its leading spaces
        if let Some(current) = to_unindent {
            // .unwrap_or(usize::MAX) not available in const fn
            if line_leading_spaces < current {
                to_unindent = Some(line_leading_spaces);
            }
        } else {
            to_unindent = Some(line_leading_spaces);
        }

        i = advance_to_next_line(bytes, i);
    }

    to_unindent
}

const fn advance_to_next_line(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() {
        if let Some(eol) = consume_eol(bytes, i) {
            return eol;
        }
        i += 1;
    }
    i
}

/// Brings elements in `bytes` forward until `\n` (inclusive) or end of `source`.
///
/// `read_idx` must be greater than or equal to `write_idx`.
const fn copy_forward_until_eol(
    bytes: &mut [u8],
    mut read_idx: usize,
    mut write_idx: usize,
) -> (usize, usize) {
    assert!(read_idx >= write_idx);
    while read_idx < bytes.len() {
        let value = bytes[read_idx];
        bytes[write_idx] = value;
        read_idx += 1;
        write_idx += 1;
        if value == b'\n' {
            break;
        }
    }
    (read_idx, write_idx)
}

const fn count_spaces(bytes: &[u8], mut i: usize) -> usize {
    let mut count = 0;
    while i < bytes.len() && bytes[i] == b' ' {
        count += 1;
        i += 1;
    }
    count
}

const fn consume_eol(bytes: &[u8], i: usize) -> Option<usize> {
    if bytes.len() == i {
        // special case: treat end of buffer as EOL without consuming anything
        Some(i)
    } else if bytes.len() > i && bytes[i] == b'\n' {
        Some(i + 1)
    } else if bytes[i] == b'\r' && bytes.len() > i + 1 && bytes[i + 1] == b'\n' {
        Some(i + 2)
    } else {
        None
    }
}

pub const fn unindent_sized<const N: usize>(src: &[u8]) -> ([u8; N], usize) {
    let mut out: [u8; N] = [0; N];
    slice_copy_from_slice(&mut out, src);
    let new_len = unindent_bytes(&mut out);
    (out, new_len)
}

/// Helper for `py_run!` macro which unindents a string at compile time.
#[macro_export]
#[doc(hidden)]
macro_rules! unindent {
    ($value:expr) => {{
        const RAW: &str = $value;
        const LEN: usize = RAW.len();
        const UNINDENTED: ([u8; LEN], usize) =
            $crate::impl_::unindent::unindent_sized::<LEN>(RAW.as_bytes());
        // SAFETY: this removes only spaces and preserves all other contents
        unsafe { ::core::str::from_utf8_unchecked(UNINDENTED.0.split_at(UNINDENTED.1).0) }
    }};
}

pub use crate::unindent;

/// Equivalent of the `unindent!` macro, but works at runtime.
pub fn unindent(s: &str) -> String {
    let mut bytes = s.as_bytes().to_owned();
    let unindented_size = unindent_bytes(&mut bytes);
    bytes.resize(unindented_size, 0);
    String::from_utf8(bytes).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_1_WITH_FIRST_LINE: &str = "  first line
        line one

          line two
    ";

    const UNINDENTED_1: &str = "  first line\nline one\n\n  line two\n";

    const SAMPLE_2_EMPTY_FIRST_LINE: &str = "
            line one

              line two
        ";
    const UNINDENTED_2: &str = "line one\n\n  line two\n";

    const SAMPLE_3_NO_INDENT: &str = "
no indent
  here";

    const UNINDENTED_3: &str = "no indent\n  here";

    const SAMPLE_4_NOOP: &str = "no indent\nhere\n  but here";

    const SAMPLE_5_EMPTY: &str = "   \n   \n";

    const ALL_CASES: &[(&str, &str)] = &[
        (SAMPLE_1_WITH_FIRST_LINE, UNINDENTED_1),
        (SAMPLE_2_EMPTY_FIRST_LINE, UNINDENTED_2),
        (SAMPLE_3_NO_INDENT, UNINDENTED_3),
        (SAMPLE_4_NOOP, SAMPLE_4_NOOP),
        (SAMPLE_5_EMPTY, SAMPLE_5_EMPTY),
    ];

    // run const tests for each sample to ensure they work at compile time

    #[test]
    fn test_unindent_const() {
        const UNINDENTED: &str = unindent!(SAMPLE_1_WITH_FIRST_LINE);
        assert_eq!(UNINDENTED, UNINDENTED_1);
    }

    #[test]
    fn test_unindent_const_removes_empty_first_line() {
        const UNINDENTED: &str = unindent!(SAMPLE_2_EMPTY_FIRST_LINE);
        assert_eq!(UNINDENTED, UNINDENTED_2);
    }

    #[test]
    fn test_unindent_const_no_indent() {
        const UNINDENTED: &str = unindent!(SAMPLE_3_NO_INDENT);
        assert_eq!(UNINDENTED, UNINDENTED_3);
    }

    #[test]
    fn test_unindent_macro_runtime() {
        // this variation on the test ensures full coverage (const eval not included in coverage)
        const INDENTED: &str = SAMPLE_1_WITH_FIRST_LINE;
        const LEN: usize = INDENTED.len();
        let (unindented, unindented_size) = unindent_sized::<LEN>(INDENTED.as_bytes());
        let unindented = std::str::from_utf8(&unindented[..unindented_size]).unwrap();
        assert_eq!(unindented, UNINDENTED_1);
    }

    #[test]
    fn test_unindent_function() {
        for (indented, expected) in ALL_CASES {
            let unindented = unindent(indented);
            assert_eq!(&unindented, expected);
        }
    }
}
