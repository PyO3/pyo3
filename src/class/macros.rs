// Copyright (c) 2017-present PyO3 Project and Contributors

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_func {
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:ty) => {
        py_unary_func!($trait, $class::$f, $res_type, $conv, *mut $crate::ffi::PyObject);
    };
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:ty, $ret_type:ty) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> $ret_type
            where T: for<'p> $trait<'p> + $crate::Park<T>
        {
            use token::PythonPtr;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            let guard = $crate::callback::AbortOnDrop(LOCATION);
            let ret = $crate::std::panic::catch_unwind(|| {
                let py = $crate::Python::assume_gil_acquired();
                let slf = T::from_borrowed_ptr(slf);
                let result = {
                    let res = slf.as_mut(py).$f(py).into();

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
                };
                py.release(slf);
                result
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
            where T: for<'p> $trait<'p> + Park<T>
        {
            use token::PythonPtr;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            let guard = $crate::callback::AbortOnDrop(LOCATION);
            let ret = $crate::std::panic::catch_unwind(|| {
                let py = $crate::Python::assume_gil_acquired();
                let slf = T::from_borrowed_ptr(slf);
                let result = {
                    let res = slf.as_mut(py).$f(py).into();

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
                };
                py.release(slf);
                result
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
            where T: for<'p> $trait<'p> + $crate::Park<T>
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
            where T: for<'p> $trait<'p> + $crate::Park<T>
        {
            use token::PythonPtr;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            let guard = $crate::callback::AbortOnDrop(LOCATION);
            let ret = $crate::std::panic::catch_unwind(|| {
                let py = $crate::Python::assume_gil_acquired();
                let slf = T::from_borrowed_ptr(slf);
                let arg = $crate::PyObject::from_borrowed_ptr(py, arg);

                let result = {
                    let result = match arg.extract(py) {
                        Ok(arg) => {
                            slf.as_mut(py).$f(py, arg).into()
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
                };
                py.release(arg);
                py.release(slf);
                result
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
macro_rules! py_binary_self_func{
    ($trait:ident, $class:ident :: $f:ident) => {{
        #[allow(unused_mut)]
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                     arg: *mut ffi::PyObject) -> *mut $crate::ffi::PyObject
            where T: for<'p> $trait<'p> + $crate::Park<T>
        {
            use token::PythonPtr;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            let guard = $crate::callback::AbortOnDrop(LOCATION);
            let ret = $crate::std::panic::catch_unwind(|| {
                let py = $crate::Python::assume_gil_acquired();
                let slf1 = T::from_borrowed_ptr(slf);
                let arg = $crate::PyObject::from_borrowed_ptr(py, arg);

                let result = {
                    let result = match arg.extract(py) {
                        Ok(arg) => {
                            slf1.as_mut(py).$f(py, arg).into()
                        }
                        Err(e) => Err(e.into()),
                    };

                    match result {
                        Ok(_) => {
                            ffi::Py_INCREF(slf);
                            slf
                        }
                        Err(e) => {
                            e.restore(py);
                            $crate::std::ptr::null_mut()
                        }
                    }
                };
                py.release(arg);
                py.release(slf1);
                result
            });

            let ret = match ret {
                Ok(r) => r,
                Err(ref err) => {
                    $crate::callback::handle_panic($crate::Python::assume_gil_acquired(), err);
                    $crate::std::ptr::null_mut()
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
            where T: for<'p> $trait<'p> + $crate::Park<T>
        {
            use token::PythonPtr;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            let guard = $crate::callback::AbortOnDrop(LOCATION);
            let ret = $crate::std::panic::catch_unwind(|| {
                let py = $crate::Python::assume_gil_acquired();
                let slf = T::from_borrowed_ptr(slf);

                let result = {
                    let result = slf.as_mut(py).$f(py, arg as isize).into();
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
                };
                py.release(slf);
                result
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
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject,
                                     arg1: *mut $crate::ffi::PyObject,
                                     arg2: *mut $crate::ffi::PyObject) -> $return_type
            where T: for<'p> $trait<'p> + $crate::Park<T>
        {
            use token::PythonPtr;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            let guard = $crate::callback::AbortOnDrop(LOCATION);
            let ret = $crate::std::panic::catch_unwind(|| {
                let py = $crate::Python::assume_gil_acquired();
                let slf = T::from_borrowed_ptr(slf);
                let arg1 = $crate::PyObject::from_borrowed_ptr(py, arg1);
                let arg2 = $crate::PyObject::from_borrowed_ptr(py, arg2);

                let result = {
                    let result = match arg1.extract(py) {
                        Ok(arg1) => match arg2.extract(py) {
                            Ok(arg2) => slf.as_mut(py).$f(py, arg1, arg2).into(),
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
                };
                py.release(arg2);
                py.release(arg1);
                py.release(slf);
                result
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

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_self_func{
    ($trait:ident, $class:ident :: $f:ident) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject,
                                     arg1: *mut $crate::ffi::PyObject,
                                     arg2: *mut $crate::ffi::PyObject)
                                     -> *mut $crate::ffi::PyObject
            where T: for<'p> $trait<'p> + $crate::Park<T>
        {
            use token::PythonPtr;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            let guard = $crate::callback::AbortOnDrop(LOCATION);
            let ret = $crate::std::panic::catch_unwind(|| {
                let py = $crate::Python::assume_gil_acquired();
                let slf1 = T::from_borrowed_ptr(slf);
                let arg1 = $crate::PyObject::from_borrowed_ptr(py, arg1);
                let arg2 = $crate::PyObject::from_borrowed_ptr(py, arg2);

                let result = {
                    let result = match arg1.extract(py) {
                        Ok(arg1) => match arg2.extract(py) {
                            Ok(arg2) => slf1.as_mut(py).$f(py, arg1, arg2).into(),
                            Err(e) => Err(e.into())
                        },
                        Err(e) => Err(e.into()),
                    };

                    match result {
                        Ok(_) => slf,
                        Err(e) => {
                            e.restore(py);
                            $crate::std::ptr::null_mut()
                        }
                    }
                };
                py.release(arg2);
                py.release(arg1);
                py.release(slf1);
                result
            });

            let ret = match ret {
                Ok(r) => r,
                Err(ref err) => {
                    $crate::callback::handle_panic(
                        $crate::Python::assume_gil_acquired(), err);
                    $crate::std::ptr::null_mut()
                }
            };
            $crate::mem::forget(guard);
            ret
        }

         Some(wrap::<T>)
    }}
}


#[macro_export]
#[doc(hidden)]
macro_rules! py_func_set{
    ($trait:ident, $class:ident :: $f:ident) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject,
                                     name: *mut $crate::ffi::PyObject,
                                     value: *mut $crate::ffi::PyObject) -> $crate::c_int
            where T: for<'p> $trait<'p> + $crate::Park<T>
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::cb_unary_unit::<T, _>(LOCATION, slf, |py, slf| {
                if value.is_null() {
                    let e = PyErr::new::<exc::NotImplementedError, _>(
                        py, format!("Subscript deletion not supported by {:?}",
                                            stringify!(T)));
                    e.restore(py);
                    return -1
                } else {
                    let name = ::PyObject::from_borrowed_ptr(py, name);
                    let value = ::PyObject::from_borrowed_ptr(py, value);
                    let result = match name.extract(py) {
                        Ok(name) => match value.extract(py) {
                            Ok(value) => {
                                slf.$f(py, name, value).into()
                            },
                            Err(e) => Err(e.into()),
                        },
                        Err(e) => Err(e.into()),
                    };
                    py.release(value);
                    py.release(name);

                    match result {
                        Ok(_) =>
                            0,
                        Err(e) => {
                            e.restore(py);
                            -1
                        }
                    }
                }
            })
        }

         Some(wrap::<T>)
    }}
}


#[macro_export]
#[doc(hidden)]
macro_rules! py_func_del{
    ($trait:ident, $class:ident :: $f:ident) => {{
        #[allow(unused_mut)]
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject,
                                     name: *mut $crate::ffi::PyObject,
                                     value: *mut $crate::ffi::PyObject) -> $crate::c_int
            where T: for<'p> $trait<'p> + $crate::Park<T>
        {
            use token::PythonPtr;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            let guard = $crate::callback::AbortOnDrop(LOCATION);
            let ret = $crate::std::panic::catch_unwind(|| {
                let py = $crate::Python::assume_gil_acquired();

                if value.is_null() {
                    let slf = T::from_borrowed_ptr(slf);
                    let name = PyObject::from_borrowed_ptr(py, name);

                    let result = {
                        let result = match name.extract(py) {
                            Ok(name) =>
                                slf.as_mut(py).$f(py, name).into(),
                            Err(e) => Err(e.into()),
                        };
                        match result {
                            Ok(_) => 0,
                            Err(e) => {
                                e.restore(py);
                                -1
                            }
                        }
                    };
                    py.release(name);
                    py.release(slf);
                    result
                } else {
                    let e = PyErr::new::<exc::NotImplementedError, _>(
                        py, format!("Subscript assignment not supported by {:?}",
                                            stringify!(T)));
                    e.restore(py);
                    return -1

                }
            });

            let ret = match ret {
                Ok(r) => r,
                Err(ref err) => {
                    $crate::callback::handle_panic(
                        $crate::Python::assume_gil_acquired(), err);
                    -1
                }
            };
            $crate::mem::forget(guard);
            ret
        }

         Some(wrap::<T>)
    }}
}


#[macro_export]
#[doc(hidden)]
macro_rules! py_func_set_del{
    ($trait:ident, $trait2:ident, $class:ident :: $f:ident/$f2:ident) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject,
                                     name: *mut $crate::ffi::PyObject,
                                     value: *mut $crate::ffi::PyObject) -> $crate::c_int
            where T: for<'p> $trait<'p> + for<'p> $trait2<'p> + $crate::Park<T>
        {
            use token::PythonPtr;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            let guard = $crate::callback::AbortOnDrop(LOCATION);
            let ret = $crate::std::panic::catch_unwind(|| {
                let py = $crate::Python::assume_gil_acquired();
                let slf = T::from_borrowed_ptr(slf);
                let name = PyObject::from_borrowed_ptr(py, name);

                let result = {
                    if value.is_null() {
                        let result = match name.extract(py) {
                            Ok(name) =>
                                slf.as_mut(py).$f2(py, name).into(),
                            Err(e) => Err(e.into()),
                        };
                        match result {
                            Ok(_) => 0,
                            Err(e) => {
                                e.restore(py);
                                -1
                            }
                        }
                    } else {
                        let value = ::PyObject::from_borrowed_ptr(py, value);
                        let result = {
                            let result = match name.extract(py) {
                                Ok(name) => match value.extract(py) {
                                    Ok(value) => {
                                        slf.as_mut(py).$f(py, name, value).into()
                                    },
                                    Err(e) => Err(e.into()),
                                },
                                Err(e) => Err(e.into()),
                            };
                            match result {
                                Ok(_) => 0,
                                Err(e) => {
                                    e.restore(py);
                                    -1
                                }
                            }
                        };
                        py.release(value);
                        result
                    }
                };

                py.release(name);
                py.release(slf);
                result
            });

            let ret = match ret {
                Ok(r) => r,
                Err(ref err) => {
                    $crate::callback::handle_panic(
                        $crate::Python::assume_gil_acquired(), err);
                    -1
                }
            };
            $crate::mem::forget(guard);
            ret
        }

        Some(wrap::<T>)
    }}
}
