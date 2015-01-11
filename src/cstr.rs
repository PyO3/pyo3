use libc::c_char;
use std::ffi::CString;
use std::borrow::{BorrowFrom, ToOwned};
use std::{fmt, mem, ops, str};

// Idea:
// CString = Vec<u8> + 0-invariant (0-terminated, but no interior 0)
// this makes it similar to
// String = Vec<u8> + UTF-8-invariant

// Dereferencing String results in
// str = [u8] + UTF-8 invariant

// So why does dereferencing CString result in [u8], dropping the 0-invariant?
// This module implements a type CStr = [u8] + 0-invariant.

// This allows writing safe FFI bindings that accept &CStr and call a C function:
// fn f(s: &CStr) { ffi::f(s.as_ptr()) }

// Without CStr, f() would have to take &CString (forcing the string to be heap-allocated),
// or f() could take &[u8] and verify the 0-invariant itself -- but this verification is redundant
// when the &[u8] was borrowed from a CString.
// (or otherwise known to be valid, e.g. the output of a cstr!("string literal") macro)


// CString changes if this type is adopted in std::ffi:
// * <CString as Deref>::Target should change from &[c_char] to &CStr
// * CString::{as_slice_with_nul, as_bytes, as_bytes_with_nul} can be deleted, as they already available through Deref
// * The free functions c_str_to_bytes and c_str_to_bytes_with_nul can be removed, as the same functionality is
//   available through CStr::from_ptr( ).as_bytes() / CStr::from_ptr( ).as_bytes_with_nul()

// Independently from CStr:
// * CString::from_slice(s.as_bytes()) looks weird, since both 'as_slice' and 'as_bytes'
//    exist on CString with different return types.
//   CString::from_slice should be renamed to from_bytes

// #[derive(PartialEq, PartialOrd, Eq, Ord, Hash) -- ICE #18805
// #[repr(newtype)] or something, for the transmute in from_slice_with_nul_unchecked
pub struct CStr {
    // invariants:
    // - data.len() >= 1
    // - data[0..data.len()-1] does not contain '\0'
    // - data[data.len()-1] == '\0'
    inner: [c_char]
}


impl CStr {
    /// Convert a C string pointer into a &CStr reference.
    ///
    /// Unsafe because:
    /// * The pointer is assumed to point to a valid C string.
    /// * The lifetime provided may not be a suitable lifetime for the returned &CStr.
    pub unsafe fn from_ptr<'a>(raw: &'a *const c_char) -> &'a CStr {
        CStr::from_bytes_with_nul_unchecked(::std::ffi::c_str_to_bytes_with_nul(raw))
    }

    /// Create a C-compatible string slice from a byte slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the last byte in the slice is not 0,
    /// or if any other bytes are 0.
    pub fn from_bytes_with_nul(v: &[u8]) -> &CStr {
        assert!(v[v.len() - 1] == 0 && !v[..v.len() - 1].iter().any(|&x| x == 0));
        unsafe { CStr::from_bytes_with_nul_unchecked(v) }
    }

    /// Create a C-compatible string slice from a byte slice.
    /// The slice must have a length >= 1, the last byte must be 0,
    /// and no other bytes may be 0.
    ///
    /// Violating these constraints causes undefined behavior.
    pub unsafe fn from_bytes_with_nul_unchecked(v: &[u8]) -> &CStr {
        // TODO: does this transmute have defined behavior?
        // we're relying on repr([u8]) = repr(CStr) here, newtypes would surely be helpful
        mem::transmute::<&[u8], &CStr>(v)
    }

    // as_ptr(), as_slice(): should be coming from Deref<Target=c_char>, but
    // we need to re-implement them to avoid an ICE (#16812 ?)
    pub fn as_ptr(&self) -> *const c_char {
        self.inner.as_ptr()
    }

    pub fn as_slice(&self) -> &[c_char] {
        self.inner.slice_to(self.inner.len() - 1)
    }

    /// Create a view into this C string which includes the trailing nul
    /// terminator at the end of the string.
    pub fn as_slice_with_nul(&self) -> &[c_char] {
        &self.inner
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { mem::transmute(self.as_slice()) }
    }

    pub fn as_bytes_with_nul(&self) -> &[u8] {
        unsafe { mem::transmute(self.as_slice_with_nul()) }
    }
    
    pub fn as_utf8(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(self.as_bytes())
    }
}

impl ops::Deref for CStr {
    type Target = [c_char];

    fn deref(&self) -> &[c_char] {
        // Does not underflow thanks to our invariant.
        // But rustc doesn't know that, so it may need some help to generate efficient code.
        self.inner.slice_to(self.inner.len() - 1)
    }
}

impl BorrowFrom<CString> for CStr {
    fn borrow_from(owned: &CString) -> &CStr {
        // This is safe because CStr and CString have the same invariant.
        unsafe { CStr::from_bytes_with_nul_unchecked(owned.as_bytes_with_nul()) }
    }
}

impl ToOwned<CString> for CStr {
    fn to_owned(&self) -> CString {
        // This is safe because CStr and CString have the same invariant.
        unsafe {
            CString::from_vec_unchecked(self.as_bytes_with_nul().to_owned())
        }
    }
}

impl fmt::Show for CStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        String::from_utf8_lossy(self.as_bytes()).fmt(f)
    }
}

#[macro_export]
macro_rules! cstr(
    ($s: tt) => (
        // TODO: verify that $s is a string literal without nuls,
        // and remove the runtime check by using from_bytes_with_nul_unchecked.
        $crate::cstr::CStr::from_bytes_with_nul(concat!($s, "\0").as_bytes())
    );
);

