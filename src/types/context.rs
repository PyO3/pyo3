//! Safe Rust wrappers for types defined in the Python `contextvars` library
//!
//! For more details about these types, see the [Python
//! documentation](https://docs.python.org/3/library/contextvars.html)

use crate::err::PyResult;
use crate::sync::GILOnceCell;
use crate::{ffi, Py, PyTypeInfo};
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::{Bound, BoundObject, IntoPyObject, PyAny, PyErr, Python};
use std::ffi::CStr;
use std::ptr;

use super::{PyAnyMethods, PyString};

/// Implementation of functionality for [`PyContext`].
///
/// These methods are defined for the `Bound<'py, PyContext>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyContext")]
pub trait PyContextMethods<'py>: crate::sealed::Sealed {
    /// Return a shallow copy of the context object.
    fn copy(&self) -> PyResult<Bound<'py, PyContext>>;
    /// Set ctx as the current context for the current thread
    fn enter(&self) -> PyResult<()>;
    /// Deactivate the context and restore the previous context as the current context for the current thread
    fn exit(&self) -> PyResult<()>;
}

/// Implementation of functionality for [`PyContextVar`].
///
/// These methods are defined for the `Bound<'py, PyContextVar>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyContextVar")]
pub trait PyContextVarMethods<'py>: crate::sealed::Sealed {
    /// The name of the variable.
    fn name(&self) -> Bound<'py, PyString>;
    
    /// Return a value for the context variable for the current context.
    fn get(&self) -> PyResult<Option<Bound<'py, PyAny>>>;

    /// Return a value for the context variable for the current context.
    fn get_or_default(&self, default: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>>;

    /// Call to set a new value for the context variable in the current context.
    /// 
    /// Returns a Token object that can be used to restore the variable to its previous value via the ContextVar.reset() method.
    fn set<T>(&self, value: Bound<'py, T>) -> PyResult<Bound<'py, PyContextToken>>;

    /// Reset the context variable to the value it had before the ContextVar.set() that created the token was used.
    fn reset(&self, token: Bound<'py, PyContextToken>) -> PyResult<()>;
}

/// Implementation of functionality for [`PyContextToken`].
///
/// These methods are defined for the `Bound<'py, PyContextToken>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyContextToken")]
pub trait PyContextTokenMethods<'py>: crate::sealed::Sealed {
    /// The ContextVar object that created this token
    fn var(&self) -> PyResult<Bound<'py, PyContextVar>>;

    /// Set to the value the variable had before the ContextVar.set() method call that created the token.
    /// 
    /// It returns `None`` if the variable was not set before the call.
    fn old_value(&self) -> PyResult<Option<Bound<'py, PyAny>>>;
}

/// A mapping of ContextVars to their values.
/// 
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyContext>`][crate::Py] or [`Bound<'py, PyContext>`][Bound].
#[repr(transparent)]
pub struct PyContext(PyAny);
pyobject_native_type_core!(
    PyContext,
    pyobject_native_static_type_object!(ffi::PyContext_Type),
    #module=Some("contextvars"),
    #checkfunction=ffi::PyContext_CheckExact
);

impl PyContext {
    /// Create a new empty context object
    pub fn new(py: Python<'_>) -> PyResult<Bound<'_, PyContext>> {
        unsafe {
            ffi::PyContext_New()
                .assume_owned_or_err(py)
                .downcast_into_unchecked()
        }
    }

    /// Returns a copy of the current Context object.
    pub fn copy_current(py: Python<'_>) -> PyResult<Bound<'_, PyContext>> {
        unsafe {
            ffi::PyContext_CopyCurrent()
                .assume_owned_or_err(py)
                .downcast_into_unchecked()
        }
    }
}

impl<'py> PyContextMethods<'py> for Bound<'py, PyContext> {
    fn copy(&self) -> PyResult<Bound<'py, PyContext>> {
        unsafe {
            ffi::PyContext_Copy(self.as_ptr())
                .assume_owned_or_err(self.py())
                .downcast_into_unchecked()
        }
    }

    fn enter(&self) -> PyResult<()> {
        let r = unsafe { ffi::PyContext_Enter(self.as_ptr()) };
        if r == 0 {
            Ok(())
        } else {
            Err(PyErr::fetch(self.py()))
        }
    }

    fn exit(&self) -> PyResult<()> {
        let r = unsafe { ffi::PyContext_Exit(self.as_ptr()) };
        if r == 0 {
            Ok(())
        } else {
            Err(PyErr::fetch(self.py()))
        }
    }
}

/// Bindings around `contextvars.ContextVar`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyContextVar>`][crate::Py] or [`Bound<'py, PyContextVar>`][Bound].
#[repr(transparent)]
pub struct PyContextVar(PyAny);
pyobject_native_type_core!(
    PyContextVar,
    pyobject_native_static_type_object!(ffi::PyContextVar_Type),
    #module=Some("contextvars"),
    #checkfunction=ffi::PyContextVar_CheckExact
);

impl PyContextVar {
    /// Create new ContextVar with no default
    pub fn new<'py>(py: Python<'py>, name: &'static CStr) -> PyResult<Bound<'py, PyContextVar>> {
        unsafe {
            ffi::PyContextVar_New(name.as_ptr(), ptr::null_mut())
                .assume_owned_or_err(py)
                .downcast_into_unchecked()
        }
    }

    /// Create new ContextVar with default value
    pub fn with_default<'py, D: IntoPyObject<'py>>(py: Python<'py>, name: &CStr, default: D) -> PyResult<Bound<'py, PyContextVar>> {
        let def = default.into_pyobject(py).map_err(Into::into)?;
        unsafe {
            ffi::PyContextVar_New(name.as_ptr(), def.as_ptr())
                .assume_owned_or_err(py)
                .downcast_into_unchecked()
        }
    }
}

impl<'py> PyContextVarMethods<'py> for Bound<'py, PyContextVar> {
    fn name(&self) -> Bound<'py, PyString> {
        self.getattr("name")
            .unwrap()
            .downcast_into_exact::<PyString>()
            .unwrap()
    }

    fn get(&self) -> PyResult<Option<Bound<'py, PyAny>>> {
        let mut value = ptr::null_mut();
        let r = unsafe { ffi::PyContextVar_Get(self.as_ptr(), ptr::null_mut(), &mut value) };
        if r == 0 {
            Ok(unsafe { value.assume_owned_or_opt(self.py()) })
        } else {
            Err(PyErr::fetch(self.py()))
        }
    }

    fn get_or_default(&self, default: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let mut value = ptr::null_mut();
        let r = unsafe { ffi::PyContextVar_Get(self.as_ptr(), default.as_ptr(), &mut value) };
        if r == 0 {
            Ok(unsafe { value.assume_owned(self.py()) })
        } else {
            Err(PyErr::fetch(self.py()))
        }
    }

    fn set<T>(&self, value: Bound<'py, T>) -> PyResult<Bound<'py, PyContextToken>> {
        unsafe {
            ffi::PyContextVar_Set(self.as_ptr(), value.as_ptr())
                .assume_owned_or_err(self.py())
                .downcast_into_unchecked()
        }
    }

    fn reset(&self, token: Bound<'py, PyContextToken>) -> PyResult<()> {
        let r = unsafe { ffi::PyContextVar_Reset(self.as_ptr(), token.as_ptr()) };
        if r == 0 {
            Ok(())
        } else {
            Err(PyErr::fetch(self.py()))
        }
    }
}


/// Bindings around `contextvars.Token`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
#[repr(transparent)]
pub struct PyContextToken(PyAny);
pyobject_native_type_core!(
    PyContextToken,
    pyobject_native_static_type_object!(ffi::PyContextToken_Type),
    #module=Some("contextvars"),
    #checkfunction=ffi::PyContextToken_CheckExact
);

impl<'py> PyContextTokenMethods<'py> for Bound<'py, PyContextToken> {
    fn var(&self) -> PyResult<Bound<'py, PyContextVar>> {
        self.getattr("var")
            .downcast_into()
    }

    fn old_value(&self) -> PyResult<Option<Bound<'py, PyAny>>> {
        let old_value = self.getattr("old_value")?;

        // Check if token is missing
        static TOKEN_MISSING: GILOnceCell<Py<PyAny>> = GILOnceCell::new();
        let missing = TOKEN_MISSING.get_or_init( self.py(), || {
            PyContextToken::type_object(self.py())
                .getattr("MISSING")
                .expect("Unable to get contextvars.Token.MISSING")
                .unbind()
        });
        Ok(if old_value.is(missing) {
            None
        } else {
            Some(old_value)
        })
    }
}