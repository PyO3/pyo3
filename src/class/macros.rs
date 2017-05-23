// Copyright (c) 2017-present PyO3 Project and Contributors

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_func {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {
        py_unary_func!($trait, $class::$f, $conv, *mut $crate::ffi::PyObject);
    };
    ($trait:ident, $class:ident :: $f:ident, $conv:expr, $res_type:ty) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> $res_type
            where T: $trait + PythonObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::handle_callback(LOCATION, $conv, |py| {
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
macro_rules! py_unary_func_ {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {
        py_unary_func_!($trait, $class::$f, $conv, *mut $crate::ffi::PyObject);
    };
    ($trait:ident, $class:ident :: $f:ident, $conv:expr, $res_type:ty) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> $res_type
            where T: $trait
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::handle_callback(LOCATION, $conv, |py| {
                let slf = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let ret = slf.$f(py).into();
                $crate::PyDrop::release_ref(slf, py);
                ret
            })
        }
        Some(wrap::<$class>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_func_2 {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {
        py_unary_func_2!($trait, $class::$f, $conv, *mut $crate::ffi::PyObject);
    };
    ($trait:ident, $class:ident :: $f:ident, $conv:expr, $res_type:ty) => {{
        unsafe extern "C" fn wrap<'a, T>(slf: *mut $crate::ffi::PyObject) -> $res_type
            where T: $trait<'a> + $crate::class::typeob::PyTypeInfo
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::handle_callback2(LOCATION, $conv, |py| {
                let slf: $crate::Py<T> = $crate::Py::from_borrowed_ptr(py, slf);
                slf.as_ref().$f().into()
            })
        }
        Some(wrap::<$class>)
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
            $crate::callback::handle_callback(LOCATION, $conv, |py| {
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
macro_rules! py_len_func_ {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> $crate::ffi::Py_ssize_t
            where T: $trait
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::handle_callback(LOCATION, $conv, |py| {
                let slf = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let ret = slf.$f(py);
                $crate::PyDrop::release_ref(slf, py);
                ret.into()
            })
        }
        Some(wrap::<$class>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_len_func2 {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {{
        unsafe extern "C" fn wrap<'a, T>(slf: *mut $crate::ffi::PyObject)
                                         -> $crate::ffi::Py_ssize_t
            where T: $trait<'a>
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::handle_callback2(LOCATION, $conv, |py| {
                let slf: $crate::Py<T> = $crate::Py::from_borrowed_ptr(py, slf);
                slf.as_ref().$f().into()
            })
        }
        Some(wrap::<$class>)
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
            $crate::callback::handle_callback(LOCATION, $conv, |py| {
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
    macro_rules! py_binary_func_ {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject,
                                     arg: *mut $crate::ffi::PyObject)
                                     -> *mut $crate::ffi::PyObject
            where T: $trait
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::handle_callback(LOCATION, $conv, |py| {
                let slf = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let arg = $crate::PyObject::from_borrowed_ptr(py, arg);
                let ret = match arg.extract(py) {
                    Ok(arg) => slf.$f(py, arg).into(),
                    Err(e) => Err(e),
                };
                $crate::PyDrop::release_ref(arg, py);
                $crate::PyDrop::release_ref(slf, py);
                ret
            })
        }
        Some(wrap::<$class>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_func_2 {
    ($trait:ident, $class:ident :: $f:ident, $arg:ident, $conv:expr) => {{
        unsafe extern "C" fn wrap<'a, T>(slf: *mut ffi::PyObject,
                                         arg: *mut ffi::PyObject) -> *mut ffi::PyObject
            where T: $trait<'a>
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::handle_callback2(LOCATION, $conv, |py| {
                match $crate::Py::<T::$arg>::cast_from_borrowed(py, arg) {
                    Ok(arg) => {
                        let arg1: &$crate::Py<T::$arg> = {&arg as *const _}.as_ref().unwrap();
                        match arg1.extr() {
                            Ok(arg) => {
                                let slf: $crate::Py<T> = $crate::Py::from_borrowed_ptr(py, slf);
                                slf.as_ref().$f(arg).into()
                            }
                            Err(e) => Err(e.into()),
                        }
                    },
                    Err(e) => Err(e.into()),
                }
            })
        }
        Some(wrap::<$class>)
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
            $crate::callback::handle_callback(LOCATION, $conv, |py| {
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
        unsafe extern "C" fn wrap<'a, T>(slf: *mut $crate::ffi::PyObject,
                                     arg: $crate::Py_ssize_t)
                                     -> *mut $crate::ffi::PyObject
            where T: $trait<'a>
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::handle_callback2(LOCATION, $conv, |py| {
                let slf: $crate::Py<T> = $crate::Py::from_borrowed_ptr(py, slf);
                slf.as_ref().$f(arg as isize).into()
            })
        }
        Some(wrap::<$class>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_func {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {
        py_ternary_func!($trait, $class::$f, $conv, *mut $crate::ffi::PyObject);
    };
    ($trait:ident, $class:ident :: $f:ident, $conv:expr, $res_type: ty) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject,
                                     arg1: *mut $crate::ffi::PyObject,
                                     arg2: *mut $crate::ffi::PyObject) -> $res_type
            where T: $trait + PythonObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::handle_callback(LOCATION, $conv, |py| {
                let slf = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let arg1 = $crate::PyObject::from_borrowed_ptr(py, arg1);
                let arg2 = $crate::PyObject::from_borrowed_ptr(py, arg2);

                let ret = match arg1.extract(py) {
                    Ok(arg1) => match arg2.extract(py) {
                        Ok(arg2) =>
                            slf.$f(py, arg1, arg2).into(),
                        Err(e) => Err(e),
                    },
                    Err(e) => Err(e)
                };

                $crate::PyDrop::release_ref(arg2, py);
                $crate::PyDrop::release_ref(arg1, py);
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
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {
        py_ternary_slot!($trait, $class::$f, $conv, *mut $crate::ffi::PyObject);
    };
    ($trait:ident, $class:ident :: $f:ident,
     $arg1_type:ty, $arg2_type:ty, $conv:expr, $res_type: ty) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject,
                                     arg1: *mut $crate::ffi::PyObject,
                                     arg2: *mut $crate::ffi::PyObject) -> $res_type
            where T: $trait
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::handle_callback(LOCATION, $conv, |py| {
                let slf = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let arg1 = $crate::PyObject::from_borrowed_ptr(py, arg1);

                let tmp;
                let value = if arg2.is_null() {
                    None
                } else {
                    tmp = $crate::PyObject::from_borrowed_ptr(py, arg2);
                    Some(&tmp)
                };

                let ret = slf.$f(py, &arg1, value).into();

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
