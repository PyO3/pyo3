/// `concat!` but working with constants
#[macro_export]
#[doc(hidden)]
macro_rules! const_concat {
    ($e:expr) => {{
        $e
    }};
    ($l:expr, $($r:expr),+ $(,)?) => {{
        const L: &'static str = $l;
        const R: &'static str = $crate::impl_::concat::const_concat!($($r),*);
        const LEN: usize = L.len() + R.len();
        const fn combine(l: &'static [u8], r: &'static [u8]) -> [u8; LEN] {
            let mut out = [0u8; LEN];
            let mut i = 0;
            while i < l.len() {
                out[i] = l[i];
                i += 1;
            }
            while i < LEN {
                out[i] = r[i - l.len()];
                i += 1;
            }
            out
        }
        #[allow(unsafe_code)]
        unsafe { ::std::str::from_utf8_unchecked(&combine(L.as_bytes(), R.as_bytes())) }
    }}
}

pub use const_concat;
