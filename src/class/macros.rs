// Copyright (c) 2017-present PyO3 Project and Contributors

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_func {
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr) => {
        py_unary_func!(
            $trait,
            $class::$f,
            $res_type,
            $conv,
            *mut $crate::ffi::PyObject
        );
    };
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr, $ret_type:ty) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> $ret_type
        where
            T: for<'p> $trait<'p>,
        {
            let _pool = $crate::GILPool::new();
            let py = $crate::Python::assume_gil_acquired();
            let slf = py.mut_from_borrowed_ptr::<T>(slf);
            let res = slf.$f().into();
            $crate::callback::cb_convert($conv, py, res)
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_pyref_func {
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::instance::PyRefMut;
            let _pool = $crate::GILPool::new();
            let py = $crate::Python::assume_gil_acquired();
            let slf = py.mut_from_borrowed_ptr::<T>(slf);
            let res = $class::$f(PyRefMut::from_mut(slf)).into();
            $crate::callback::cb_convert($conv, py, res)
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_len_func {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> $crate::ffi::Py_ssize_t
        where
            T: for<'p> $trait<'p>,
        {
            let _pool = $crate::GILPool::new();
            let py = Python::assume_gil_acquired();
            let slf = py.mut_from_borrowed_ptr::<T>(slf);

            let result = slf.$f().into();
            $crate::callback::cb_convert($conv, py, result)
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_func {
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr) => {
        py_binary_func!(
            $trait,
            $class::$f,
            $res_type,
            $conv,
            *mut $crate::ffi::PyObject
        )
    };
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr, $return:ty) => {{
        #[allow(unused_mut)]
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject, arg: *mut ffi::PyObject) -> $return
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;
            let _pool = $crate::GILPool::new();
            let py = $crate::Python::assume_gil_acquired();
            let slf = py.mut_from_borrowed_ptr::<T>(slf);
            let arg = py.from_borrowed_ptr::<$crate::types::PyAny>(arg);

            let result = match arg.extract() {
                Ok(arg) => slf.$f(arg).into(),
                Err(e) => Err(e.into()),
            };
            $crate::callback::cb_convert($conv, py, result)
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_num_func {
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr) => {{
        #[allow(unused_mut)]
        unsafe extern "C" fn wrap<T>(
            lhs: *mut ffi::PyObject,
            rhs: *mut ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;
            let _pool = $crate::GILPool::new();
            let py = $crate::Python::assume_gil_acquired();
            let lhs = py.from_borrowed_ptr::<$crate::types::PyAny>(lhs);
            let rhs = py.from_borrowed_ptr::<$crate::types::PyAny>(rhs);

            let result = match lhs.extract() {
                Ok(lhs) => match rhs.extract() {
                    Ok(rhs) => $class::$f(lhs, rhs).into(),
                    Err(e) => Err(e.into()),
                },
                Err(e) => Err(e.into()),
            };
            $crate::callback::cb_convert($conv, py, result)
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_self_func {
    ($trait:ident, $class:ident :: $f:ident) => {{
        #[allow(unused_mut)]
        unsafe extern "C" fn wrap<T>(
            slf: *mut ffi::PyObject,
            arg: *mut ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;

            let _pool = $crate::GILPool::new();
            let py = $crate::Python::assume_gil_acquired();
            let slf1 = py.mut_from_borrowed_ptr::<T>(slf);
            let arg = py.from_borrowed_ptr::<$crate::types::PyAny>(arg);

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
                    std::ptr::null_mut()
                }
            }
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ssizearg_func {
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr) => {{
        #[allow(unused_mut)]
        unsafe extern "C" fn wrap<T>(
            slf: *mut ffi::PyObject,
            arg: $crate::ffi::Py_ssize_t,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            let _pool = $crate::GILPool::new();
            let py = $crate::Python::assume_gil_acquired();
            let slf = py.mut_from_borrowed_ptr::<T>(slf);
            let result = slf.$f(arg.into()).into();
            $crate::callback::cb_convert($conv, py, result)
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_func {
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr) => {
        py_ternary_func!(
            $trait,
            $class::$f,
            $res_type,
            $conv,
            *mut $crate::ffi::PyObject
        );
    };
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr, $return_type:ty) => {{
        unsafe extern "C" fn wrap<T>(
            slf: *mut $crate::ffi::PyObject,
            arg1: *mut $crate::ffi::PyObject,
            arg2: *mut $crate::ffi::PyObject,
        ) -> $return_type
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;

            let _pool = $crate::GILPool::new();
            let py = $crate::Python::assume_gil_acquired();
            let slf = py.mut_from_borrowed_ptr::<T>(slf);
            let arg1 = py.from_borrowed_ptr::<$crate::types::PyAny>(arg1);
            let arg2 = py.from_borrowed_ptr::<$crate::types::PyAny>(arg2);

            let result = match arg1.extract() {
                Ok(arg1) => match arg2.extract() {
                    Ok(arg2) => slf.$f(arg1, arg2).into(),
                    Err(e) => Err(e.into()),
                },
                Err(e) => Err(e.into()),
            };
            $crate::callback::cb_convert($conv, py, result)
        }

        Some(wrap::<T>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_num_func {
    ($trait:ident, $class:ident :: $f:ident, $res_type:ty, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(
            arg1: *mut $crate::ffi::PyObject,
            arg2: *mut $crate::ffi::PyObject,
            arg3: *mut $crate::ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;

            let _pool = $crate::GILPool::new();
            let py = $crate::Python::assume_gil_acquired();
            let arg1 = py.from_borrowed_ptr::<$crate::types::PyAny>(arg1);
            let arg2 = py.from_borrowed_ptr::<$crate::types::PyAny>(arg2);
            let arg3 = py.from_borrowed_ptr::<$crate::types::PyAny>(arg3);

            let result = match arg1.extract() {
                Ok(arg1) => match arg2.extract() {
                    Ok(arg2) => match arg3.extract() {
                        Ok(arg3) => $class::$f(arg1, arg2, arg3).into(),
                        Err(e) => Err(e.into()),
                    },
                    Err(e) => Err(e.into()),
                },
                Err(e) => Err(e.into()),
            };
            $crate::callback::cb_convert($conv, py, result)
        }

        Some(wrap::<T>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_self_func {
    ($trait:ident, $class:ident :: $f:ident) => {{
        unsafe extern "C" fn wrap<T>(
            slf: *mut $crate::ffi::PyObject,
            arg1: *mut $crate::ffi::PyObject,
            arg2: *mut $crate::ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;

            let _pool = $crate::GILPool::new();
            let py = $crate::Python::assume_gil_acquired();
            let slf1 = py.mut_from_borrowed_ptr::<T>(slf);
            let arg1 = py.from_borrowed_ptr::<$crate::types::PyAny>(arg1);
            let arg2 = py.from_borrowed_ptr::<$crate::types::PyAny>(arg2);

            let result = match arg1.extract() {
                Ok(arg1) => match arg2.extract() {
                    Ok(arg2) => slf1.$f(arg1, arg2).into(),
                    Err(e) => Err(e.into()),
                },
                Err(e) => Err(e.into()),
            };

            match result {
                Ok(_) => slf,
                Err(e) => {
                    e.restore(py);
                    std::ptr::null_mut()
                }
            }
        }
        Some(wrap::<T>)
    }};
}

#[doc(hidden)]
macro_rules! py_func_set {
    ($trait_name:ident, $generic:ident, $fn_set:ident) => {{
        unsafe extern "C" fn wrap<$generic>(
            slf: *mut $crate::ffi::PyObject,
            name: *mut $crate::ffi::PyObject,
            value: *mut $crate::ffi::PyObject,
        ) -> libc::c_int
        where
            T: for<'p> $trait_name<'p>,
        {
            use $crate::ObjectProtocol;

            let _pool = $crate::GILPool::new();
            let py = $crate::Python::assume_gil_acquired();
            let slf = py.mut_from_borrowed_ptr::<$generic>(slf);

            let result = if value.is_null() {
                Err($crate::PyErr::new::<exceptions::NotImplementedError, _>(
                    format!(
                        "Subscript deletion not supported by {:?}",
                        stringify!($generic)
                    ),
                ))
            } else {
                let name = py.mut_from_borrowed_ptr::<$crate::types::PyAny>(name);
                let value = py.from_borrowed_ptr::<$crate::types::PyAny>(value);
                match name.extract() {
                    Ok(name) => match value.extract() {
                        Ok(value) => slf.$fn_set(name, value).into(),
                        Err(e) => Err(e.into()),
                    },
                    Err(e) => Err(e.into()),
                }
            };
            match result {
                Ok(_) => 0,
                Err(e) => {
                    e.restore(py);
                    -1
                }
            }
        }

        Some(wrap::<$generic>)
    }};
}

#[doc(hidden)]
macro_rules! py_func_del {
    ($trait_name:ident, $generic:ident, $fn_del:ident) => {{
        unsafe extern "C" fn wrap<U>(
            slf: *mut $crate::ffi::PyObject,
            name: *mut $crate::ffi::PyObject,
            value: *mut $crate::ffi::PyObject,
        ) -> libc::c_int
        where
            U: for<'p> $trait_name<'p>,
        {
            use $crate::ObjectProtocol;

            let _pool = $crate::GILPool::new();
            let py = $crate::Python::assume_gil_acquired();

            let result = if value.is_null() {
                let slf = py.mut_from_borrowed_ptr::<U>(slf);
                let name = py.from_borrowed_ptr::<$crate::types::PyAny>(name);

                match name.extract() {
                    Ok(name) => slf.$fn_del(name).into(),
                    Err(e) => Err(e.into()),
                }
            } else {
                Err(PyErr::new::<exceptions::NotImplementedError, _>(
                    "Subscript assignment not supported",
                ))
            };
            match result {
                Ok(_) => 0,
                Err(e) => {
                    e.restore(py);
                    -1
                }
            }
        }

        Some(wrap::<$generic>)
    }};
}

#[doc(hidden)]
macro_rules! py_func_set_del {
    ($trait1:ident, $trait2:ident, $generic:ident, $fn_set:ident, $fn_del:ident) => {{
        unsafe extern "C" fn wrap<$generic>(
            slf: *mut $crate::ffi::PyObject,
            name: *mut $crate::ffi::PyObject,
            value: *mut $crate::ffi::PyObject,
        ) -> libc::c_int
        where
            T: for<'p> $trait1<'p> + for<'p> $trait2<'p>,
        {
            use $crate::ObjectProtocol;

            let _pool = $crate::GILPool::new();
            let py = $crate::Python::assume_gil_acquired();
            let slf = py.mut_from_borrowed_ptr::<$generic>(slf);
            let name = py.from_borrowed_ptr::<$crate::types::PyAny>(name);

            let result = if value.is_null() {
                match name.extract() {
                    Ok(name) => slf.$fn_del(name).into(),
                    Err(e) => Err(e.into()),
                }
            } else {
                let value = py.from_borrowed_ptr::<$crate::types::PyAny>(value);
                match name.extract() {
                    Ok(name) => match value.extract() {
                        Ok(value) => slf.$fn_set(name, value).into(),
                        Err(e) => Err(e.into()),
                    },
                    Err(e) => Err(e.into()),
                }
            };
            match result {
                Ok(_) => 0,
                Err(e) => {
                    e.restore(py);
                    -1
                }
            }
        }
        Some(wrap::<$generic>)
    }};
}
