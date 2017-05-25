// Copyright (c) 2017-present PyO3 Project and Contributors

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_func {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {
        py_unary_func!($trait, $class::$f, $conv, *mut $crate::ffi::PyObject);
    };
    ($trait:ident, $class:ident :: $f:ident, $conv:expr, $res_type:ty) => {{
        unsafe extern "C" fn wrap<'a, T>(slf: *mut $crate::ffi::PyObject) -> $res_type
            where T: $trait<'a>
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::handle(LOCATION, $conv, |py| {
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
        unsafe extern "C" fn wrap<'a, T>(slf: *mut $crate::ffi::PyObject)
                                         -> $crate::ffi::Py_ssize_t
            where T: $trait<'a>
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::handle(LOCATION, $conv, |py| {
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
        unsafe extern "C" fn wrap<'a, T>(slf: *mut ffi::PyObject,
                                         arg: *mut ffi::PyObject) -> *mut ffi::PyObject
            where T: $trait<'a>
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::handle(LOCATION, $conv, |py| {
                let arg = $crate::PyObject::from_borrowed_ptr(py, arg);

                match $crate::callback::unref(arg).extract() {
                    Ok(arg) => {
                        let slf: $crate::Py<T> = $crate::Py::from_borrowed_ptr(py, slf);
                        slf.as_ref().$f(arg).into()
                    }
                    Err(e) => Err(e.into()),
                }
            })
        }
        Some(wrap::<$class>)
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
            $crate::callback::handle(LOCATION, $conv, |py| {
                let slf: $crate::Py<T> = $crate::Py::from_borrowed_ptr(py, slf);
                slf.as_ref().$f(arg as isize).into()
            })
        }
        Some(wrap::<$class>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_func{
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {
        py_ternary_func!($trait, $class::$f, $conv, *mut $crate::ffi::PyObject);
    };
    ($trait:ident, $class:ident :: $f:ident, $conv:expr, $res_type: ty) => {{
        unsafe extern "C" fn wrap<'p, T>(slf: *mut $crate::ffi::PyObject,
                                         arg1: *mut $crate::ffi::PyObject,
                                         arg2: *mut $crate::ffi::PyObject) -> $res_type
            where T: $trait<'p>
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::handle(LOCATION, $conv, |py| {
                let slf: $crate::Py<T> = $crate::Py::from_borrowed_ptr(py, slf);
                let arg1 = $crate::PyObject::from_borrowed_ptr(py, arg1);
                let arg2 = $crate::PyObject::from_borrowed_ptr(py, arg2);

                match ::callback::unref(arg1).extract() {
                    Ok(arg1) => match ::callback::unref(arg2).extract() {
                        Ok(arg2) =>
                            slf.$f(arg1, arg2).into(),
                        Err(e) => Err(e),
                    },
                    Err(e) => Err(e)
                }
            })
        }
         Some(wrap::<T>)
    }}
}
