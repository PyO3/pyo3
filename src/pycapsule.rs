// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::Python;
use crate::{ffi, AsPyPointer, PyAny};
use crate::{pyobject_native_type_core, PyErr, PyResult};
use std::ffi::{c_void, CStr};
use std::os::raw::c_int;

/// Represents a Python Capsule
/// As described in [Capsules](https://docs.python.org/3/c-api/capsule.html#capsules)
/// > This subtype of PyObject represents an opaque value, useful for C extension
/// > modules who need to pass an opaque value (as a void* pointer) through Python
/// > code to other C code. It is often used to make a C function pointer defined
/// > in one module available to other modules, so the regular import mechanism can
/// > be used to access C APIs defined in dynamically loaded modules.
///
///
/// # Example
/// ```
///  use std::ffi::CString;
///  use pyo3::{prelude::*, pycapsule::PyCapsule};
///
///  #[repr(C)]
///  struct Foo {
///     pub val: u32,
///  }
///
///  let r = Python::with_gil(|py| -> PyResult<()> {
///     let foo = Foo { val: 123 };
///     let name = CString::new("builtins.capsule").unwrap();
///
///     let capsule = PyCapsule::new(py, foo, name.as_ref(), None)?;
///
///     let module = PyModule::import(py, "builtins")?;
///     module.add("capsule", capsule)?;
///
///     let cap: &Foo = PyCapsule::import(py, name.as_ref(), false)?;
///     assert_eq!(cap.val, 123);
///     Ok(())
///  });
///  assert!(r.is_ok());
/// ```
#[repr(transparent)]
pub struct PyCapsule(PyAny);

pyobject_native_type_core!(PyCapsule, ffi::PyCapsule_Type, #checkfunction=ffi::PyCapsule_CheckExact);

impl PyCapsule {
    /// Constructs a new capsule of whose contents are `T` associated with `name`.
    pub fn new<'py, T: 'static>(py: Python<'py>, value: T, name: &CStr) -> PyResult<&'py Self> {
        Self::new_with_destructor(py, value, name, std::mem::drop)
    }

    /// Constructs a new capsule of whose contents are `T` associated with `name`
    /// Provide a destructor for when `PyCapsule` is destroyed it will be passed the capsule.
    pub fn new_with_destructor<'py, T: 'static, F: FnOnce(T)>(
        py: Python<'py>,
        value: T,
        name: &CStr,
        destructor: F,
    ) -> PyResult<&'py Self> {
        let val = Box::new(CapsuleContents { value, destructor });

        let cap_ptr = unsafe {
            ffi::PyCapsule_New(
                Box::into_raw(val) as *mut c_void,
                name.as_ptr(),
                Some(capsule_destructor::<T, F>),
            )
        };
        if cap_ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(unsafe { py.from_owned_ptr::<PyCapsule>(cap_ptr) })
        }
    }

    /// Import an existing capsule.
    ///
    /// The `name` should match the path to module attribute exactly in the form
    /// of `module.attribute`, which should be the same as the name within the
    /// capsule. `no_block` indicates to use
    /// [PyImport_ImportModuleNoBlock()](https://docs.python.org/3/c-api/import.html#c.PyImport_ImportModuleNoBlock)
    /// or [PyImport_ImportModule()](https://docs.python.org/3/c-api/import.html#c.PyImport_ImportModule)
    /// when accessing the capsule.
    ///
    /// # Safety
    ///
    /// It must be known that this capsule's pointer is to an item of type `T`.
    pub unsafe fn import<'py, T>(py: Python<'py>, name: &CStr, no_block: bool) -> PyResult<&'py T> {
        let ptr = ffi::PyCapsule_Import(name.as_ptr(), no_block as c_int);
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(&*(ptr as *const T))
        }
    }

    /// Set a context pointer in the capsule to `T`
    pub fn set_context<T: 'static>(&self, py: Python, context: T) -> PyResult<()> {
        let ctx = Box::new(context);
        let result =
            unsafe { ffi::PyCapsule_SetContext(self.as_ptr(), Box::into_raw(ctx) as _) as u8 };
        if result != 0 {
            Err(PyErr::fetch(py))
        } else {
            Ok(())
        }
    }

    /// Get a reference to the context `T` in the capsule, if any.
    ///
    /// # Safety
    ///
    /// It must be known that this capsule's contains a context item of type `T`.
    pub unsafe fn get_context<T>(&self, py: Python) -> PyResult<Option<&T>> {
        let ctx = ffi::PyCapsule_GetContext(self.as_ptr());
        if ctx.is_null() {
            if self.is_valid() && PyErr::occurred(py) {
                Err(PyErr::fetch(py))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(&*(ctx as *const T)))
        }
    }

    /// Obtain a reference to the value `T` of this capsule.
    ///
    /// # Safety
    ///
    /// It must be known that this capsule's pointer is to an item of type `T`.
    pub unsafe fn reference<T>(&self) -> &T {
        &*(self.get_pointer() as *const T)
    }

    /// Get the raw `c_void` pointer to the value in this capsule.
    pub fn get_pointer(&self) -> *mut c_void {
        unsafe { ffi::PyCapsule_GetPointer(self.0.as_ptr(), self.name().as_ptr()) }
    }

    /// Check if this is a valid capsule.
    pub fn is_valid(&self) -> bool {
        let r = unsafe { ffi::PyCapsule_IsValid(self.as_ptr(), self.name().as_ptr()) } as u8;
        r != 0
    }

    /// Retrieve the name of this capsule.
    pub fn name(&self) -> &CStr {
        unsafe {
            let ptr = ffi::PyCapsule_GetName(self.as_ptr());
            CStr::from_ptr(ptr)
        }
    }
}

// C layout, as PyCapsule::get_reference depends on `T` being first.
#[repr(C)]
struct CapsuleContents<T: 'static, D: FnOnce(T)> {
    value: T,
    destructor: D,
}

// Wrapping ffi::PyCapsule_Destructor for a user supplied FnOnce(T) for capsule destructor
unsafe extern "C" fn capsule_destructor<T: 'static, F: FnOnce(T)>(capsule: *mut ffi::PyObject) {
    let ptr = ffi::PyCapsule_GetPointer(capsule, ffi::PyCapsule_GetName(capsule));
    let CapsuleContents { value, destructor } = *Box::from_raw(ptr as *mut CapsuleContents<T, F>);
    destructor(value)
}

#[cfg(test)]
mod tests {
    use crate::prelude::PyModule;
    use crate::{pycapsule::PyCapsule, Py, PyResult, Python};
    use std::ffi::{c_void, CString};

    #[test]
    fn test_pycapsule_struct() -> PyResult<()> {
        #[repr(C)]
        struct Foo {
            pub val: u32,
        }

        impl Foo {
            fn get_val(&self) -> u32 {
                self.val
            }
        }

        Python::with_gil(|py| -> PyResult<()> {
            let foo = Foo { val: 123 };
            let name = CString::new("foo").unwrap();

            let cap = PyCapsule::new(py, foo, &name)?;
            assert!(cap.is_valid());

            let foo_capi = unsafe { cap.reference::<Foo>() };
            assert_eq!(foo_capi.val, 123);
            assert_eq!(foo_capi.get_val(), 123);
            assert_eq!(cap.name(), name.as_ref());
            Ok(())
        })
    }

    #[test]
    fn test_pycapsule_func() {
        let cap: Py<PyCapsule> = Python::with_gil(|py| {
            extern "C" fn foo(x: u32) -> u32 {
                x
            }

            let name = CString::new("foo").unwrap();
            let cap = PyCapsule::new(py, foo as *const c_void, &name).unwrap();
            cap.into()
        });

        Python::with_gil(|py| {
            let f = unsafe { cap.as_ref(py).reference::<fn(u32) -> u32>() };
            assert_eq!(f(123), 123);
        });
    }

    #[test]
    fn test_pycapsule_context() -> PyResult<()> {
        Python::with_gil(|py| {
            let name = CString::new("foo").unwrap();
            let cap = PyCapsule::new(py, (), &name)?;

            let c = unsafe { cap.get_context::<()>(py)? };
            assert!(c.is_none());

            cap.set_context(py, 123)?;

            let ctx: Option<&u32> = unsafe { cap.get_context(py)? };
            assert_eq!(ctx, Some(&123));
            Ok(())
        })
    }

    #[test]
    fn test_pycapsule_import() -> PyResult<()> {
        #[repr(C)]
        struct Foo {
            pub val: u32,
        }

        Python::with_gil(|py| -> PyResult<()> {
            let foo = Foo { val: 123 };
            let name = CString::new("builtins.capsule").unwrap();

            let capsule = PyCapsule::new(py, foo, &name)?;

            let module = PyModule::import(py, "builtins")?;
            module.add("capsule", capsule)?;

            let cap: &Foo = unsafe { PyCapsule::import(py, name.as_ref(), false)? };
            assert_eq!(cap.val, 123);
            Ok(())
        })
    }

    #[test]
    fn test_vec_storage() {
        let cap: Py<PyCapsule> = Python::with_gil(|py| {
            let name = CString::new("foo").unwrap();

            let stuff: Vec<u8> = vec![1, 2, 3, 4];
            let cap = PyCapsule::new(py, stuff, &name).unwrap();

            cap.into()
        });

        Python::with_gil(|py| {
            let ctx: &Vec<u8> = unsafe { cap.as_ref(py).reference() };
            assert_eq!(ctx, &[1, 2, 3, 4]);
        })
    }

    #[test]
    fn test_vec_context() {
        let cap: Py<PyCapsule> = Python::with_gil(|py| {
            let name = CString::new("foo").unwrap();
            let cap = PyCapsule::new(py, (), &name).unwrap();

            let ctx: Vec<u8> = vec![1, 2, 3, 4];
            cap.set_context(py, ctx).unwrap();

            cap.into()
        });

        Python::with_gil(|py| {
            let ctx: Option<&Vec<u8>> = unsafe { cap.as_ref(py).get_context(py).unwrap() };
            assert_eq!(ctx, Some(&vec![1_u8, 2, 3, 4]));
        })
    }
}
