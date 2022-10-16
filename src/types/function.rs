use crate::derive_utils::PyFunctionArguments;
use crate::exceptions::PyValueError;
use crate::impl_::panic::PanicTrap;
use crate::methods::PyMethodDefDestructor;
use crate::panic::PanicException;
use crate::{
    ffi,
    impl_::pymethods::{self, PyMethodDef},
    types, AsPyPointer,
};
use crate::{prelude::*, GILPool};
use std::mem::ManuallyDrop;
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
    crate::callback_body!(py, {
        let boxed_fn: &ClosureDestructor<F> =
            &*(ffi::PyCapsule_GetPointer(capsule_ptr, CLOSURE_CAPSULE_NAME.as_ptr() as *const _)
                as *mut ClosureDestructor<F>);
        let args = py.from_borrowed_ptr::<types::PyTuple>(args);
        let kwargs = py.from_borrowed_ptr_or_opt::<types::PyDict>(kwargs);
        (boxed_fn.closure)(args, kwargs)
    })
}

struct ClosureDestructor<F> {
    closure: F,
    def: ffi::PyMethodDef,
    // Used to destroy the cstrings in `def`, if necessary.
    #[allow(dead_code)]
    def_destructor: PyMethodDefDestructor,
}

unsafe extern "C" fn drop_closure<F, R>(capsule_ptr: *mut ffi::PyObject)
where
    F: Fn(&types::PyTuple, Option<&types::PyDict>) -> R + Send + 'static,
    R: crate::callback::IntoPyCallbackOutput<*mut ffi::PyObject>,
{
    let trap = PanicTrap::new("uncaught panic during drop_closure");
    let pool = GILPool::new();
    if let Err(payload) = std::panic::catch_unwind(|| {
        let destructor: Box<ClosureDestructor<F>> = Box::from_raw(ffi::PyCapsule_GetPointer(
            capsule_ptr,
            CLOSURE_CAPSULE_NAME.as_ptr() as *const _,
        )
            as *mut ClosureDestructor<F>);
        drop(destructor)
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
    ///     let add_one = types::PyCFunction::new_closure(add_one, py).unwrap();
    ///     py_run!(py, add_one, "assert add_one(42) == 43");
    /// });
    /// ```
    pub fn new_closure<F, R>(closure: F, py: Python<'_>) -> PyResult<&PyCFunction>
    where
        F: Fn(&types::PyTuple, Option<&types::PyDict>) -> R + Send + 'static,
        R: crate::callback::IntoPyCallbackOutput<*mut ffi::PyObject>,
    {
        let method_def = pymethods::PyMethodDef::cfunction_with_keywords(
            "pyo3-closure\0",
            pymethods::PyCFunctionWithKeywords(run_closure::<F, R>),
            "\0",
        );
        let (def, def_destructor) = method_def
            .as_method_def()
            .map_err(|err| PyValueError::new_err(err.0))?;
        let ptr = Box::into_raw(Box::new(ClosureDestructor {
            closure,
            def,
            // Disable the `ManuallyDrop`; we do actually want to drop this later.
            def_destructor: ManuallyDrop::into_inner(def_destructor),
        }));

        let destructor = unsafe {
            PyObject::from_owned_ptr_or_err(
                py,
                ffi::PyCapsule_New(
                    ptr as *mut c_void,
                    CLOSURE_CAPSULE_NAME.as_ptr() as *const _,
                    Some(drop_closure::<F, R>),
                ),
            )?
        };

        unsafe {
            py.from_owned_ptr_or_err::<PyCFunction>(ffi::PyCFunction_NewEx(
                #[cfg(addr_of)]
                core::ptr::addr_of_mut!((*ptr).def),
                #[cfg(not(addr_of))]
                &mut (*ptr).def,
                destructor.as_ptr(),
                std::ptr::null_mut(),
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
        let (def, _destructor) = method_def
            .as_method_def()
            .map_err(|err| PyValueError::new_err(err.0))?;

        let def = Box::into_raw(Box::new(def));

        unsafe {
            py.from_owned_ptr_or_err::<PyCFunction>(ffi::PyCFunction_NewEx(
                def,
                mod_ptr,
                module_name,
            ))
        }
    }
}

/// Represents a Python function object.
#[repr(transparent)]
#[cfg(not(any(PyPy, Py_LIMITED_API)))]
pub struct PyFunction(PyAny);

#[cfg(not(any(PyPy, Py_LIMITED_API)))]
pyobject_native_type_core!(PyFunction, ffi::PyFunction_Type, #checkfunction=ffi::PyFunction_Check);
