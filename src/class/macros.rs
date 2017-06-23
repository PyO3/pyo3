// Copyright (c) 2017-present PyO3 Project and Contributors

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_func {
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr) => {
        py_unary_func!($trait, $class::$f, $res_type, $conv, *mut $crate::ffi::PyObject);
    };
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr, $ret_type:ty) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> $ret_type
            where T: for<'p> $trait<'p> + $crate::PyDowncastFrom
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            $crate::callback::cb_pyfunc::<_, _, $res_type>(LOCATION, $conv, |py| {
                let slf = py.mut_cast_from_borrowed_ptr::<T>(slf);
                let res = slf.$f().into();
                $crate::callback::cb_convert($conv, py, res)
            })
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
            where T: for<'p> $trait<'p> + $crate::PyDowncastFrom
        {
            use $crate::ObjectProtocol;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            $crate::callback::cb_pyfunc::<_, _, $res_type>(LOCATION, $conv, |py| {
                let slf = py.mut_cast_from_borrowed_ptr::<T>(slf);
                let res = slf.$f().into();
                $crate::callback::cb_convert($conv, py, res)
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
            where T: for<'p> $trait<'p> + $crate::PyDowncastFrom
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
                  $crate::callback::cb_unary::<T, _, _, _>(LOCATION, slf, $conv, |_, slf| {
                      slf.$f().into()
            })
        }
        Some(wrap::<$class>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_func{
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr) => {
        py_binary_func!($trait, $class::$f, $res_type, $conv, *mut $crate::ffi::PyObject)
    };
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr, $return:ty) => {{
        #[allow(unused_mut)]
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                     arg: *mut ffi::PyObject) -> $return
            where T: for<'p> $trait<'p> + $crate::PyDowncastFrom
        {
            use $crate::ObjectProtocol;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            $crate::callback::cb_pyfunc::<_, _, $res_type>(LOCATION, $conv, |py| {
                let slf = py.mut_cast_from_borrowed_ptr::<T>(slf);
                let arg = py.cast_from_borrowed_ptr::<$crate::PyInstance>(arg);

                let result = match arg.extract() {
                    Ok(arg) => slf.$f(arg).into(),
                    Err(e) => Err(e.into()),
                };
                $crate::callback::cb_convert($conv, py, result)
            })
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
            where T: for<'p> $trait<'p> + $crate::PyDowncastFrom
        {
            use $crate::ObjectProtocol;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            $crate::callback::cb_meth(LOCATION, |py| {
                let slf1 = py.mut_cast_from_borrowed_ptr::<T>(slf);
                let arg = py.cast_from_borrowed_ptr::<$crate::PyInstance>(arg);

                let result = match arg.extract() {
                    Ok(arg) => slf1.$f(arg).into(),
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
            })
        }
        Some(wrap::<$class>)
    }}
}


#[macro_export]
#[doc(hidden)]
macro_rules! py_ssizearg_func {
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr) => {{
        #[allow(unused_mut)]
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                     arg: $crate::Py_ssize_t) -> *mut $crate::ffi::PyObject
            where T: for<'p> $trait<'p> + $crate::PyDowncastFrom
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            $crate::callback::cb_meth(LOCATION, |py| {
                let slf = py.mut_cast_from_borrowed_ptr::<T>(slf);
                let result = slf.$f(arg as isize).into();
                $crate::callback::cb_convert($conv, py, result)
            })
        }
        Some(wrap::<$class>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_func{
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr) => {
        py_ternary_func!($trait, $class::$f, $res_type, $conv, *mut $crate::ffi::PyObject);
    };
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr, $return_type:ty) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject,
                                     arg1: *mut $crate::ffi::PyObject,
                                     arg2: *mut $crate::ffi::PyObject) -> $return_type
            where T: for<'p> $trait<'p> + $crate::PyDowncastFrom
        {
            use $crate::ObjectProtocol;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            $crate::callback::cb_pyfunc::<_, _, $res_type>(LOCATION, $conv, |py| {
                let slf = py.mut_cast_from_borrowed_ptr::<T>(slf);
                let arg1 = py.cast_from_borrowed_ptr::<$crate::PyInstance>(arg1);
                let arg2 = py.cast_from_borrowed_ptr::<$crate::PyInstance>(arg2);

                let result = match arg1.extract() {
                    Ok(arg1) => match arg2.extract() {
                        Ok(arg2) => slf.$f(arg1, arg2).into(),
                        Err(e) => Err(e.into())
                    },
                    Err(e) => Err(e.into()),
                };
                $crate::callback::cb_convert($conv, py, result)
            })
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
            where T: for<'p> $trait<'p>
        {
            use $crate::ObjectProtocol;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            $crate::callback::cb_meth(LOCATION, |py| {
                let slf1 = py.mut_cast_from_borrowed_ptr::<T>(slf);
                let arg1 = py.cast_from_borrowed_ptr::<$crate::PyInstance>(arg1);
                let arg2 = py.cast_from_borrowed_ptr::<$crate::PyInstance>(arg2);

                let result = match arg1.extract() {
                    Ok(arg1) => match arg2.extract() {
                        Ok(arg2) => slf1.$f(arg1, arg2).into(),
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
            })
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
            where T: for<'p> $trait<'p>
        {
            use $crate::ObjectProtocol;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::callback::cb_unary_unit::<T, _>(LOCATION, slf, |py, slf| {
                if value.is_null() {
                    let e = $crate::PyErr::new::<exc::NotImplementedError, _>(
                        py, format!("Subscript deletion not supported by {:?}", stringify!(T)));
                    e.restore(py);
                    -1
                } else {
                    let name = py.mut_cast_from_borrowed_ptr::<$crate::PyInstance>(name);
                    let value = py.cast_from_borrowed_ptr::<$crate::PyInstance>(value);
                    let result = match name.extract() {
                        Ok(name) => match value.extract() {
                            Ok(value) =>
                                slf.$f(name, value).into(),
                            Err(e) => Err(e.into()),
                        },
                        Err(e) => Err(e.into()),
                    };
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
            where T: for<'p> $trait<'p>
        {
            use $crate::ObjectProtocol;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            $crate::callback::cb_pyfunc::<_, _, ()>(
                LOCATION, $crate::callback::UnitCallbackConverter, |py|
            {
                if value.is_null() {
                    let slf = py.mut_cast_from_borrowed_ptr::<T>(slf);
                    let name = py.cast_from_borrowed_ptr::<$crate::PyInstance>(name);

                    let result = match name.extract() {
                        Ok(name) => slf.$f(name).into(),
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
                    let e = PyErr::new::<exc::NotImplementedError, _>(
                        py, format!("Subscript assignment not supported by {:?}",
                                            stringify!(T)));
                    e.restore(py);
                    -1
                }
            })
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
            where T: for<'p> $trait<'p> + for<'p> $trait2<'p> + $crate::PyDowncastFrom
        {
            use $crate::ObjectProtocol;
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");

            $crate::callback::cb_pyfunc::<_, _, ()>(
                LOCATION, $crate::callback::UnitCallbackConverter, |py|
            {
                let slf = py.mut_cast_from_borrowed_ptr::<T>(slf);
                let name = py.cast_from_borrowed_ptr::<$crate::PyInstance>(name);

                if value.is_null() {
                    let result = match name.extract() {
                        Ok(name) => slf.$f2(name).into(),
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
                    let value = py.cast_from_borrowed_ptr::<$crate::PyInstance>(value);
                    let result = match name.extract() {
                        Ok(name) => match value.extract() {
                            Ok(value) => {
                                slf.$f(name, value).into()
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
                }
            })
        }
        Some(wrap::<T>)
    }}
}
