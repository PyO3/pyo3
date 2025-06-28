//! Defines how Python calls are dispatched, see [`PyCallArgs`].for more information.

use crate::ffi_ptr_ext::FfiPtrExt as _;
use crate::types::{PyAnyMethods as _, PyDict, PyString, PyTuple};
use crate::{ffi, Borrowed, Bound, IntoPyObjectExt as _, Py, PyAny, PyResult};

pub(crate) mod private {
    use super::*;

    pub trait Sealed {}

    impl Sealed for () {}
    impl Sealed for Bound<'_, PyTuple> {}
    impl Sealed for &'_ Bound<'_, PyTuple> {}
    impl Sealed for Py<PyTuple> {}
    impl Sealed for &'_ Py<PyTuple> {}
    impl Sealed for Borrowed<'_, '_, PyTuple> {}
    pub struct Token;
}

/// This trait marks types that can be used as arguments to Python function
/// calls.
///
/// This trait is currently implemented for Rust tuple (up to a size of 12),
/// [`Bound<'py, PyTuple>`] and [`Py<PyTuple>`]. Custom types that are
/// convertable to `PyTuple` via `IntoPyObject` need to do so before passing it
/// to `call`.
///
/// This trait is not intended to used by downstream crates directly. As such it
/// has no publicly available methods and cannot be implemented ouside of
/// `pyo3`. The corresponding public API is available through [`call`]
/// ([`call0`], [`call1`] and friends) on [`PyAnyMethods`].
///
/// # What is `PyCallArgs` used for?
/// `PyCallArgs` is used internally in `pyo3` to dispatch the Python calls in
/// the most optimal way for the current build configuration. Certain types,
/// such as Rust tuples, do allow the usage of a faster calling convention of
/// the Python interpreter (if available). More types that may take advantage
/// from this may be added in the future.
///
/// [`call0`]: crate::types::PyAnyMethods::call0
/// [`call1`]: crate::types::PyAnyMethods::call1
/// [`call`]: crate::types::PyAnyMethods::call
/// [`PyAnyMethods`]: crate::types::PyAnyMethods
#[cfg_attr(
    diagnostic_namespace,
    diagnostic::on_unimplemented(
        message = "`{Self}` cannot used as a Python `call` argument",
        note = "`PyCallArgs` is implemented for Rust tuples, `Bound<'py, PyTuple>` and `Py<PyTuple>`",
        note = "if your type is convertable to `PyTuple` via `IntoPyObject`, call `<arg>.into_pyobject(py)` manually",
        note = "if you meant to pass the type as a single argument, wrap it in a 1-tuple, `(<arg>,)`"
    )
)]
pub trait PyCallArgs<'py>: Sized + private::Sealed {
    #[doc(hidden)]
    fn call(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        kwargs: Borrowed<'_, 'py, PyDict>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>>;

    #[doc(hidden)]
    fn call_positional(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>>;

    #[doc(hidden)]
    fn call_method_positional(
        self,
        object: Borrowed<'_, 'py, PyAny>,
        method_name: Borrowed<'_, 'py, PyString>,
        _: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        object
            .getattr(method_name)
            .and_then(|method| method.call1(self))
    }
}

impl<'py> PyCallArgs<'py> for () {
    fn call(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        kwargs: Borrowed<'_, 'py, PyDict>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        let args = self.into_pyobject_or_pyerr(function.py())?;
        args.call(function, kwargs, token)
    }

    fn call_positional(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        let args = self.into_pyobject_or_pyerr(function.py())?;
        args.call_positional(function, token)
    }
}

impl<'py> PyCallArgs<'py> for Bound<'py, PyTuple> {
    #[inline]
    fn call(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        kwargs: Borrowed<'_, 'py, PyDict>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.as_borrowed().call(function, kwargs, token)
    }

    #[inline]
    fn call_positional(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.as_borrowed().call_positional(function, token)
    }
}

impl<'py> PyCallArgs<'py> for &'_ Bound<'py, PyTuple> {
    #[inline]
    fn call(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        kwargs: Borrowed<'_, 'py, PyDict>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.as_borrowed().call(function, kwargs, token)
    }

    #[inline]
    fn call_positional(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.as_borrowed().call_positional(function, token)
    }
}

impl<'py> PyCallArgs<'py> for Py<PyTuple> {
    #[inline]
    fn call(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        kwargs: Borrowed<'_, 'py, PyDict>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.bind_borrowed(function.py())
            .call(function, kwargs, token)
    }

    #[inline]
    fn call_positional(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.bind_borrowed(function.py())
            .call_positional(function, token)
    }
}

impl<'py> PyCallArgs<'py> for &'_ Py<PyTuple> {
    #[inline]
    fn call(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        kwargs: Borrowed<'_, 'py, PyDict>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.bind_borrowed(function.py())
            .call(function, kwargs, token)
    }

    #[inline]
    fn call_positional(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.bind_borrowed(function.py())
            .call_positional(function, token)
    }
}

impl<'py> PyCallArgs<'py> for Borrowed<'_, 'py, PyTuple> {
    #[inline]
    fn call(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        kwargs: Borrowed<'_, 'py, PyDict>,
        _: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        unsafe {
            ffi::PyObject_Call(function.as_ptr(), self.as_ptr(), kwargs.as_ptr())
                .assume_owned_or_err(function.py())
        }
    }

    #[inline]
    fn call_positional(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        _: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        unsafe {
            ffi::PyObject_Call(function.as_ptr(), self.as_ptr(), std::ptr::null_mut())
                .assume_owned_or_err(function.py())
        }
    }
}

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {
    use crate::{
        pyfunction,
        types::{PyDict, PyTuple},
        Py,
    };

    #[pyfunction(signature = (*args, **kwargs), crate = "crate")]
    fn args_kwargs(
        args: Py<PyTuple>,
        kwargs: Option<Py<PyDict>>,
    ) -> (Py<PyTuple>, Option<Py<PyDict>>) {
        (args, kwargs)
    }

    #[test]
    fn test_call() {
        use crate::{
            types::{IntoPyDict, PyAnyMethods, PyDict, PyTuple},
            wrap_pyfunction, Py, Python,
        };

        Python::attach(|py| {
            let f = wrap_pyfunction!(args_kwargs, py).unwrap();

            let args = PyTuple::new(py, [1, 2, 3]).unwrap();
            let kwargs = &[("foo", 1), ("bar", 2)].into_py_dict(py).unwrap();

            macro_rules! check_call {
                ($args:expr, $kwargs:expr) => {
                    let (a, k): (Py<PyTuple>, Py<PyDict>) = f
                        .call(args.clone(), Some(kwargs))
                        .unwrap()
                        .extract()
                        .unwrap();
                    assert!(a.is(&args));
                    assert!(k.is(kwargs));
                };
            }

            // Bound<'py, PyTuple>
            check_call!(args.clone(), kwargs);

            // &Bound<'py, PyTuple>
            check_call!(&args, kwargs);

            // Py<PyTuple>
            check_call!(args.clone().unbind(), kwargs);

            // &Py<PyTuple>
            check_call!(&args.as_unbound(), kwargs);

            // Borrowed<'_, '_, PyTuple>
            check_call!(args.as_borrowed(), kwargs);
        })
    }

    #[test]
    fn test_call_positional() {
        use crate::{
            types::{PyAnyMethods, PyNone, PyTuple},
            wrap_pyfunction, Py, Python,
        };

        Python::attach(|py| {
            let f = wrap_pyfunction!(args_kwargs, py).unwrap();

            let args = PyTuple::new(py, [1, 2, 3]).unwrap();

            macro_rules! check_call {
                ($args:expr, $kwargs:expr) => {
                    let (a, k): (Py<PyTuple>, Py<PyNone>) =
                        f.call1(args.clone()).unwrap().extract().unwrap();
                    assert!(a.is(&args));
                    assert!(k.is_none(py));
                };
            }

            // Bound<'py, PyTuple>
            check_call!(args.clone(), kwargs);

            // &Bound<'py, PyTuple>
            check_call!(&args, kwargs);

            // Py<PyTuple>
            check_call!(args.clone().unbind(), kwargs);

            // &Py<PyTuple>
            check_call!(args.as_unbound(), kwargs);

            // Borrowed<'_, '_, PyTuple>
            check_call!(args.as_borrowed(), kwargs);
        })
    }
}
