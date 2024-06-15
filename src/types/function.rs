use crate::c_str;
#[cfg(feature = "gil-refs")]
use crate::derive_utils::PyFunctionArguments;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::types::capsule::PyCapsuleMethods;
use crate::types::module::PyModuleMethods;
#[cfg(feature = "gil-refs")]
use crate::PyNativeType;
use crate::{
    ffi,
    impl_::pymethods::{self, PyMethodDef},
    types::{PyCapsule, PyDict, PyModule, PyString, PyTuple},
};
use crate::{Bound, IntoPy, Py, PyAny, PyResult, Python};
use std::cell::UnsafeCell;
use std::ffi::CStr;

/// Represents a builtin Python function object.
#[repr(transparent)]
pub struct PyCFunction(PyAny);

pyobject_native_type_core!(PyCFunction, pyobject_native_static_type_object!(ffi::PyCFunction_Type), #checkfunction=ffi::PyCFunction_Check);

impl PyCFunction {
    /// Deprecated form of [`PyCFunction::new_with_keywords_bound`]
    #[cfg(feature = "gil-refs")]
    #[deprecated(
        since = "0.21.0",
        note = "`PyCFunction::new_with_keywords` will be replaced by `PyCFunction::new_with_keywords_bound` in a future PyO3 version"
    )]
    pub fn new_with_keywords<'a>(
        fun: ffi::PyCFunctionWithKeywords,
        name: &'static CStr,
        doc: &'static CStr,
        py_or_module: PyFunctionArguments<'a>,
    ) -> PyResult<&'a Self> {
        let (py, module) = py_or_module.into_py_and_maybe_module();
        Self::internal_new(
            py,
            &PyMethodDef::cfunction_with_keywords(name, fun, doc),
            module.map(PyNativeType::as_borrowed).as_deref(),
        )
        .map(Bound::into_gil_ref)
    }

    /// Create a new built-in function with keywords (*args and/or **kwargs).
    ///
    /// To create `name` and `doc` static strings on Rust versions older than 1.77 (which added c"" literals),
    /// use the `c_str!` macro.
    pub fn new_with_keywords_bound<'py>(
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

    /// Deprecated form of [`PyCFunction::new`]
    #[cfg(feature = "gil-refs")]
    #[deprecated(
        since = "0.21.0",
        note = "`PyCFunction::new` will be replaced by `PyCFunction::new_bound` in a future PyO3 version"
    )]
    pub fn new<'a>(
        fun: ffi::PyCFunction,
        name: &'static CStr,
        doc: &'static CStr,
        py_or_module: PyFunctionArguments<'a>,
    ) -> PyResult<&'a Self> {
        let (py, module) = py_or_module.into_py_and_maybe_module();
        Self::internal_new(
            py,
            &PyMethodDef::noargs(name, fun, doc),
            module.map(PyNativeType::as_borrowed).as_deref(),
        )
        .map(Bound::into_gil_ref)
    }

    /// Create a new built-in function which takes no arguments.
    ///
    /// To create `name` and `doc` static strings on Rust versions older than 1.77 (which added c"" literals),
    /// use the [`c_str!`] macro.
    pub fn new_bound<'py>(
        py: Python<'py>,
        fun: ffi::PyCFunction,
        name: &'static CStr,
        doc: &'static CStr,
        module: Option<&Bound<'py, PyModule>>,
    ) -> PyResult<Bound<'py, Self>> {
        Self::internal_new(py, &PyMethodDef::noargs(name, fun, doc), module)
    }

    /// Deprecated form of [`PyCFunction::new_closure`]
    #[cfg(feature = "gil-refs")]
    #[deprecated(
        since = "0.21.0",
        note = "`PyCFunction::new_closure` will be replaced by `PyCFunction::new_closure_bound` in a future PyO3 version"
    )]
    pub fn new_closure<'a, F, R>(
        py: Python<'a>,
        name: Option<&'static str>,
        doc: Option<&'static str>,
        closure: F,
    ) -> PyResult<&'a PyCFunction>
    where
        F: Fn(&PyTuple, Option<&PyDict>) -> R + Send + 'static,
        R: crate::callback::IntoPyCallbackOutput<*mut ffi::PyObject>,
    {
        Self::new_closure_bound(py, name, doc, move |args, kwargs| {
            closure(args.as_gil_ref(), kwargs.map(Bound::as_gil_ref))
        })
        .map(Bound::into_gil_ref)
    }

    /// Create a new function from a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::{py_run, types::{PyCFunction, PyDict, PyTuple}};
    ///
    /// Python::with_gil(|py| {
    ///     let add_one = |args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>| -> PyResult<_> {
    ///         let i = args.extract::<(i64,)>()?.0;
    ///         Ok(i+1)
    ///     };
    ///     let add_one = PyCFunction::new_closure_bound(py, None, None, add_one).unwrap();
    ///     py_run!(py, add_one, "assert add_one(42) == 43");
    /// });
    /// ```
    pub fn new_closure_bound<'py, F, R>(
        py: Python<'py>,
        name: Option<&'static CStr>,
        doc: Option<&'static CStr>,
        closure: F,
    ) -> PyResult<Bound<'py, Self>>
    where
        F: Fn(&Bound<'_, PyTuple>, Option<&Bound<'_, PyDict>>) -> R + Send + 'static,
        R: crate::callback::IntoPyCallbackOutput<*mut ffi::PyObject>,
    {
        let name = name.unwrap_or(c_str!("pyo3-closure"));
        let doc = doc.unwrap_or(c_str!(""));
        let method_def =
            pymethods::PyMethodDef::cfunction_with_keywords(&name, run_closure::<F, R>, &doc);
        let def = method_def.as_method_def();

        let capsule = PyCapsule::new_bound(
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
                .downcast_into_unchecked()
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
            (mod_ptr, Some(m.name()?.into_py(py)))
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
                .downcast_into_unchecked()
        }
    }
}

static CLOSURE_CAPSULE_NAME: &'static CStr = c_str!("pyo3-closure");

unsafe extern "C" fn run_closure<F, R>(
    capsule_ptr: *mut ffi::PyObject,
    args: *mut ffi::PyObject,
    kwargs: *mut ffi::PyObject,
) -> *mut ffi::PyObject
where
    F: Fn(&Bound<'_, PyTuple>, Option<&Bound<'_, PyDict>>) -> R + Send + 'static,
    R: crate::callback::IntoPyCallbackOutput<*mut ffi::PyObject>,
{
    use crate::types::any::PyAnyMethods;

    crate::impl_::trampoline::cfunction_with_keywords(
        capsule_ptr,
        args,
        kwargs,
        |py, capsule_ptr, args, kwargs| {
            let boxed_fn: &ClosureDestructor<F> =
                &*(ffi::PyCapsule_GetPointer(capsule_ptr, CLOSURE_CAPSULE_NAME.as_ptr())
                    as *mut ClosureDestructor<F>);
            let args = Bound::ref_from_ptr(py, &args).downcast_unchecked::<PyTuple>();
            let kwargs = Bound::ref_from_ptr_or_opt(py, &kwargs)
                .as_ref()
                .map(|b| b.downcast_unchecked::<PyDict>());
            let result = (boxed_fn.closure)(args, kwargs);
            crate::callback::convert(py, result)
        },
    )
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
#[repr(transparent)]
#[cfg(all(not(Py_LIMITED_API), not(all(PyPy, not(Py_3_8)))))]
pub struct PyFunction(PyAny);

#[cfg(all(not(Py_LIMITED_API), not(all(PyPy, not(Py_3_8)))))]
pyobject_native_type_core!(PyFunction, pyobject_native_static_type_object!(ffi::PyFunction_Type), #checkfunction=ffi::PyFunction_Check);
