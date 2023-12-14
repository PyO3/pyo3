use crate::derive_utils::PyFunctionArguments;
use crate::methods::PyMethodDefDestructor;
use crate::prelude::*;
use crate::{
    ffi,
    impl_::pymethods::{self, PyMethodDef},
    types::{PyCapsule, PyDict, PyString, PyTuple},
};
use std::cell::UnsafeCell;
use std::ffi::CStr;

/// Represents a builtin Python function object.
#[repr(transparent)]
pub struct PyCFunction(PyAny);

pyobject_native_type_core!(PyCFunction, pyobject_native_static_type_object!(ffi::PyCFunction_Type), #checkfunction=ffi::PyCFunction_Check);

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
    /// # use pyo3::{py_run, types::{PyCFunction, PyDict, PyTuple}};
    ///
    /// Python::with_gil(|py| {
    ///     let add_one = |args: &PyTuple, _kwargs: Option<&PyDict>| -> PyResult<_> {
    ///         let i = args.extract::<(i64,)>()?.0;
    ///         Ok(i+1)
    ///     };
    ///     let add_one = PyCFunction::new_closure(py, None, None, add_one).unwrap();
    ///     py_run!(py, add_one, "assert add_one(42) == 43");
    /// });
    /// ```
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
        let method_def = pymethods::PyMethodDef::cfunction_with_keywords(
            name.unwrap_or("pyo3-closure\0"),
            pymethods::PyCFunctionWithKeywords(run_closure::<F, R>),
            doc.unwrap_or("\0"),
        );
        let (def, def_destructor) = method_def.as_method_def()?;

        let capsule = PyCapsule::new(
            py,
            ClosureDestructor::<F> {
                closure,
                def: UnsafeCell::new(def),
                def_destructor,
            },
            Some(closure_capsule_name().to_owned()),
        )?;

        // Safety: just created the capsule with type ClosureDestructor<F> above
        let data = unsafe { capsule.reference::<ClosureDestructor<F>>() };

        unsafe {
            py.from_owned_ptr_or_err::<PyCFunction>(ffi::PyCFunction_NewEx(
                data.def.get(),
                capsule.as_ptr(),
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
        let (mod_ptr, module_name): (_, Option<Py<PyString>>) = if let Some(m) = module {
            let mod_ptr = m.as_ptr();
            (mod_ptr, Some(m.name()?.into_py(py)))
        } else {
            (std::ptr::null_mut(), None)
        };
        let (def, destructor) = method_def.as_method_def()?;

        // FIXME: stop leaking the def and destructor
        let def = Box::into_raw(Box::new(def));
        std::mem::forget(destructor);

        let module_name_ptr = module_name
            .as_ref()
            .map_or(std::ptr::null_mut(), Py::as_ptr);

        unsafe {
            py.from_owned_ptr_or_err::<PyCFunction>(ffi::PyCFunction_NewEx(
                def,
                mod_ptr,
                module_name_ptr,
            ))
        }
    }
}

fn closure_capsule_name() -> &'static CStr {
    // TODO replace this with const CStr once MSRV new enough
    CStr::from_bytes_with_nul(b"pyo3-closure\0").unwrap()
}

unsafe extern "C" fn run_closure<F, R>(
    capsule_ptr: *mut ffi::PyObject,
    args: *mut ffi::PyObject,
    kwargs: *mut ffi::PyObject,
) -> *mut ffi::PyObject
where
    F: Fn(&PyTuple, Option<&PyDict>) -> R + Send + 'static,
    R: crate::callback::IntoPyCallbackOutput<*mut ffi::PyObject>,
{
    crate::impl_::trampoline::cfunction_with_keywords(
        capsule_ptr,
        args,
        kwargs,
        |py, capsule_ptr, args, kwargs| {
            let boxed_fn: &ClosureDestructor<F> =
                &*(ffi::PyCapsule_GetPointer(capsule_ptr, closure_capsule_name().as_ptr())
                    as *mut ClosureDestructor<F>);
            let args = py.from_borrowed_ptr::<PyTuple>(args);
            let kwargs = py.from_borrowed_ptr_or_opt::<PyDict>(kwargs);
            crate::callback::convert(py, (boxed_fn.closure)(args, kwargs))
        },
    )
}

struct ClosureDestructor<F> {
    closure: F,
    // Wrapped in UnsafeCell because Python C-API wants a *mut pointer
    // to this member.
    def: UnsafeCell<ffi::PyMethodDef>,
    // Used to destroy the cstrings in `def`, if necessary.
    #[allow(dead_code)]
    def_destructor: PyMethodDefDestructor,
}

// Safety: F is send and none of the fields are ever mutated
unsafe impl<F: Send> Send for ClosureDestructor<F> {}

/// Represents a Python function object.
#[repr(transparent)]
#[cfg(all(not(Py_LIMITED_API), not(all(PyPy, not(Py_3_8)))))]
pub struct PyFunction(PyAny);

#[cfg(all(not(Py_LIMITED_API), not(all(PyPy, not(Py_3_8)))))]
pyobject_native_type_core!(PyFunction, pyobject_native_static_type_object!(ffi::PyFunction_Type), #checkfunction=ffi::PyFunction_Check);
