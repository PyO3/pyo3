// Copyright (c) 2017-present PyO3 Project and Contributors

/// Stringify a dotted path.
#[macro_export]
macro_rules! dot_stringify {
    ($e:ident) => (
        stringify!($e)
    );
    ($e:ident. $($es:ident).+) => (
        concat!(stringify!($e), ".", dot_stringify!($($es).*))
    );
}

/// Defines rust type for exception defined in Python code.
///
/// # Syntax
/// `import_exception!(module, MyError)`
///
/// * `module` is the name of the containing module.
/// * `MyError` is the name of the new exception type.
///
/// # Example
/// ```
/// #![feature(const_fn, const_ptr_null_mut)]
///
/// #[macro_use] extern crate pyo3;
///
/// use pyo3::{Python, PyDict};
///
/// import_exception!(socket, gaierror);
///
/// fn main() {
///     let gil = Python::acquire_gil();
///     let py = gil.python();
///     let ctx = PyDict::new(py);
///
///     ctx.set_item("gaierror", py.get_type::<gaierror>()).unwrap();
///     py.run("import socket; assert gaierror is socket.gaierror", None, Some(ctx)).unwrap();
/// }
/// ```
#[macro_export]
macro_rules! import_exception {
    ($($module:ident).+ , $name: ident) => {
        #[allow(non_camel_case_types)]
        pub struct $name;

        impl ::std::convert::From<$name> for $crate::PyErr {
            fn from(_err: $name) -> $crate::PyErr {
                $crate::PyErr::new::<$name, _>(())
            }
        }

        impl<T> ::std::convert::Into<$crate::PyResult<T>> for $name {
            fn into(self) -> $crate::PyResult<T> {
                $crate::PyErr::new::<$name, _>(()).into()
            }
        }

        impl $name {
            #[cfg_attr(feature = "cargo-clippy", allow(new_ret_no_self))]
            pub fn new<T: $crate::ToPyObject + 'static>(args: T) -> $crate::PyErr
                where Self: $crate::typeob::PyTypeObject + Sized
            {
                $crate::PyErr::new::<Self, T>(args)
            }
            pub fn into<R, T: $crate::ToPyObject + 'static>(args: T) -> $crate::PyResult<R>
                where Self: $crate::typeob::PyTypeObject + Sized
            {
                $crate::PyErr::new::<Self, T>(args).into()
            }
        }

        impl $crate::typeob::PyTypeObject for $name {
            #[inline(always)]
            fn init_type() {}

            #[inline]
            fn type_object() -> $crate::Py<$crate::PyType> {
                use $crate::IntoPyPointer;
                static mut TYPE_OBJECT: *mut $crate::ffi::PyTypeObject = ::std::ptr::null_mut();

                unsafe {
                    if TYPE_OBJECT.is_null() {
                        let gil = $crate::Python::acquire_gil();
                        let py = gil.python();

                        let imp = py.import(dot_stringify!($($module).*))
                            .expect(concat!(
                                "Can not import module: ", dot_stringify!($($module).*)));
                        let cls = imp.get(stringify!($name))
                            .expect(concat!(
                                "Can not load exception class: {}.{}", dot_stringify!($($module).*),
                                ".", stringify!($name)));
                        TYPE_OBJECT = cls.into_ptr() as *mut $crate::ffi::PyTypeObject;
                    }

                    $crate::Py::from_borrowed_ptr(
                        TYPE_OBJECT as *const _ as *mut $crate::ffi::PyObject)
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use {PyErr, Python};
    use objects::PyDict;

    import_exception!(socket, gaierror);
    import_exception!(email.errors, MessageError);

    #[test]
    fn test_check_exception() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let err: PyErr = gaierror.into();

        let d = PyDict::new(py);
        d.set_item("socket", py.import("socket").map_err(|e| e.print(py)).unwrap()).unwrap();
        d.set_item("exc", err).unwrap();

        py.run("assert isinstance(exc, socket.gaierror)", None, Some(d)).unwrap();
    }

    #[test]
    fn test_check_exception_nested() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let err: PyErr = MessageError.into();

        let d = PyDict::new(py);
        d.set_item("email", py.import("email").map_err(|e| e.print(py)).unwrap()).unwrap();
        d.set_item("exc", err).unwrap();

        py.run("assert isinstance(exc, email.errors.MessageError)", None, Some(d)).unwrap();
    }
}
