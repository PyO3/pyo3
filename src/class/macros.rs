// Copyright (c) 2017-present PyO3 Project and Contributors

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_func {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject)
                                     -> *mut $crate::ffi::PyObject
            where T: $trait + PythonObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::_detail::handle_callback(LOCATION, $conv, |py| {
                let slf = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let ret = slf.$f(py);
                $crate::PyDrop::release_ref(slf, py);
                ret
            })
        }
        Some(wrap::<T>)
    }}
}

#[macro_export]
#[doc(hidden)]
    macro_rules! py_len_func {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject)
                                     -> $crate::ffi::Py_ssize_t
            where T: $trait + PythonObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::_detail::handle_callback(LOCATION, $conv, |py| {
                let slf = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let ret = slf.$f(py);
                $crate::PyDrop::release_ref(slf, py);
                ret
            })
        }
        Some(wrap::<T>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_func {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject,
                                     arg: *mut $crate::ffi::PyObject)
                                     -> *mut $crate::ffi::PyObject
            where T: $trait + PythonObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::_detail::handle_callback(LOCATION, $conv, |py| {
                let slf = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let arg = $crate::PyObject::from_borrowed_ptr(py, arg);
                let ret = slf.$f(py, &arg);
                $crate::PyDrop::release_ref(arg, py);
                $crate::PyDrop::release_ref(slf, py);
                ret
            })
        }
        Some(wrap::<T>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_func {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {
        py_ternary_func!($trait, $class::$f, $conv, *mut $crate::ffi::PyObject);
    };
    ($trait:ident, $class:ident :: $f:ident, $conv:expr, $res_type:ty) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject,
                                     arg1: *mut $crate::ffi::PyObject,
                                     arg2: *mut $crate::ffi::PyObject) -> $res_type
            where T: $trait + PythonObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::_detail::handle_callback(LOCATION, $conv, |py| {
                let slf = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let arg1 = $crate::PyObject::from_borrowed_ptr(py, arg1);
                let arg2 = $crate::PyObject::from_borrowed_ptr(py, arg2);
                let ret = slf.$f(py, &arg1, &arg2);
                $crate::PyDrop::release_ref(arg1, py);
                $crate::PyDrop::release_ref(arg2, py);
                $crate::PyDrop::release_ref(slf, py);
                ret
            })
        }
        Some(wrap::<T>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ssizearg_func {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject,
                             arg: $crate::Py_ssize_t) -> *mut $crate::ffi::PyObject
            where T: $trait + PythonObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::_detail::handle_callback(LOCATION, $conv, |py| {
                let slf = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let ret = slf.$f(py, arg as isize);
                $crate::PyDrop::release_ref(slf, py);
                ret
            })
        }
        Some(wrap::<T>)
    }}
}


#[macro_export]
#[doc(hidden)]
macro_rules! py_objobj_proc {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject,
                                     arg: *mut $crate::ffi::PyObject) -> $crate::c_int
            where T: $trait + PythonObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::_detail::handle_callback(LOCATION, $conv, |py| {
                let slf = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let arg = PyObject::from_borrowed_ptr(py, arg);
                let ret = slf.$f(py, &arg);
                $crate::PyDrop::release_ref(arg, py);
                $crate::PyDrop::release_ref(slf, py);
                ret
            })
        }
        Some(wrap::<T>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_slot {
    ($trait:ident, $class:ident :: $f:ident,
     $arg1_type:ty, $arg2_type:ty, $res_type:ty, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(
            slf: *mut $crate::ffi::PyObject,
            arg1: *mut $crate::ffi::PyObject,
            arg2: *mut $crate::ffi::PyObject) -> $res_type
            where T: $trait + PythonObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::_detail::handle_callback(
                LOCATION, $conv, |py|
                {
                    let slf = $crate::PyObject::from_borrowed_ptr(
                        py, slf).unchecked_cast_into::<T>();
                    let arg1 = $crate::PyObject::from_borrowed_ptr(py, arg1);

                    let tmp;
                    let value = if arg2.is_null() {
                        None
                    } else {
                        tmp = $crate::PyObject::from_borrowed_ptr(py, arg2);
                        Some(&tmp)
                    };

                    let ret = slf.$f(py, &arg1, value);

                    $crate::PyDrop::release_ref(arg1, py);
                    if ! arg2.is_null() {
                        $crate::PyDrop::release_ref(
                            $crate::PyObject::from_borrowed_ptr(py, arg2), py);
                    }
                    $crate::PyDrop::release_ref(slf, py);
                    ret
                })
        }
         Some(wrap::<T>)
    }}
}
