use crate::ffi_ptr_ext::FfiPtrExt;
use crate::impl_::pyfunction::create_py_c_function;
use crate::py_result_ext::PyResultExt;
use crate::types::capsule::PyCapsuleMethods;
use crate::{
    ffi,
    impl_::pymethods::{self, PyMethodDef},
    types::{PyCapsule, PyDict, PyModule, PyTuple},
};
use crate::{Bound, PyAny, PyResult, Python};
use std::cell::UnsafeCell;
use std::ffi::CStr;
use std::ptr::NonNull;

/// Represents a builtin Python function object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyCFunction>`][crate::Py] or [`Bound<'py, PyCFunction>`][Bound].
#[repr(transparent)]
pub struct PyCFunction(PyAny);

pyobject_native_type_core!(PyCFunction, pyobject_native_static_type_object!(ffi::PyCFunction_Type), "builtins", "builtin_function_or_method", #checkfunction=ffi::PyCFunction_Check);

impl PyCFunction {
    /// Create a new built-in function with keywords (*args and/or **kwargs).
    ///
    /// To create `name` and `doc` static strings on Rust versions older than 1.77 (which added c"" literals),
    /// use the [`c_str!`](crate::ffi::c_str) macro.
    pub fn new_with_keywords<'py>(
        py: Python<'py>,
        fun: ffi::PyCFunctionWithKeywords,
        name: &'static CStr,
        doc: &'static CStr,
        module: Option<&Bound<'py, PyModule>>,
    ) -> PyResult<Bound<'py, Self>> {
        let def = PyMethodDef::cfunction_with_keywords(name, fun, doc).into_raw();
        // FIXME: stop leaking the def
        let def = Box::leak(Box::new(def));
        // Safety: def is static
        unsafe { create_py_c_function(py, def, module) }
    }

    /// Create a new built-in function which takes no arguments.
    ///
    /// To create `name` and `doc` static strings on Rust versions older than 1.77 (which added c"" literals),
    /// use the [`c_str!`](crate::ffi::c_str) macro.
    pub fn new<'py>(
        py: Python<'py>,
        fun: ffi::PyCFunction,
        name: &'static CStr,
        doc: &'static CStr,
        module: Option<&Bound<'py, PyModule>>,
    ) -> PyResult<Bound<'py, Self>> {
        let def = PyMethodDef::noargs(name, fun, doc).into_raw();
        // FIXME: stop leaking the def
        let def = Box::leak(Box::new(def));
        // Safety: def is static
        unsafe { create_py_c_function(py, def, module) }
    }

    /// Create a new function from a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::{py_run, types::{PyCFunction, PyDict, PyTuple}};
    ///
    /// Python::attach(|py| {
    ///     let add_one = |args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>| -> PyResult<_> {
    ///         let i = args.extract::<(i64,)>()?.0;
    ///         Ok(i+1)
    ///     };
    ///     let add_one = PyCFunction::new_closure(py, None, None, add_one).unwrap();
    ///     py_run!(py, add_one, "assert add_one(42) == 43");
    /// });
    /// ```
    pub fn new_closure<'py, F, R>(
        py: Python<'py>,
        name: Option<&'static CStr>,
        doc: Option<&'static CStr>,
        closure: F,
    ) -> PyResult<Bound<'py, Self>>
    where
        F: Fn(&Bound<'_, PyTuple>, Option<&Bound<'_, PyDict>>) -> R + Send + 'static,
        for<'p> R: crate::impl_::callback::IntoPyCallbackOutput<'p, *mut ffi::PyObject>,
    {
        let name = name.unwrap_or(c"pyo3-closure");
        let doc = doc.unwrap_or(c"");
        let method_def =
            pymethods::PyMethodDef::cfunction_with_keywords(name, run_closure::<F, R>, doc);
        let def = method_def.into_raw();

        let capsule = PyCapsule::new(
            py,
            ClosureDestructor::<F> {
                closure,
                def: UnsafeCell::new(def),
            },
            Some(CLOSURE_CAPSULE_NAME.to_owned()),
        )?;

        let data: NonNull<ClosureDestructor<F>> =
            capsule.pointer_checked(Some(CLOSURE_CAPSULE_NAME))?.cast();

        // SAFETY: The capsule has just been created with the value, and will exist as long as
        // the function object exists.
        let method_def = unsafe { data.as_ref().def.get() };

        // SAFETY: The arguments to `PyCFunction_NewEx` are valid, we are attached to the
        // interpreter and we know the function either returns a new reference or errors.
        unsafe {
            ffi::PyCFunction_NewEx(method_def, capsule.as_ptr(), std::ptr::null_mut())
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }
    }
}

static CLOSURE_CAPSULE_NAME: &CStr = c"pyo3-closure";

unsafe extern "C" fn run_closure<F, R>(
    capsule_ptr: *mut ffi::PyObject,
    args: *mut ffi::PyObject,
    kwargs: *mut ffi::PyObject,
) -> *mut ffi::PyObject
where
    F: Fn(&Bound<'_, PyTuple>, Option<&Bound<'_, PyDict>>) -> R + Send + 'static,
    for<'py> R: crate::impl_::callback::IntoPyCallbackOutput<'py, *mut ffi::PyObject>,
{
    unsafe {
        crate::impl_::trampoline::cfunction_with_keywords::inner(
            capsule_ptr,
            args,
            kwargs,
            |py, capsule_ptr, args, kwargs| {
                let boxed_fn: &ClosureDestructor<F> =
                    &*(ffi::PyCapsule_GetPointer(capsule_ptr, CLOSURE_CAPSULE_NAME.as_ptr())
                        as *mut ClosureDestructor<F>);
                let args = Bound::ref_from_ptr(py, &args).cast_unchecked::<PyTuple>();
                let kwargs = Bound::ref_from_ptr_or_opt(py, &kwargs)
                    .as_ref()
                    .map(|b| b.cast_unchecked::<PyDict>());
                let result = (boxed_fn.closure)(args, kwargs);
                crate::impl_::callback::convert(py, result)
            },
        )
    }
}

struct ClosureDestructor<F> {
    closure: F,
    // Wrapped in UnsafeCell because Python C-API wants a *mut pointer
    // to this member.
    def: UnsafeCell<ffi::PyMethodDef>,
}

// Safety: F is send and none of the fields are ever mutated
unsafe impl<F: Send> Send for ClosureDestructor<F> {}

/// Represents a Python function object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyFunction>`][crate::Py] or [`Bound<'py, PyFunction>`][Bound].
#[repr(transparent)]
#[cfg(not(Py_LIMITED_API))]
pub struct PyFunction(PyAny);

#[cfg(not(Py_LIMITED_API))]
pyobject_native_type_core!(PyFunction, pyobject_native_static_type_object!(ffi::PyFunction_Type), "builtins", "function", #checkfunction=ffi::PyFunction_Check);
