// Copyright (c) 2017-present PyO3 Project and Contributors

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_func {
    ($trait: ident, $class:ident :: $f:ident, $call:ident, $ret_type: ty  $(, $conv:expr)?) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> $ret_type
        where
            T: for<'p> $trait<'p>,
        {
            let pool = $crate::GILPool::new();
            let py = pool.python();
            $crate::run_callback(py, || {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
                $crate::callback::convert(py, $call!(slf, $f)$(.map($conv))?)
            })
        }
        Some(wrap::<$class>)
    }};
    // Use call_ref! by default
    ($trait:ident, $class:ident :: $f:ident, $ret_type:ty $(, $conv:expr)?) => {
        py_unary_func!($trait, $class::$f, call_ref, $ret_type $(, $conv)?);
    };
    ($trait:ident, $class:ident :: $f:ident $(, $conv:expr)?) => {
        py_unary_func!($trait, $class::$f, call_ref, *mut $crate::ffi::PyObject $(, $conv)?);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_unarys_func {
    ($trait:ident, $class:ident :: $f:ident $(, $conv:expr)?) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            let pool = $crate::GILPool::new();
            let py = pool.python();
            $crate::run_callback(py, || {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
                let borrow = <T::Receiver>::try_from_pycell(slf)
                    .map_err(|e| e.into())?;
                let res = $class::$f(borrow).into();
                $crate::callback::convert(py, res $(.map($conv))?)
            })
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_len_func {
    ($trait:ident, $class:ident :: $f:ident) => {
        py_unary_func!(
            $trait,
            $class::$f,
            $crate::ffi::Py_ssize_t,
            $crate::callback::LenCallbackOutput
        )
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_func {
    // Use call_ref! by default
    ($trait:ident, $class:ident :: $f:ident, $return:ty, $call:ident $(, $conv:expr)?) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject, arg: *mut ffi::PyObject) -> $return
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;
            let pool = $crate::GILPool::new();
            let py = pool.python();
            $crate::run_callback(py, || {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
                let arg = py.from_borrowed_ptr::<$crate::PyAny>(arg);
                $crate::callback::convert(py, $call!(slf, $f, arg)$(.map($conv))?)
            })
        }
        Some(wrap::<$class>)
    }};
    ($trait:ident, $class:ident :: $f:ident, $return:ty $(, $conv:expr)?) => {
        py_binary_func!($trait, $class::$f, $return, call_ref $(, $conv)?)
    };
    ($trait:ident, $class:ident :: $f:ident $(, $conv:expr)?) => {
        py_binary_func!($trait, $class::$f, *mut $crate::ffi::PyObject $(, $conv)?)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_num_func {
    ($trait:ident, $class:ident :: $f:ident) => {{
        unsafe extern "C" fn wrap<T>(
            lhs: *mut ffi::PyObject,
            rhs: *mut ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;
            let pool = $crate::GILPool::new();
            let py = pool.python();
            $crate::run_callback(py, || {
                let lhs = py.from_borrowed_ptr::<$crate::PyAny>(lhs);
                let rhs = py.from_borrowed_ptr::<$crate::PyAny>(rhs);

                let result = $class::$f(lhs.extract()?, rhs.extract()?).into();
                $crate::callback::convert(py, result)
            })
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_reverse_num_func {
    ($trait:ident, $class:ident :: $f:ident) => {{
        unsafe extern "C" fn wrap<T>(
            lhs: *mut ffi::PyObject,
            rhs: *mut ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;
            let pool = $crate::GILPool::new();
            let py = pool.python();
            $crate::run_callback(py, || {
                // Swap lhs <-> rhs
                let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(rhs);
                let arg = py.from_borrowed_ptr::<$crate::PyAny>(lhs);
                $crate::callback::convert(
                    py,
                    $class::$f(&*slf.try_borrow()?, arg.extract()?).into(),
                )
            })
        }
        Some(wrap::<$class>)
    }};
}

// NOTE(kngwyu): This macro is used only for inplace operations, so I used call_mut here.
#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_self_func {
    ($trait:ident, $class:ident :: $f:ident) => {{
        unsafe extern "C" fn wrap<T>(
            slf: *mut ffi::PyObject,
            arg: *mut ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;

            let pool = $crate::GILPool::new();
            let py = pool.python();
            $crate::run_callback(py, || {
                let slf_ = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
                let arg = py.from_borrowed_ptr::<$crate::PyAny>(arg);
                call_mut!(slf_, $f, arg)?;
                ffi::Py_INCREF(slf);
                Ok(slf)
            })
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ssizearg_func {
    // Use call_ref! by default
    ($trait:ident, $class:ident :: $f:ident) => {
        py_ssizearg_func!($trait, $class::$f, call_ref)
    };
    ($trait:ident, $class:ident :: $f:ident, $call:ident) => {{
        unsafe extern "C" fn wrap<T>(
            slf: *mut ffi::PyObject,
            arg: $crate::ffi::Py_ssize_t,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            let pool = $crate::GILPool::new();
            let py = pool.python();
            $crate::run_callback(py, || {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
                $crate::callback::convert(py, $call!(slf, $f; arg.into()))
            })
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_func {
    ($trait:ident, $class:ident :: $f:ident, $return_type:ty) => {{
        unsafe extern "C" fn wrap<T>(
            slf: *mut $crate::ffi::PyObject,
            arg1: *mut $crate::ffi::PyObject,
            arg2: *mut $crate::ffi::PyObject,
        ) -> $return_type
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;

            let pool = $crate::GILPool::new();
            let py = pool.python();
            $crate::run_callback(py, || {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
                let arg1 = py
                    .from_borrowed_ptr::<$crate::types::PyAny>(arg1)
                    .extract()?;
                let arg2 = py
                    .from_borrowed_ptr::<$crate::types::PyAny>(arg2)
                    .extract()?;

                $crate::callback::convert(py, slf.try_borrow()?.$f(arg1, arg2).into())
            })
        }

        Some(wrap::<T>)
    }};
    ($trait:ident, $class:ident :: $f:ident) => {
        py_ternary_func!($trait, $class::$f, *mut $crate::ffi::PyObject);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_num_func {
    ($trait:ident, $class:ident :: $f:ident) => {{
        unsafe extern "C" fn wrap<T>(
            arg1: *mut $crate::ffi::PyObject,
            arg2: *mut $crate::ffi::PyObject,
            arg3: *mut $crate::ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;

            let pool = $crate::GILPool::new();
            let py = pool.python();
            $crate::run_callback(py, || {
                let arg1 = py
                    .from_borrowed_ptr::<$crate::types::PyAny>(arg1)
                    .extract()?;
                let arg2 = py
                    .from_borrowed_ptr::<$crate::types::PyAny>(arg2)
                    .extract()?;
                let arg3 = py
                    .from_borrowed_ptr::<$crate::types::PyAny>(arg3)
                    .extract()?;

                let result = $class::$f(arg1, arg2, arg3).into();
                $crate::callback::convert(py, result)
            })
        }

        Some(wrap::<T>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_reverse_num_func {
    ($trait:ident, $class:ident :: $f:ident) => {{
        unsafe extern "C" fn wrap<T>(
            arg1: *mut $crate::ffi::PyObject,
            arg2: *mut $crate::ffi::PyObject,
            arg3: *mut $crate::ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;
            let pool = $crate::GILPool::new();
            let py = pool.python();
            $crate::run_callback(py, || {
                // Swap lhs <-> rhs
                let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(arg2);
                let slf = slf.try_borrow()?;
                let arg1 = py.from_borrowed_ptr::<$crate::PyAny>(arg1);
                let arg2 = py.from_borrowed_ptr::<$crate::PyAny>(arg3);
                let result = $class::$f(&*slf, arg1.extract()?, arg2.extract()?).into();
                $crate::callback::convert(py, result)
            })
        }
        Some(wrap::<$class>)
    }};
}

// NOTE(kngwyu): Somehow __ipow__ causes SIGSEGV in Python < 3.8 when we extract arg2,
// so we ignore it. It's the same as what CPython does.
#[macro_export]
#[doc(hidden)]
macro_rules! py_dummy_ternary_self_func {
    ($trait:ident, $class:ident :: $f:ident) => {{
        unsafe extern "C" fn wrap<T>(
            slf: *mut $crate::ffi::PyObject,
            arg1: *mut $crate::ffi::PyObject,
            _arg2: *mut $crate::ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;

            let pool = $crate::GILPool::new();
            let py = pool.python();
            $crate::run_callback(py, || {
                let slf_cell = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
                let arg1 = py.from_borrowed_ptr::<$crate::PyAny>(arg1);
                call_mut!(slf_cell, $f, arg1)?;
                ffi::Py_INCREF(slf);
                Ok(slf)
            })
        }
        Some(wrap::<$class>)
    }};
}

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

            let pool = $crate::GILPool::new();
            let py = pool.python();
            $crate::run_callback(py, || {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<$generic>>(slf);

                if value.is_null() {
                    Err($crate::PyErr::new::<exceptions::NotImplementedError, _>(
                        format!(
                            "Subscript deletion not supported by {:?}",
                            stringify!($generic)
                        ),
                    ))
                } else {
                    let name = py.from_borrowed_ptr::<$crate::PyAny>(name);
                    let value = py.from_borrowed_ptr::<$crate::PyAny>(value);
                    crate::callback::convert(py, call_mut!(slf, $fn_set, name, value))
                }
            })
        }

        Some(wrap::<$generic>)
    }};
}

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

            let pool = $crate::GILPool::new();
            let py = pool.python();
            $crate::run_callback(py, || {
                if value.is_null() {
                    let slf = py.from_borrowed_ptr::<$crate::PyCell<U>>(slf);
                    let name = py
                        .from_borrowed_ptr::<$crate::types::PyAny>(name)
                        .extract()?;
                    $crate::callback::convert(py, slf.try_borrow_mut()?.$fn_del(name).into())
                } else {
                    Err(PyErr::new::<exceptions::NotImplementedError, _>(
                        "Subscript assignment not supported",
                    ))
                }
            })
        }

        Some(wrap::<$generic>)
    }};
}

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

            let pool = $crate::GILPool::new();
            let py = pool.python();
            $crate::run_callback(py, || {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<$generic>>(slf);
                let name = py.from_borrowed_ptr::<$crate::PyAny>(name);

                let result = if value.is_null() {
                    call_mut!(slf, $fn_del, name)
                } else {
                    let value = py.from_borrowed_ptr::<$crate::PyAny>(value);
                    call_mut!(slf, $fn_set, name, value)
                };
                $crate::callback::convert(py, result)
            })
        }
        Some(wrap::<$generic>)
    }};
}

macro_rules! _call_impl {
    ($slf: expr, $fn: ident $(; $args: expr)*) => {
        $slf.$fn($($args,)*).into()
    };
    ($slf: expr, $fn: ident, $raw_arg: expr $(,$raw_args: expr)* $(; $args: expr)*) => {
        _call_impl!($slf, $fn $(,$raw_args)* $(;$args)* ;$raw_arg.extract()?)
    };
}

/// Call `slf.try_borrow()?.$fn(...)`
macro_rules! call_ref {
    ($slf: expr, $fn: ident $(,$raw_args: expr)* $(; $args: expr)*) => {
        _call_impl!($slf.try_borrow()?, $fn $(,$raw_args)* $(;$args)*)
    };
}

/// Call `slf.try_borrow_mut()?.$fn(...)`
macro_rules! call_mut {
    ($slf: expr, $fn: ident $(,$raw_args: expr)* $(; $args: expr)*) => {
        _call_impl!($slf.try_borrow_mut()?, $fn $(,$raw_args)* $(;$args)*)
    };
}
