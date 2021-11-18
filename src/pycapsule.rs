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
    /// Can optionally provide a destructor for when `PyCapsule` is destroyed
    /// it will be passed the capsule.
    pub fn new<'py, T: 'static>(
        py: Python<'py>,
        value: T,
        name: &CStr,
        destructor: Option<ffi::PyCapsule_Destructor>,
    ) -> PyResult<&'py Self> {
        let val = Box::new(value);

        let cap_ptr = unsafe {
            ffi::PyCapsule_New(Box::into_raw(val) as *mut c_void, name.as_ptr(), destructor)
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
    /// ## Safety
    /// This is unsafe, as there is no guarantee when casting `*mut void` into `T`.
    pub unsafe fn import<'py, T>(py: Python<'py>, name: &CStr, no_block: bool) -> PyResult<&'py T> {
        let ptr = ffi::PyCapsule_Import(name.as_ptr(), no_block as c_int);
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(&*(ptr as *const T))
        }
    }

    /// Set a context pointer in the capsule to `T`
    pub fn set_context<'py, T: 'static>(&self, py: Python<'py>, context: T) -> PyResult<()> {
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
    /// ## Safety
    ///
    /// This is unsafe, as there is no guarantee when casting `*mut void` into `T`.
    pub unsafe fn get_context<T>(&self, py: Python) -> PyResult<Option<&T>> {
        let ctx = ffi::PyCapsule_GetContext(self.as_ptr());
        if ctx.is_null() {
            if self.is_valid() & PyErr::occurred(py) {
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
    /// This is unsafe because there is no guarantee the pointer is `T`
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

    /// Get the capsule destructor, if any.
    pub fn get_destructor(&self, py: Python) -> PyResult<Option<ffi::PyCapsule_Destructor>> {
        match unsafe { ffi::PyCapsule_GetDestructor(self.as_ptr()) } {
            Some(destructor) => Ok(Some(destructor)),
            None => {
                // A None can mean an error was raised, or there is no destructor
                // https://docs.python.org/3/c-api/capsule.html#c.PyCapsule_GetDestructor
                if self.is_valid() {
                    Ok(None)
                } else {
                    Err(PyErr::fetch(py))
                }
            }
        }
    }

    /// Retrieve the name of this capsule.
    pub fn name(&self) -> &CStr {
        unsafe {
            let ptr = ffi::PyCapsule_GetName(self.as_ptr());
            CStr::from_ptr(ptr)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::PyModule;
    use crate::{ffi, pycapsule::PyCapsule, Py, PyResult, Python};
    use std::ffi::{c_void, CString};
    use std::sync::mpsc::{channel, Sender};

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

            let cap = PyCapsule::new(py, foo, &name, None)?;
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
            let cap = PyCapsule::new(py, foo as *const c_void, &name, None).unwrap();
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
            let cap = PyCapsule::new(py, (), &name, None)?;

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

            let capsule = PyCapsule::new(py, foo, &name, None)?;

            let module = PyModule::import(py, "builtins")?;
            module.add("capsule", capsule)?;

            let cap: &Foo = unsafe { PyCapsule::import(py, name.as_ref(), false)? };
            assert_eq!(cap.val, 123);
            Ok(())
        })
    }

    #[test]
    fn test_pycapsule_destructor() {
        #[repr(C)]
        struct Foo {
            called: Sender<bool>,
        }

        let (tx, rx) = channel();

        // Setup destructor, call sender to notify of being called
        unsafe extern "C" fn destructor(ptr: *mut ffi::PyObject) {
            Python::with_gil(|py| {
                let cap = py.from_borrowed_ptr::<PyCapsule>(ptr);
                let foo = cap.reference::<Foo>();
                foo.called.send(true).unwrap();
            })
        }

        // Create a capsule and allow it to be freed.
        let r = Python::with_gil(|py| -> PyResult<()> {
            let foo = Foo { called: tx };
            let name = CString::new("builtins.capsule").unwrap();
            let _capsule = PyCapsule::new(py, foo, &name, Some(destructor))?;
            Ok(())
        });
        assert!(r.is_ok());

        // Indeed it was
        assert_eq!(rx.recv(), Ok(true));
    }

    #[test]
    fn test_vec_storage() {
        let cap: Py<PyCapsule> = Python::with_gil(|py| {
            let name = CString::new("foo").unwrap();

            let stuff: Vec<u8> = vec![1, 2, 3, 4];
            let cap = PyCapsule::new(py, stuff, &name, None).unwrap();

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
            let cap = PyCapsule::new(py, (), &name, None).unwrap();

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
