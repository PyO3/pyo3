extern crate libc;
extern crate python3_sys;

unsafe fn get_str<'a>(s: *const libc::c_char) -> &'a str {
    let bytes = std::ffi::CStr::from_ptr(s).to_bytes();
    std::str::from_utf8(bytes).unwrap()
}

fn main() {
    unsafe {
        python3_sys::Py_Initialize();
        println!("{}", get_str(python3_sys::Py_GetVersion()));
    }
}

