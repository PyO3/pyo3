use crate::Python;
use crate::{ffi, PyAny};
use crate::{pyobject_native_type_core, PyErr, PyResult};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

/// Represents a Python Capsule
/// as described in [Capsules](https://docs.python.org/3/c-api/capsule.html#capsules):
/// > This subtype of PyObject represents an opaque value, useful for C extension
/// > modules who need to pass an opaque value (as a void* pointer) through Python
/// > code to other C code. It is often used to make a C function pointer defined
/// > in one module available to other modules, so the regular import mechanism can
/// > be used to access C APIs defined in dynamically loaded modules.
///
///
/// # Example
/// ```
/// use pyo3::{prelude::*, types::PyCapsule};
/// use std::ffi::CString;
///
/// #[repr(C)]
/// struct Foo {
///     pub val: u32,
/// }
///
/// let r = Python::with_gil(|py| -> PyResult<()> {
///     let foo = Foo { val: 123 };
///     let name = CString::new("builtins.capsule").unwrap();
///
///     let capsule = PyCapsule::new(py, foo, Some(name.clone()))?;
///
///     let module = PyModule::import(py, "builtins")?;
///     module.add("capsule", capsule)?;
///
///     let cap: &Foo = unsafe { PyCapsule::import(py, name.as_ref())? };
///     assert_eq!(cap.val, 123);
///     Ok(())
/// });
/// assert!(r.is_ok());
/// ```
#[repr(transparent)]
pub struct PyCapsule(PyAny);

pyobject_native_type_core!(PyCapsule, pyobject_native_static_type_object!(ffi::PyCapsule_Type), #checkfunction=ffi::PyCapsule_CheckExact);

impl PyCapsule {
    /// Constructs a new capsule whose contents are `value`, associated with `name`.
    /// `name` is the identifier for the capsule; if it is stored as an attribute of a module,
    /// the name should be in the format `"modulename.attribute"`.
    ///
    /// It is checked at compile time that the type T is not zero-sized. Rust function items
    /// need to be cast to a function pointer (`fn(args) -> result`) to be put into a capsule.
    ///
    /// # Example
    ///
    /// ```
    /// use pyo3::{prelude::*, types::PyCapsule};
    /// use std::ffi::CString;
    ///
    /// Python::with_gil(|py| {
    ///     let name = CString::new("foo").unwrap();
    ///     let capsule = PyCapsule::new(py, 123_u32, Some(name)).unwrap();
    ///     let val = unsafe { capsule.reference::<u32>() };
    ///     assert_eq!(*val, 123);
    /// });
    /// ```
    ///
    /// However, attempting to construct a `PyCapsule` with a zero-sized type will not compile:
    ///
    /// ```compile_fail
    /// use pyo3::{prelude::*, types::PyCapsule};
    /// use std::ffi::CString;
    ///
    /// Python::with_gil(|py| {
    ///     let capsule = PyCapsule::new(py, (), None).unwrap();  // Oops! `()` is zero sized!
    /// });
    /// ```
    pub fn new<T: 'static + Send + AssertNotZeroSized>(
        py: Python<'_>,
        value: T,
        name: Option<CString>,
    ) -> PyResult<&Self> {
        Self::new_with_destructor(py, value, name, |_, _| {})
    }

    /// Constructs a new capsule whose contents are `value`, associated with `name`.
    ///
    /// Also provides a destructor: when the `PyCapsule` is destroyed, it will be passed the original object,
    /// as well as a `*mut c_void` which will point to the capsule's context, if any.
    ///
    /// The `destructor` must be `Send`, because there is no guarantee which thread it will eventually
    /// be called from.
    pub fn new_with_destructor<
        T: 'static + Send + AssertNotZeroSized,
        F: FnOnce(T, *mut c_void) + Send,
    >(
        py: Python<'_>,
        value: T,
        name: Option<CString>,
        destructor: F,
    ) -> PyResult<&'_ Self> {
        AssertNotZeroSized::assert_not_zero_sized(&value);

        // Sanity check for capsule layout
        debug_assert_eq!(memoffset::offset_of!(CapsuleContents::<T, F>, value), 0);

        let name_ptr = name.as_ref().map_or(std::ptr::null(), |name| name.as_ptr());
        let val = Box::new(CapsuleContents {
            value,
            destructor,
            name,
        });

        unsafe {
            let cap_ptr = ffi::PyCapsule_New(
                Box::into_raw(val) as *mut c_void,
                name_ptr,
                Some(capsule_destructor::<T, F>),
            );
            py.from_owned_ptr_or_err(cap_ptr)
        }
    }

    /// Imports an existing capsule.
    ///
    /// The `name` should match the path to the module attribute exactly in the form
    /// of `"module.attribute"`, which should be the same as the name within the capsule.
    ///
    /// # Safety
    ///
    /// It must be known that the capsule imported by `name` contains an item of type `T`.
    pub unsafe fn import<'py, T>(py: Python<'py>, name: &CStr) -> PyResult<&'py T> {
        let ptr = ffi::PyCapsule_Import(name.as_ptr(), false as c_int);
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(&*(ptr as *const T))
        }
    }

    /// Sets the context pointer in the capsule.
    ///
    /// Returns an error if this capsule is not valid.
    ///
    /// # Notes
    ///
    /// The context is treated much like the value of the capsule, but should likely act as
    /// a place to store any state management when using the capsule.
    ///
    /// If you want to store a Rust value as the context, and drop it from the destructor, use
    /// `Box::into_raw` to convert it into a pointer, see the example.
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::mpsc::{channel, Sender};
    /// use libc::c_void;
    /// use pyo3::{prelude::*, types::PyCapsule};
    ///
    /// let (tx, rx) = channel::<String>();
    ///
    /// fn destructor(val: u32, context: *mut c_void) {
    ///     let ctx = unsafe { *Box::from_raw(context as *mut Sender<String>) };
    ///     ctx.send("Destructor called!".to_string()).unwrap();
    /// }
    ///
    /// Python::with_gil(|py| {
    ///     let capsule =
    ///         PyCapsule::new_with_destructor(py, 123, None, destructor as fn(u32, *mut c_void))
    ///             .unwrap();
    ///     let context = Box::new(tx);  // `Sender<String>` is our context, box it up and ship it!
    ///     capsule.set_context(Box::into_raw(context) as *mut c_void).unwrap();
    ///     // This scope will end, causing our destructor to be called...
    /// });
    ///
    /// assert_eq!(rx.recv(), Ok("Destructor called!".to_string()));
    /// ```
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn set_context(&self, context: *mut c_void) -> PyResult<()> {
        let result = unsafe { ffi::PyCapsule_SetContext(self.as_ptr(), context) };
        if result != 0 {
            Err(PyErr::fetch(self.py()))
        } else {
            Ok(())
        }
    }

    /// Gets the current context stored in the capsule. If there is no context, the pointer
    /// will be null.
    ///
    /// Returns an error if this capsule is not valid.
    pub fn context(&self) -> PyResult<*mut c_void> {
        let ctx = unsafe { ffi::PyCapsule_GetContext(self.as_ptr()) };
        if ctx.is_null() {
            ensure_no_error(self.py())?
        }
        Ok(ctx)
    }

    /// Obtains a reference to the value of this capsule.
    ///
    /// # Safety
    ///
    /// It must be known that this capsule is valid and its pointer is to an item of type `T`.
    pub unsafe fn reference<T>(&self) -> &T {
        &*(self.pointer() as *const T)
    }

    /// Gets the raw `c_void` pointer to the value in this capsule.
    ///
    /// Returns null if this capsule is not valid.
    pub fn pointer(&self) -> *mut c_void {
        unsafe {
            let ptr = ffi::PyCapsule_GetPointer(self.0.as_ptr(), self.name_ptr_ignore_error());
            if ptr.is_null() {
                ffi::PyErr_Clear();
            }
            ptr
        }
    }

    /// Checks if this is a valid capsule.
    ///
    /// Returns true if the stored `pointer()` is non-null.
    pub fn is_valid(&self) -> bool {
        // As well as if the stored pointer is null, PyCapsule_IsValid also returns false if
        // self.as_ptr() is null or not a ptr to a PyCapsule object. Both of these are guaranteed
        // to not be the case thanks to invariants of this PyCapsule struct.
        let r = unsafe { ffi::PyCapsule_IsValid(self.as_ptr(), self.name_ptr_ignore_error()) };
        r != 0
    }

    /// Retrieves the name of this capsule, if set.
    ///
    /// Returns an error if this capsule is not valid.
    pub fn name(&self) -> PyResult<Option<&CStr>> {
        unsafe {
            let ptr = ffi::PyCapsule_GetName(self.as_ptr());
            if ptr.is_null() {
                ensure_no_error(self.py())?;
                Ok(None)
            } else {
                Ok(Some(CStr::from_ptr(ptr)))
            }
        }
    }

    /// Attempts to retrieve the raw name pointer of this capsule.
    ///
    /// On error, clears the error indicator and returns NULL. This is a private function and next
    /// use of this capsule will error anyway.
    fn name_ptr_ignore_error(&self) -> *const c_char {
        let ptr = unsafe { ffi::PyCapsule_GetName(self.as_ptr()) };
        if ptr.is_null() {
            unsafe { ffi::PyErr_Clear() };
        }
        ptr
    }
}

// C layout, as PyCapsule::get_reference depends on `T` being first.
#[repr(C)]
struct CapsuleContents<T: 'static + Send, D: FnOnce(T, *mut c_void) + Send> {
    /// Value of the capsule
    value: T,
    /// Destructor to be used by the capsule
    destructor: D,
    /// Name used when creating the capsule
    name: Option<CString>,
}

// Wrapping ffi::PyCapsule_Destructor for a user supplied FnOnce(T) for capsule destructor
unsafe extern "C" fn capsule_destructor<T: 'static + Send, F: FnOnce(T, *mut c_void) + Send>(
    capsule: *mut ffi::PyObject,
) {
    let ptr = ffi::PyCapsule_GetPointer(capsule, ffi::PyCapsule_GetName(capsule));
    let ctx = ffi::PyCapsule_GetContext(capsule);
    let CapsuleContents {
        value, destructor, ..
    } = *Box::from_raw(ptr as *mut CapsuleContents<T, F>);
    destructor(value, ctx)
}

/// Guarantee `T` is not zero sized at compile time.
// credit: `<https://users.rust-lang.org/t/is-it-possible-to-assert-at-compile-time-that-foo-t-is-not-called-with-a-zst/67685>`
#[doc(hidden)]
pub trait AssertNotZeroSized: Sized {
    const _CONDITION: usize = (std::mem::size_of::<Self>() == 0) as usize;
    const _CHECK: &'static str =
        ["PyCapsule value type T must not be zero-sized!"][Self::_CONDITION];
    #[allow(path_statements, clippy::no_effect)]
    fn assert_not_zero_sized(&self) {
        <Self as AssertNotZeroSized>::_CHECK;
    }
}

impl<T> AssertNotZeroSized for T {}

fn ensure_no_error(py: Python<'_>) -> PyResult<()> {
    if let Some(err) = PyErr::take(py) {
        Err(err)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use libc::c_void;

    use crate::prelude::PyModule;
    use crate::{types::PyCapsule, Py, PyResult, Python};
    use std::ffi::CString;
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

            let cap = PyCapsule::new(py, foo, Some(name.clone()))?;
            assert!(cap.is_valid());

            let foo_capi = unsafe { cap.reference::<Foo>() };
            assert_eq!(foo_capi.val, 123);
            assert_eq!(foo_capi.get_val(), 123);
            assert_eq!(cap.name().unwrap(), Some(name.as_ref()));
            Ok(())
        })
    }

    #[test]
    fn test_pycapsule_func() {
        fn foo(x: u32) -> u32 {
            x
        }

        let cap: Py<PyCapsule> = Python::with_gil(|py| {
            let name = CString::new("foo").unwrap();
            let cap = PyCapsule::new(py, foo as fn(u32) -> u32, Some(name)).unwrap();
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
            let cap = PyCapsule::new(py, 0, Some(name))?;

            let c = cap.context()?;
            assert!(c.is_null());

            let ctx = Box::new(123_u32);
            cap.set_context(Box::into_raw(ctx) as _)?;

            let ctx_ptr: *mut c_void = cap.context()?;
            let ctx = unsafe { *Box::from_raw(ctx_ptr as *mut u32) };
            assert_eq!(ctx, 123);
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

            let capsule = PyCapsule::new(py, foo, Some(name.clone()))?;

            let module = PyModule::import(py, "builtins")?;
            module.add("capsule", capsule)?;

            // check error when wrong named passed for capsule.
            let wrong_name = CString::new("builtins.non_existant").unwrap();
            let result: PyResult<&Foo> = unsafe { PyCapsule::import(py, wrong_name.as_ref()) };
            assert!(result.is_err());

            // corret name is okay.
            let cap: &Foo = unsafe { PyCapsule::import(py, name.as_ref())? };
            assert_eq!(cap.val, 123);
            Ok(())
        })
    }

    #[test]
    fn test_vec_storage() {
        let cap: Py<PyCapsule> = Python::with_gil(|py| {
            let name = CString::new("foo").unwrap();

            let stuff: Vec<u8> = vec![1, 2, 3, 4];
            let cap = PyCapsule::new(py, stuff, Some(name)).unwrap();

            cap.into()
        });

        Python::with_gil(|py| {
            let ctx: &Vec<u8> = unsafe { cap.as_ref(py).reference() };
            assert_eq!(ctx, &[1, 2, 3, 4]);
        })
    }

    #[test]
    fn test_vec_context() {
        let context: Vec<u8> = vec![1, 2, 3, 4];

        let cap: Py<PyCapsule> = Python::with_gil(|py| {
            let name = CString::new("foo").unwrap();
            let cap = PyCapsule::new(py, 0, Some(name)).unwrap();
            cap.set_context(Box::into_raw(Box::new(&context)) as _)
                .unwrap();

            cap.into()
        });

        Python::with_gil(|py| {
            let ctx_ptr: *mut c_void = cap.as_ref(py).context().unwrap();
            let ctx = unsafe { *Box::from_raw(ctx_ptr as *mut &Vec<u8>) };
            assert_eq!(ctx, &vec![1_u8, 2, 3, 4]);
        })
    }

    #[test]
    fn test_pycapsule_destructor() {
        let (tx, rx) = channel::<bool>();

        fn destructor(_val: u32, ctx: *mut c_void) {
            assert!(!ctx.is_null());
            let context = unsafe { *Box::from_raw(ctx as *mut Sender<bool>) };
            context.send(true).unwrap();
        }

        Python::with_gil(|py| {
            let name = CString::new("foo").unwrap();
            let cap = PyCapsule::new_with_destructor(py, 0, Some(name), destructor).unwrap();
            cap.set_context(Box::into_raw(Box::new(tx)) as _).unwrap();
        });

        // the destructor was called.
        assert_eq!(rx.recv(), Ok(true));
    }

    #[test]
    fn test_pycapsule_no_name() {
        Python::with_gil(|py| {
            let cap = PyCapsule::new(py, 0usize, None).unwrap();

            assert_eq!(unsafe { cap.reference::<usize>() }, &0usize);
            assert_eq!(cap.name().unwrap(), None);
            assert_eq!(cap.context().unwrap(), std::ptr::null_mut());
        });
    }
}
