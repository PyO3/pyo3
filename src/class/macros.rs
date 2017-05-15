

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_slot {
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> $res_type
            where T: $trait + PythonObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::_detail::handle_callback(
                LOCATION, $conv,
                |py| {
                    let slf = $crate::PyObject::from_borrowed_ptr(
                        py, slf).unchecked_cast_into::<T>();
                    let ret = slf.$f(py);
                    $crate::PyDrop::release_ref(slf, py);
                    ret
                })
        }
        Some(wrap::<T>)
    }}
}
