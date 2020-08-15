// Copyright (c) 2017-present PyO3 Project and Contributors

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_func {
    ($trait: ident, $class:ident :: $f:ident, $call:ident, $ret_type: ty) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> $ret_type
        where
            T: for<'p> $trait<'p>,
        {
            $crate::callback_body!(py, {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
                $call!(slf, $f).convert(py)
            })
        }
        Some(wrap::<$class>)
    }};
    // Use call_ref! by default
    ($trait:ident, $class:ident :: $f:ident, $ret_type:ty) => {
        py_unary_func!($trait, $class::$f, call_ref, $ret_type);
    };
    ($trait:ident, $class:ident :: $f:ident) => {
        py_unary_func!($trait, $class::$f, call_ref, *mut $crate::ffi::PyObject);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_unarys_func {
    ($trait:ident, $class:ident :: $f:ident) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            $crate::callback_body!(py, {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
                let borrow =
                    <T::Receiver as $crate::derive_utils::TryFromPyCell<_>>::try_from_pycell(slf)
                        .map_err(|e| e.into())?;

                $class::$f(borrow).convert(py)
            })
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_len_func {
    ($trait:ident, $class:ident :: $f:ident) => {
        py_unary_func!($trait, $class::$f, $crate::ffi::Py_ssize_t)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_func {
    // Use call_ref! by default
    ($trait:ident, $class:ident :: $f:ident, $return:ty, $call:ident) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject, arg: *mut ffi::PyObject) -> $return
        where
            T: for<'p> $trait<'p>,
        {
            $crate::callback_body!(py, {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
                let arg = py.from_borrowed_ptr::<$crate::PyAny>(arg);
                $call!(slf, $f, arg).convert(py)
            })
        }
        Some(wrap::<$class>)
    }};
    ($trait:ident, $class:ident :: $f:ident, $return:ty) => {
        py_binary_func!($trait, $class::$f, $return, call_ref)
    };
    ($trait:ident, $class:ident :: $f:ident) => {
        py_binary_func!($trait, $class::$f, *mut $crate::ffi::PyObject)
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
            $crate::callback_body!(py, {
                let lhs = py.from_borrowed_ptr::<$crate::PyAny>(lhs);
                let rhs = extract_or_return_not_implemented!(py, rhs);
                $class::$f(lhs.extract()?, rhs).convert(py)
            })
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_reversed_num_func {
    ($trait:ident, $class:ident :: $f:ident) => {{
        unsafe extern "C" fn wrap<T>(
            lhs: *mut ffi::PyObject,
            rhs: *mut ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            $crate::callback_body!(py, {
                // Swap lhs <-> rhs
                let slf: &$crate::PyCell<T> = extract_or_return_not_implemented!(py, rhs);
                let arg = extract_or_return_not_implemented!(py, lhs);
                $class::$f(&*slf.try_borrow()?, arg).convert(py)
            })
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_fallbacked_num_func {
    ($class:ident, $lop_trait: ident :: $lop: ident, $rop_trait: ident :: $rop: ident) => {{
        unsafe extern "C" fn wrap<T>(
            lhs: *mut ffi::PyObject,
            rhs: *mut ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $lop_trait<'p> + for<'p> $rop_trait<'p>,
        {
            $crate::callback_body!(py, {
                let lhs = py.from_borrowed_ptr::<$crate::PyAny>(lhs);
                let rhs = py.from_borrowed_ptr::<$crate::PyAny>(rhs);
                // First, try the left hand method (e.g., __add__)
                match (lhs.extract(), rhs.extract()) {
                    (Ok(l), Ok(r)) => $class::$lop(l, r).convert(py),
                    _ => {
                        // Next, try the right hand method (e.g., __add__)
                        let slf: &$crate::PyCell<T> = extract_or_return_not_implemented!(rhs);
                        let arg = extract_or_return_not_implemented!(lhs);
                        $class::$rop(&*slf.try_borrow()?, arg).convert(py)
                    }
                }
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
            $crate::callback_body!(py, {
                let slf_ = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
                let arg = py.from_borrowed_ptr::<$crate::PyAny>(arg);
                call_operator_mut!(py, slf_, $f, arg).convert(py)?;
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
            $crate::callback_body!(py, {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
                $call!(slf, $f; arg.into()).convert(py)
            })
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternarys_func {
    ($trait:ident, $class:ident :: $f:ident, $return_type:ty) => {{
        unsafe extern "C" fn wrap<T>(
            slf: *mut $crate::ffi::PyObject,
            arg1: *mut $crate::ffi::PyObject,
            arg2: *mut $crate::ffi::PyObject,
        ) -> $return_type
        where
            T: for<'p> $trait<'p>,
        {
            $crate::callback_body!(py, {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
                let slf =
                    <T::Receiver as $crate::derive_utils::TryFromPyCell<_>>::try_from_pycell(slf)
                        .map_err(|e| e.into())?;
                let arg1 = py
                    .from_borrowed_ptr::<$crate::types::PyAny>(arg1)
                    .extract()?;
                let arg2 = py
                    .from_borrowed_ptr::<$crate::types::PyAny>(arg2)
                    .extract()?;

                $class::$f(slf, arg1, arg2).convert(py)
            })
        }

        Some(wrap::<T>)
    }};
    ($trait:ident, $class:ident :: $f:ident) => {
        py_ternarys_func!($trait, $class::$f, *mut $crate::ffi::PyObject);
    };
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
            $crate::callback_body!(py, {
                let slf_cell = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
                let arg1 = py.from_borrowed_ptr::<$crate::PyAny>(arg1);
                call_operator_mut!(py, slf_cell, $f, arg1).convert(py)?;
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
            $crate::callback_body!(py, {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<$generic>>(slf);

                if value.is_null() {
                    Err($crate::PyErr::new::<exceptions::PyNotImplementedError, _>(
                        format!(
                            "Subscript deletion not supported by {:?}",
                            stringify!($generic)
                        ),
                    ))
                } else {
                    let name = py.from_borrowed_ptr::<$crate::PyAny>(name);
                    let value = py.from_borrowed_ptr::<$crate::PyAny>(value);
                    call_mut!(slf, $fn_set, name, value).convert(py)
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
            $crate::callback_body!(py, {
                if value.is_null() {
                    let slf = py.from_borrowed_ptr::<$crate::PyCell<U>>(slf);
                    let name = py
                        .from_borrowed_ptr::<$crate::types::PyAny>(name)
                        .extract()?;
                    slf.try_borrow_mut()?.$fn_del(name).convert(py)
                } else {
                    Err(PyErr::new::<exceptions::PyNotImplementedError, _>(
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
            $crate::callback_body!(py, {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<$generic>>(slf);
                let name = py.from_borrowed_ptr::<$crate::PyAny>(name);

                if value.is_null() {
                    call_mut!(slf, $fn_del, name).convert(py)
                } else {
                    let value = py.from_borrowed_ptr::<$crate::PyAny>(value);
                    call_mut!(slf, $fn_set, name, value).convert(py)
                }
            })
        }
        Some(wrap::<$generic>)
    }};
}

macro_rules! extract_or_return_not_implemented {
    ($arg: ident) => {
        match $arg.extract() {
            Ok(value) => value,
            Err(_) => {
                let res = $crate::ffi::Py_NotImplemented();
                ffi::Py_INCREF(res);
                return Ok(res);
            }
        }
    };
    ($py: ident, $arg: ident) => {
        match $py
            .from_borrowed_ptr::<$crate::types::PyAny>($arg)
            .extract()
        {
            Ok(value) => value,
            Err(_) => return $py.NotImplemented().convert($py),
        }
    };
}

macro_rules! _call_impl {
    ($slf: expr, $fn: ident $(; $args: expr)*) => {
        $slf.$fn($($args,)*)
    };
    ($slf: expr, $fn: ident, $raw_arg: expr $(,$raw_args: expr)* $(; $args: expr)*) => {
        _call_impl!($slf, $fn $(,$raw_args)* $(;$args)* ;$raw_arg.extract()?)
    };
    (op $py:ident; $slf: expr, $fn: ident, $raw_arg: expr $(,$raw_args: expr)* $(; $args: expr)*) => {
        _call_impl!(
            $slf, $fn ;
            (match $raw_arg.extract() {
                Ok(res) => res,
                _=> return Ok($py.NotImplemented().convert($py)?)
            })
            $(;$args)*
        )
    }
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

macro_rules! call_operator_mut {
    ($py:ident, $slf: expr, $fn: ident $(,$raw_args: expr)* $(; $args: expr)*) => {
        _call_impl!(op $py; $slf.try_borrow_mut()?, $fn $(,$raw_args)* $(;$args)*)
    };
}
