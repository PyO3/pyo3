use crate::derive_utils::PyFunctionArguments;
use crate::exceptions::PyValueError;
use crate::impl_::panic::PanicTrap;
use crate::panic::PanicException;
use crate::{
    ffi,
    impl_::pymethods::{self, PyMethodDef},
    types, AsPyPointer,
};
use crate::{prelude::*, GILPool};
use std::os::raw::c_void;

/// Represents a builtin Python function object.
#[repr(transparent)]
pub struct PyCFunction(PyAny);

pyobject_native_type_core!(PyCFunction, ffi::PyCFunction_Type, #checkfunction=ffi::PyCFunction_Check);

const CLOSURE_CAPSULE_NAME: &[u8] = b"pyo3-closure\0";

unsafe extern "C" fn run_closure<F, R>(
    capsule_ptr: *mut ffi::PyObject,
    args: *mut ffi::PyObject,
    kwargs: *mut ffi::PyObject,
) -> *mut ffi::PyObject
where
    F: Fn(&types::PyTuple, Option<&types::PyDict>) -> R + Send + 'static,
    R: crate::callback::IntoPyCallbackOutput<*mut ffi::PyObject>,
{
    crate::impl_::trampoline::cfunction_with_keywords(
        capsule_ptr,
        args,
        kwargs,
        |py, capsule_ptr, args, kwargs| {
            let boxed_fn: &F = &*(ffi::PyCapsule_GetPointer(
                capsule_ptr,
                CLOSURE_CAPSULE_NAME.as_ptr() as *const _,
            ) as *mut F);
            let args = py.from_borrowed_ptr::<types::PyTuple>(args);
            let kwargs = py.from_borrowed_ptr_or_opt::<types::PyDict>(kwargs);
            crate::callback::convert(py, boxed_fn(args, kwargs))
        },
    )
}

unsafe extern "C" fn drop_closure<F, R>(capsule_ptr: *mut ffi::PyObject)
where
    F: Fn(&types::PyTuple, Option<&types::PyDict>) -> R + Send + 'static,
    R: crate::callback::IntoPyCallbackOutput<*mut ffi::PyObject>,
{
    let trap = PanicTrap::new("uncaught panic during drop_closure");
    let pool = GILPool::new();
    if let Err(payload) = std::panic::catch_unwind(|| {
        let boxed_fn: Box<F> = Box::from_raw(ffi::PyCapsule_GetPointer(
            capsule_ptr,
            CLOSURE_CAPSULE_NAME.as_ptr() as *const _,
        ) as *mut F);
        drop(boxed_fn)
    }) {
        let py = pool.python();
        let err = PanicException::from_panic_payload(payload);
        err.write_unraisable(py, "when dropping a closure".into_py(py));
    };
    trap.disarm();
}

impl PyCFunction {
    /// Create a new built-in function with keywords (*args and/or **kwargs).
    pub fn new_with_keywords<'a>(
        fun: ffi::PyCFunctionWithKeywords,
        name: &'static str,
        doc: &'static str,
        py_or_module: PyFunctionArguments<'a>,
    ) -> PyResult<&'a Self> {
        Self::internal_new(
            &PyMethodDef::cfunction_with_keywords(
                name,
                pymethods::PyCFunctionWithKeywords(fun),
                doc,
            ),
            py_or_module,
        )
    }

    /// Create a new built-in function which takes no arguments.
    pub fn new<'a>(
        fun: ffi::PyCFunction,
        name: &'static str,
        doc: &'static str,
        py_or_module: PyFunctionArguments<'a>,
    ) -> PyResult<&'a Self> {
        Self::internal_new(
            &PyMethodDef::noargs(name, pymethods::PyCFunction(fun), doc),
            py_or_module,
        )
    }

    /// Create a new function from a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::{py_run, types};
    ///
    /// Python::with_gil(|py| {
    ///     let add_one = |args: &types::PyTuple, _kwargs: Option<&types::PyDict>| -> PyResult<_> {
    ///         let i = args.extract::<(i64,)>()?.0;
    ///         Ok(i+1)
    ///     };
    ///     let add_one = types::PyCFunction::new_closure(py, None, None, add_one).unwrap();
    ///     py_run!(py, add_one, "assert add_one(42) == 43");
    /// });
    /// ```
    pub fn new_closure<'a, F, R>(
        py: Python<'a>,
        name: Option<&'static str>,
        doc: Option<&'static str>,
        f: F,
    ) -> PyResult<&'a PyCFunction>
    where
        F: Fn(&types::PyTuple, Option<&types::PyDict>) -> R + Send + 'static,
        R: crate::callback::IntoPyCallbackOutput<*mut ffi::PyObject>,
    {
        let function_ptr = Box::into_raw(Box::new(f));
        let capsule = unsafe {
            PyObject::from_owned_ptr_or_err(
                py,
                ffi::PyCapsule_New(
                    function_ptr as *mut c_void,
                    CLOSURE_CAPSULE_NAME.as_ptr() as *const _,
                    Some(drop_closure::<F, R>),
                ),
            )?
        };
        let method_def = pymethods::PyMethodDef::cfunction_with_keywords(
            name.unwrap_or("pyo3-closure\0"),
            pymethods::PyCFunctionWithKeywords(run_closure::<F, R>),
            doc.unwrap_or("\0"),
        );
        Self::internal_new_from_pointers(&method_def, py, capsule.as_ptr(), std::ptr::null_mut())
    }

    #[doc(hidden)]
    fn internal_new_from_pointers<'py>(
        method_def: &PyMethodDef,
        py: Python<'py>,
        mod_ptr: *mut ffi::PyObject,
        module_name: *mut ffi::PyObject,
    ) -> PyResult<&'py Self> {
        let def = method_def
            .as_method_def()
            .map_err(|err| PyValueError::new_err(err.0))?;
        unsafe {
            py.from_owned_ptr_or_err::<PyCFunction>(ffi::PyCFunction_NewEx(
                Box::into_raw(Box::new(def)),
                mod_ptr,
                module_name,
            ))
        }
    }

    #[doc(hidden)]
    pub fn internal_new<'py>(
        method_def: &PyMethodDef,
        py_or_module: PyFunctionArguments<'py>,
    ) -> PyResult<&'py Self> {
        let (py, module) = py_or_module.into_py_and_maybe_module();
        let (mod_ptr, module_name) = if let Some(m) = module {
            let mod_ptr = m.as_ptr();
            let name: Py<PyAny> = m.name()?.into_py(py);
            (mod_ptr, name.as_ptr())
        } else {
            (std::ptr::null_mut(), std::ptr::null_mut())
        };
        Self::internal_new_from_pointers(method_def, py, mod_ptr, module_name)
    }
}

/// Represents a Python function object.
#[repr(transparent)]
#[cfg(not(any(PyPy, Py_LIMITED_API)))]
pub struct PyFunction(PyAny);

#[cfg(not(any(PyPy, Py_LIMITED_API)))]
pyobject_native_type_core!(PyFunction, ffi::PyFunction_Type, #checkfunction=ffi::PyFunction_Check);
