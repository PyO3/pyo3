use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::types::capsule::PyCapsuleMethods;
use crate::types::module::PyModuleMethods;
use crate::{
    ffi,
    impl_::pymethods::{self, PyMethodDef},
    types::{PyCapsule, PyDict, PyModule, PyString, PyTuple},
};
use crate::{Bound, Py, PyAny, PyResult, Python};
use std::cell::UnsafeCell;
use std::ffi::CStr;

/// Represents a builtin Python function object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyCFunction>`][crate::Py] or [`Bound<'py, PyCFunction>`][Bound].
#[repr(transparent)]
pub struct PyCFunction(PyAny);

pyobject_native_type_core!(PyCFunction, pyobject_native_static_type_object!(ffi::PyCFunction_Type), #checkfunction=ffi::PyCFunction_Check);

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
        Self::internal_new(
            py,
            &PyMethodDef::cfunction_with_keywords(name, fun, doc),
            module,
        )
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
        Self::internal_new(py, &PyMethodDef::noargs(name, fun, doc), module)
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
        let name = name.unwrap_or(ffi::c_str!("pyo3-closure"));
        let doc = doc.unwrap_or(ffi::c_str!(""));
        let method_def =
            pymethods::PyMethodDef::cfunction_with_keywords(name, run_closure::<F, R>, doc);
        let def = method_def.as_method_def();

        let capsule = PyCapsule::new(
            py,
            ClosureDestructor::<F> {
                closure,
                def: UnsafeCell::new(def),
            },
            Some(CLOSURE_CAPSULE_NAME.to_owned()),
        )?;

        // Safety: just created the capsule with type ClosureDestructor<F> above
        let data = unsafe { capsule.reference::<ClosureDestructor<F>>() };

        unsafe {
            ffi::PyCFunction_NewEx(data.def.get(), capsule.as_ptr(), std::ptr::null_mut())
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }
    }

    #[doc(hidden)]
    pub fn internal_new<'py>(
        py: Python<'py>,
        method_def: &PyMethodDef,
        module: Option<&Bound<'py, PyModule>>,
    ) -> PyResult<Bound<'py, Self>> {
        let (mod_ptr, module_name): (_, Option<Py<PyString>>) = if let Some(m) = module {
            let mod_ptr = m.as_ptr();
            (mod_ptr, Some(m.name()?.unbind()))
        } else {
            (std::ptr::null_mut(), None)
        };
        let def = method_def.as_method_def();

        // FIXME: stop leaking the def
        let def = Box::into_raw(Box::new(def));

        let module_name_ptr = module_name
            .as_ref()
            .map_or(std::ptr::null_mut(), Py::as_ptr);

        unsafe {
            ffi::PyCFunction_NewEx(def, mod_ptr, module_name_ptr)
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }
    }
}

static CLOSURE_CAPSULE_NAME: &CStr = ffi::c_str!("pyo3-closure");

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
        crate::impl_::trampoline::cfunction_with_keywords(
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
pyobject_native_type_core!(PyFunction, pyobject_native_static_type_object!(ffi::PyFunction_Type), #checkfunction=ffi::PyFunction_Check);
