// Copyright (c) 2017-present PyO3 Project and Contributors

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_func {
    ($trait:ident, $class:ident :: $f:ident, $conv: expr) => {
        py_unary_func!($trait, $class::$f, $conv, *mut $crate::ffi::PyObject);
    };
    // Use call_ref! by default
    ($trait:ident, $class:ident :: $f:ident, $conv: expr, $ret_type:ty) => {
        py_unary_func!(
            $trait,
            $class::$f,
            $conv,
            $ret_type,
            call_ref_with_converter
        );
    };
    ($trait: ident,
     $class:ident :: $f:ident,
     $conv: expr,
     $ret_type: ty,
     $call: ident
    ) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> $ret_type
        where
            T: for<'p> $trait<'p>,
        {
            let py = $crate::Python::assume_gil_acquired();
            let _pool = $crate::GILPool::new(py);
            let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
            $call!(slf, $conv, py, $f)
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_unary_refmut_func {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut $crate::ffi::PyObject) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            let py = $crate::Python::assume_gil_acquired();
            let _pool = $crate::GILPool::new(py);
            let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
            let res = $class::$f(slf.borrow_mut()).into();
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
            let py = Python::assume_gil_acquired();
            let _pool = $crate::GILPool::new(py);
            let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
            let result = call_ref!(slf, $f);
            $crate::callback::cb_convert($conv, py, result)
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_func {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {
        py_binary_func!($trait, $class::$f, $conv, *mut $crate::ffi::PyObject)
    };
    // Use call_ref! by default
    ($trait:ident, $class:ident :: $f:ident, $conv:expr, $return:ty) => {{
        py_binary_func!($trait, $class::$f, $conv, $return, call_ref_with_converter)
    }};
    ($trait:ident, $class:ident :: $f:ident, $conv:expr, $return:ty, $call:ident) => {{
        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject, arg: *mut ffi::PyObject) -> $return
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;
            let py = $crate::Python::assume_gil_acquired();
            let _pool = $crate::GILPool::new(py);
            let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
            let arg = py.from_borrowed_ptr::<$crate::types::PyAny>(arg);
            $call!(slf, $conv, py, $f, arg)
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_binary_num_func {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(
            lhs: *mut ffi::PyObject,
            rhs: *mut ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;
            let py = $crate::Python::assume_gil_acquired();
            let _pool = $crate::GILPool::new(py);
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

            let py = $crate::Python::assume_gil_acquired();
            let _pool = $crate::GILPool::new(py);
            let slf_ = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
            let arg = py.from_borrowed_ptr::<$crate::types::PyAny>(arg);
            let result = call_mut!(slf_, $f, arg);
            match result {
                Ok(_) => {
                    ffi::Py_INCREF(slf);
                    slf
                }
                Err(e) => e.restore_and_null(py),
            }
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ssizearg_func {
    // Use call_ref! by default
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {
        py_ssizearg_func!(
            $trait,
            $class::$f,
            $conv,
            call_ref_with_converter
        )
    };
    ($trait:ident, $class:ident :: $f:ident, $conv:expr, $call:ident) => {{
        unsafe extern "C" fn wrap<T>(
            slf: *mut ffi::PyObject,
            arg: $crate::ffi::Py_ssize_t,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            let py = $crate::Python::assume_gil_acquired();
            let _pool = $crate::GILPool::new(py);
            let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
            $call!(slf, $conv, py, $f ;arg.into())
        }
        Some(wrap::<$class>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_func {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {
        py_ternary_func!($trait, $class::$f, $conv, *mut $crate::ffi::PyObject);
    };
    ($trait:ident, $class:ident :: $f:ident, $conv:expr, $return_type:ty) => {{
        unsafe extern "C" fn wrap<T>(
            slf: *mut $crate::ffi::PyObject,
            arg1: *mut $crate::ffi::PyObject,
            arg2: *mut $crate::ffi::PyObject,
        ) -> $return_type
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;

            let py = $crate::Python::assume_gil_acquired();
            let _pool = $crate::GILPool::new(py);
            let slf = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
            let arg1 = py.from_borrowed_ptr::<$crate::types::PyAny>(arg1);
            let arg2 = py.from_borrowed_ptr::<$crate::types::PyAny>(arg2);

            call_ref_with_converter!(slf, $conv, py, $f, arg1, arg2)
        }

        Some(wrap::<T>)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_ternary_num_func {
    ($trait:ident, $class:ident :: $f:ident, $conv:expr) => {{
        unsafe extern "C" fn wrap<T>(
            arg1: *mut $crate::ffi::PyObject,
            arg2: *mut $crate::ffi::PyObject,
            arg3: *mut $crate::ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject
        where
            T: for<'p> $trait<'p>,
        {
            use $crate::ObjectProtocol;

            let py = $crate::Python::assume_gil_acquired();
            let _pool = $crate::GILPool::new(py);
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

            let py = $crate::Python::assume_gil_acquired();
            let _pool = $crate::GILPool::new(py);
            let slf_cell = py.from_borrowed_ptr::<$crate::PyCell<T>>(slf);
            let arg1 = py.from_borrowed_ptr::<$crate::types::PyAny>(arg1);
            let arg2 = py.from_borrowed_ptr::<$crate::types::PyAny>(arg2);
            let result = call_mut!(slf_cell, $f, arg1, arg2);
            match result {
                Ok(_) => slf,
                Err(e) => e.restore_and_null(py),
            }
        }
        Some(wrap::<T>)
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

            let py = $crate::Python::assume_gil_acquired();
            let _pool = $crate::GILPool::new(py);
            let slf = py.from_borrowed_ptr::<$crate::PyCell<$generic>>(slf);

            let result = if value.is_null() {
                Err($crate::PyErr::new::<exceptions::NotImplementedError, _>(
                    format!(
                        "Subscript deletion not supported by {:?}",
                        stringify!($generic)
                    ),
                ))
            } else {
                let name = py.from_borrowed_ptr::<$crate::types::PyAny>(name);
                let value = py.from_borrowed_ptr::<$crate::types::PyAny>(value);
                call_mut!(slf, $fn_set, name, value)
            };
            match result {
                Ok(_) => 0,
                Err(e) => e.restore_and_minus1(py),
            }
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

            let py = $crate::Python::assume_gil_acquired();
            let _pool = $crate::GILPool::new(py);

            let result = if value.is_null() {
                let slf = py.from_borrowed_ptr::<$crate::PyCell<U>>(slf);
                let name = py.from_borrowed_ptr::<$crate::types::PyAny>(name);

                call_mut!(slf, $fn_del, name)
            } else {
                Err(PyErr::new::<exceptions::NotImplementedError, _>(
                    "Subscript assignment not supported",
                ))
            };
            match result {
                Ok(_) => 0,
                Err(e) => e.restore_and_minus1(py),
            }
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

            let py = $crate::Python::assume_gil_acquired();
            let _pool = $crate::GILPool::new(py);
            let slf = py.from_borrowed_ptr::<$crate::PyCell<$generic>>(slf);
            let name = py.from_borrowed_ptr::<$crate::types::PyAny>(name);

            let result = if value.is_null() {
                call_mut!(slf, $fn_del, name)
            } else {
                let value = py.from_borrowed_ptr::<$crate::types::PyAny>(value);
                call_mut!(slf, $fn_set, name, value)
            };
            match result {
                Ok(_) => 0,
                Err(e) => e.restore_and_minus1(py),
            }
        }
        Some(wrap::<$generic>)
    }};
}

macro_rules! _call_impl {
    ($slf: ident, $fn: ident $(; $args: expr)*) => { $slf.$fn($($args,)*).into() };
    ($slf: ident, $fn: ident, $raw_arg: expr $(,$raw_args: expr)* $(; $args: expr)*) => {
        match $raw_arg.extract() {
            Ok(arg) => _call_impl!($slf, $fn $(,$raw_args)* $(;$args)* ;arg),
            Err(e) => Err(e.into()),
        }
    };
}

/// Call `slf.try_borrow()?.$fn(...)`
macro_rules! call_ref {
    ($slf: expr, $fn: ident $(,$raw_args: expr)* $(; $args: expr)*) => {
        match $slf.try_borrow() {
            Ok(slf) => _call_impl!(slf, $fn $(,$raw_args)* $(;$args)*),
            Err(e) => Err(e.into()),
        }
    };
}

/// Call `slf.try_borrow()?.$fn(...)` and returns the result using the given CallbackConverter
macro_rules! call_ref_with_converter {
    ($slf: expr, $conv: expr, $py: ident, $fn: ident $(,$raw_args: expr)* $(; $args: expr)*) => {
        match $slf.try_borrow() {
            Ok(slf) => $crate::callback::cb_convert($conv, $py, _call_impl!(slf, $fn $(,$raw_args)* $(;$args)*)),
            Err(e) => $crate::callback::cb_err($conv, $py, e)
        }
    };
}

/// Call `slf.try_borrow_mut()?.$fn(...)`
macro_rules! call_mut {
    ($slf: expr, $fn: ident $(,$raw_args: expr)* $(; $args: expr)*) => {
        match $slf.try_borrow_mut() {
            Ok(mut slf) => _call_impl!(slf, $fn $(,$raw_args)* $(;$args)*),
            Err(e) => Err(e.into()),
        }
    };
}

/// Call `slf.try_borrow_mut()?.$fn(...)` and returns the result using the given CallbackConverter
macro_rules! call_mut_with_converter {
    ($slf: expr, $conv: expr, $py: ident, $fn: ident $(,$raw_args: expr)* $(; $args: expr)*) => {
        match $slf.try_borrow_mut() {
            Ok(mut slf) => $crate::callback::cb_convert($conv, $py, _call_impl!(slf, $fn $(,$raw_args)* $(;$args)*)),
            Err(e) => $crate::callback::cb_err($conv, $py, e)
        }
    };
}
