use crate::Python;
use crate::{ffi, AsPyPointer, PyAny};
use crate::{pyobject_native_type_core, PyErr, PyResult};
use std::ffi::{c_void, CStr};
use std::os::raw::c_int;

/// TODO: docs
/// <https://docs.python.org/3/c-api/capsule.html#capsules>
#[repr(transparent)]
pub struct PyCapsule(PyAny);

pyobject_native_type_core!(PyCapsule, ffi::PyCapsule_Type, #checkfunction=ffi::PyCapsule_CheckExact);

impl PyCapsule {
    /// TODO: docs
    pub fn new<'py, T>(
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

    /// TODO: docs
    pub fn import<'py, T>(py: Python<'py>, name: &CStr, no_block: bool) -> PyResult<&'py T> {
        let ptr = unsafe { ffi::PyCapsule_Import(name.as_ptr(), no_block as c_int) };
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(unsafe { &*(ptr as *const T) })
        }
    }

    /// TODO: docs
    pub fn set_context<T>(&self, py: Python, context: T) -> PyResult<()> {
        let ctx = Box::new(context);
        let result =
            unsafe { ffi::PyCapsule_SetContext(self.as_ptr(), Box::into_raw(ctx) as _) as u8 };
        if result != 0 {
            Err(PyErr::fetch(py))
        } else {
            Ok(())
        }
    }

    /// TODO: docs
    pub fn get_context<T>(&self, py: Python) -> PyResult<Option<&T>> {
        let ctx = unsafe { ffi::PyCapsule_GetContext(self.as_ptr()) };
        if ctx.is_null() {
            if self.is_valid() & PyErr::occurred(py) {
                Err(PyErr::fetch(py))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(unsafe { &*(ctx as *const T) }))
        }
    }

    /// TODO: docs
    pub fn reference<T>(&self) -> &T {
        unsafe { &*(self.get_pointer() as *const T) }
    }

    /// TODO: docs
    pub fn get_pointer(&self) -> *mut c_void {
        unsafe { ffi::PyCapsule_GetPointer(self.0.as_ptr(), self.name().as_ptr()) }
    }

    /// TODO: docs
    pub fn is_valid(&self) -> bool {
        let r = unsafe { ffi::PyCapsule_IsValid(self.as_ptr(), self.name().as_ptr()) } as u8;
        r != 0
    }

    /// TODO: docs
    pub fn get_deconstructor(&self, py: Python) -> PyResult<Option<ffi::PyCapsule_Destructor>> {
        match unsafe { ffi::PyCapsule_GetDestructor(self.as_ptr()) } {
            Some(deconstructor) => Ok(Some(deconstructor)),
            None => {
                // A None can mean an error was raised, or there is no deconstructor
                // https://docs.python.org/3/c-api/capsule.html#c.PyCapsule_GetDestructor
                if self.is_valid() {
                    Ok(None)
                } else {
                    Err(PyErr::fetch(py))
                }
            }
        }
    }

    /// TODO: docs
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
    use crate::{ffi, pycapsule::PyCapsule, PyResult, Python};
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

            let foo_capi = cap.reference::<Foo>();
            assert_eq!(foo_capi.val, 123);
            assert_eq!(foo_capi.get_val(), 123);
            assert_eq!(cap.name(), name.as_ref());
            Ok(())
        })
    }

    #[test]
    fn test_pycapsule_func() -> PyResult<()> {
        extern "C" fn foo(x: u32) -> u32 {
            x
        }

        Python::with_gil(|py| {
            let name = CString::new("foo").unwrap();

            let cap = PyCapsule::new(py, foo as *const c_void, &name, None)?;
            let f = cap.reference::<fn(u32) -> u32>();
            assert_eq!(f(123), 123);
            Ok(())
        })
    }

    #[test]
    fn test_pycapsule_context() -> PyResult<()> {
        Python::with_gil(|py| {
            let name = CString::new("foo").unwrap();
            let cap = PyCapsule::new(py, (), &name, None)?;

            let c = cap.get_context::<()>(py)?;
            assert!(c.is_none());

            cap.set_context(py, 123)?;

            let ctx: Option<&u32> = cap.get_context(py)?;
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

            let path = CString::new("builtins.capsule").unwrap();
            let cap: &Foo = PyCapsule::import(py, path.as_ref(), false)?;
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
}
