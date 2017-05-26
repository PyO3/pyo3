// Copyright (c) 2017-present PyO3 Project and Contributors

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_func {
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:ty) => {
        py_unary_func!($trait, $class::$f, $res_type, $conv, *mut $crate::ffi::PyObject);
    };
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:ty, $ret_type:ty) => {{
        #[allow(unused_mut)]
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> $ret_type
            where T: for<'p> $trait<'p>
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            let guard = $crate::callback::AbortOnDrop(LOCATION);
            let ret = $crate::std::panic::catch_unwind(|| {
                let py = $crate::Python::assume_gil_acquired();
                let mut slf = $crate::Py::<T>::from_borrowed_ptr(py, slf);
                let res = slf.$f(py).into();

                match res {
                    Ok(val) => {
                        <$conv as $crate::callback::CallbackConverter<$res_type>>
                            ::convert(val, py)
                    }
                    Err(e) => {
                        e.restore(py);
                        <$conv as $crate::callback::CallbackConverter<$res_type>>
                            ::error_value()
                    }
                }
            });

            let ret = match ret {
                Ok(r) => r,
                Err(ref err) => {
                    $crate::callback::handle_panic($crate::Python::assume_gil_acquired(), err);
                    <$conv as $crate::callback::CallbackConverter<$res_type>>
                        ::error_value()
                }
            };
            $crate::mem::forget(guard);
            ret
        }
        Some(wrap::<$class>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_func_self {
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:ty) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject)
                                     -> *mut $crate::ffi::PyObject
            where T: for<'p> $trait<'p> + $crate::ToPyObject + $crate::IntoPyObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            let guard = $crate::callback::AbortOnDrop(LOCATION);
            let ret = $crate::std::panic::catch_unwind(|| {
                let py = $crate::Python::assume_gil_acquired();
                let mut slf = $crate::Py::<T>::from_borrowed_ptr(py, slf);
                let res = slf.$f(py).into();

                match res {
                    Ok(val) => {
                        <$conv as $crate::callback::CallbackConverter<$res_type>>
                            ::convert(val, py)
                    }
                    Err(e) => {
                        e.restore(py);
                        <$conv as $crate::callback::CallbackConverter<$res_type>>
                            ::error_value()
                    }
                }
            });

            let ret = match ret {
                Ok(r) => r,
                Err(ref err) => {
                    $crate::callback::handle_panic($crate::Python::assume_gil_acquired(), err);
                    <$conv as $crate::callback::CallbackConverter<$res_type>>
                        ::error_value()
                }
            };
            $crate::mem::forget(guard);
            ret
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
            where T: for<'p> $trait<'p>
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::cb_unary::<T, _, _, _>(LOCATION, slf, $conv, |py, slf| {
                slf.$f(py).into()
            })
        }
        Some(wrap::<$class>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_func{
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:ty) => {
        py_binary_func!($trait, $class::$f, $res_type, $conv, *mut $crate::ffi::PyObject)
    };
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:ty, $return:ty) => {{
        #[allow(unused_mut)]
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                     arg: *mut ffi::PyObject) -> $return
            where T: for<'p> $trait<'p>
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            let guard = $crate::callback::AbortOnDrop(LOCATION);
            let ret = $crate::std::panic::catch_unwind(|| {
                let py = $crate::Python::assume_gil_acquired();
                let mut slf = $crate::Py::<T>::from_borrowed_ptr(py, slf);
                let arg = $crate::PyObject::from_borrowed_ptr(py, arg);

                let result = match arg.extract() {
                    Ok(arg) => {
                        slf.$f(py, arg).into()
                    }
                    Err(e) => Err(e.into()),
                };

                match result {
                    Ok(val) => {
                        <$conv as $crate::callback::CallbackConverter<$res_type>>
                            ::convert(val, py)
                    }
                    Err(e) => {
                        e.restore(py);
                        <$conv as $crate::callback::CallbackConverter<$res_type>>
                            ::error_value()
                    }
                }
            });

            let ret = match ret {
                Ok(r) => r,
                Err(ref err) => {
                    $crate::callback::handle_panic($crate::Python::assume_gil_acquired(), err);
                    <$conv as $crate::callback::CallbackConverter<$res_type>>
                        ::error_value()
                }
            };
            $crate::mem::forget(guard);
            ret
        }
        Some(wrap::<$class>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ssizearg_func {
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:ty) => {{
        #[allow(unused_mut)]
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                     arg: $crate::Py_ssize_t) -> *mut $crate::ffi::PyObject
            where T: for<'p> $trait<'p> + ToPyObject + IntoPyObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            let guard = $crate::callback::AbortOnDrop(LOCATION);
            let ret = $crate::std::panic::catch_unwind(|| {
                let py = $crate::Python::assume_gil_acquired();
                let mut slf = $crate::Py::<T>::from_borrowed_ptr(py, slf);

                let result = slf.$f(py, arg as isize).into();
                match result {
                    Ok(val) => {
                        <$conv as $crate::callback::CallbackConverter<$res_type>>
                            ::convert(val, py)
                    }
                    Err(e) => {
                        e.restore(py);
                        <$conv as $crate::callback::CallbackConverter<$res_type>>
                            ::error_value()
                    }
                }
            });

            let ret = match ret {
                Ok(r) => r,
                Err(ref err) => {
                    $crate::callback::handle_panic($crate::Python::assume_gil_acquired(), err);
                    <$conv as $crate::callback::CallbackConverter<$res_type>>
                        ::error_value()
                }
            };
            $crate::mem::forget(guard);
            ret
        }
        Some(wrap::<$class>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_func{
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:ty) => {
        py_ternary_func!($trait, $class::$f, $res_type, $conv, *mut $crate::ffi::PyObject);
    };
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:ty, $return_type:ty) => {{
        #[allow(unused_mut)]
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject,
                                     arg1: *mut $crate::ffi::PyObject,
                                     arg2: *mut $crate::ffi::PyObject) -> $return_type
            where T: for<'p> $trait<'p>
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            let guard = $crate::callback::AbortOnDrop(LOCATION);
            let ret = $crate::std::panic::catch_unwind(|| {
                let py = $crate::Python::assume_gil_acquired();
                let mut slf = $crate::Py::<T>::from_borrowed_ptr(py, slf);
                let arg1 = $crate::PyObject::from_borrowed_ptr(py, arg1);
                let arg2 = $crate::PyObject::from_borrowed_ptr(py, arg2);

                let result = match arg1.extract() {
                    Ok(arg1) => match arg2.extract() {
                        Ok(arg2) => slf.$f(py, arg1, arg2).into(),
                        Err(e) => Err(e.into())
                    },
                    Err(e) => Err(e.into()),
                };

                match result {
                    Ok(val) => {
                        <$conv as $crate::callback::CallbackConverter<$res_type>>
                            ::convert(val, py)
                    }
                    Err(e) => {
                        e.restore(py);
                        <$conv as $crate::callback::CallbackConverter<$res_type>>
                            ::error_value()
                    }
                }
            });

            let ret = match ret {
                Ok(r) => r,
                Err(ref err) => {
                    $crate::callback::handle_panic(
                        $crate::Python::assume_gil_acquired(), err);
                    <$conv as $crate::callback::CallbackConverter<$res_type>>
                        ::error_value()
                }
            };
            $crate::mem::forget(guard);
            ret
        }

         Some(wrap::<T>)
    }}
}
