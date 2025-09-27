use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::{ffi, PyAny};
use crate::{Bound, Python};
use crate::{PyErr, PyResult};
use std::ffi::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};
use std::ptr::{self, NonNull};

/// Represents a Python Capsule
/// as described in [Capsules](https://docs.python.org/3/c-api/capsule.html#capsules):
/// > This subtype of PyObject represents an opaque value, useful for C extension
/// > modules who need to pass an opaque value (as a void* pointer) through Python
/// > code to other C code. It is often used to make a C function pointer defined
/// > in one module available to other modules, so the regular import mechanism can
/// > be used to access C APIs defined in dynamically loaded modules.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyCapsule>`][crate::Py] or [`Bound<'py, PyCapsule>`][Bound].
///
/// For APIs available on capsule objects, see the [`PyCapsuleMethods`] trait which is implemented for
/// [`Bound<'py, PyCapsule>`][Bound].
///
/// # Example
/// ```
/// use pyo3::{prelude::*, types::PyCapsule, ffi::c_str};
///
/// #[repr(C)]
/// struct Foo {
///     pub val: u32,
/// }
///
/// let r = Python::attach(|py| -> PyResult<()> {
///     let foo = Foo { val: 123 };
///     let capsule = PyCapsule::new(py, foo, Some(c_str!("builtins.capsule").to_owned()))?;
///
///     let module = PyModule::import(py, "builtins")?;
///     module.add("capsule", capsule)?;
///
///     let cap: &Foo = unsafe { PyCapsule::import(py, c_str!("builtins.capsule"))? };
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
    /// use pyo3::{prelude::*, types::PyCapsule, ffi::c_str};
    /// use std::ffi::CStr;
    ///
    /// // this can be c"foo" on Rust 1.77+
    /// const NAME: &CStr = c_str!("foo");
    ///
    /// Python::attach(|py| {
    ///     let capsule = PyCapsule::new(py, 123_u32, Some(NAME.to_owned())).unwrap();
    ///     let val = unsafe { capsule.reference_checked::<u32>(Some(NAME)) }.unwrap();
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
    /// Python::attach(|py| {
    ///     let capsule = PyCapsule::new(py, (), None).unwrap();  // Oops! `()` is zero sized!
    /// });
    /// ```
    pub fn new<T: 'static + Send + AssertNotZeroSized>(
        py: Python<'_>,
        value: T,
        name: Option<CString>,
    ) -> PyResult<Bound<'_, Self>> {
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
    ) -> PyResult<Bound<'_, Self>> {
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
            ffi::PyCapsule_New(
                Box::into_raw(val).cast(),
                name_ptr,
                Some(capsule_destructor::<T, F>),
            )
            .assume_owned_or_err(py)
            .cast_into_unchecked()
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
        let ptr = unsafe { ffi::PyCapsule_Import(name.as_ptr(), false as c_int) };
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(unsafe { &*ptr.cast::<T>() })
        }
    }
}

/// Implementation of functionality for [`PyCapsule`].
///
/// These methods are defined for the `Bound<'py, PyCapsule>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
///
/// # Name checking
///
/// Capsule methods contain arbitrary data which is cast to a specific type at runtime. This is
/// inherently quite dangerous, so Python allows capsules to be "named" to provide a hint as to
/// what data is contained in the capsule. Although not a perfect solution, this is better than
/// nothing.
///
/// The methods in this trait take the `name` as an `Option<&CStr>`, which is compared to the name
/// stored in the capsule (with `None` being used to indicate the capsule has no name).
#[doc(alias = "PyCapsule")]
pub trait PyCapsuleMethods<'py>: crate::sealed::Sealed {
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
    /// use std::ffi::c_void;
    /// use std::sync::mpsc::{channel, Sender};
    /// use pyo3::{prelude::*, types::PyCapsule};
    ///
    /// let (tx, rx) = channel::<String>();
    ///
    /// fn destructor(val: u32, context: *mut c_void) {
    ///     let ctx = unsafe { *Box::from_raw(context.cast::<Sender<String>>()) };
    ///     ctx.send("Destructor called!".to_string()).unwrap();
    /// }
    ///
    /// Python::attach(|py| {
    ///     let capsule =
    ///         PyCapsule::new_with_destructor(py, 123, None, destructor as fn(u32, *mut c_void))
    ///             .unwrap();
    ///     let context = Box::new(tx);  // `Sender<String>` is our context, box it up and ship it!
    ///     capsule.set_context(Box::into_raw(context).cast()).unwrap();
    ///     // This scope will end, causing our destructor to be called...
    /// });
    ///
    /// assert_eq!(rx.recv(), Ok("Destructor called!".to_string()));
    /// ```
    fn set_context(&self, context: *mut c_void) -> PyResult<()>;

    /// Gets the current context stored in the capsule. If there is no context, the pointer
    /// will be null.
    ///
    /// Returns an error if this capsule is not valid.
    fn context(&self) -> PyResult<*mut c_void>;

    /// Obtains a reference dereferenced from the pointer of this capsule, without checking its name.
    ///
    /// This does not check the name of the capsule, which is the only mechanism that Python
    /// provides to make sure that the pointer has the expected type. Prefer to use
    /// [`reference_checked()`][Self::reference_checked()] instead.
    ///
    /// # Safety
    ///
    /// See [`reference_checked()`][PyCapsuleMethods::reference_checked].
    #[deprecated(since = "0.27.0", note = "use `reference_checked()` instead")]
    unsafe fn reference<T>(&self) -> &T;

    /// Obtains a reference dereferenced from the pointer of this capsule. This is inherently very
    /// dangerous: it involves casting an untyped pointer to a specific type. Additionally, arbitrary
    /// Python code can change the contents of the capsule, which may invalidate the reference.
    ///
    /// Returns an error if the `name` does not exactly match the name stored in the capsule, or if
    /// the pointer stored in the capsule is null.
    ///
    /// # Safety
    ///
    /// - It must be known that the capsule pointer points to an item of type `T`.
    /// - The reference should not be used after arbitrary Python code has run.
    ///
    /// It is recommended to use this reference only for short-lived operations without executing any
    /// Python code. For long-lived operations, consider calling `.reference_checked()` each time the
    /// data is needed.
    unsafe fn reference_checked<T>(&self, name: Option<&CStr>) -> PyResult<&T>;

    /// Gets the raw pointer stored in this capsule, without checking its name.
    #[deprecated(since = "0.27.0", note = "use `pointer_checked()` instead")]
    fn pointer(&self) -> *mut c_void;

    /// Gets the raw pointer stored in this capsule.
    ///
    /// Returns an error if the capsule is not [valid][`PyCapsuleMethods::is_valid`] with the given `name`.
    fn pointer_checked(&self, name: Option<&CStr>) -> PyResult<NonNull<c_void>>;

    /// Checks if the capsule pointer is not null.
    ///
    /// This does not perform any check on the name of the capsule, which is the only mechanism
    /// that Python provides to make sure that the pointer has the expected type. Prefer to use
    /// [`is_valid_checked()`][Self::is_valid_checked()] instead.
    #[deprecated(since = "0.27.0", note = "use `is_valid_checked()` instead")]
    fn is_valid(&self) -> bool;

    /// Checks that the capsule name matches `name` and that the pointer is not null.
    fn is_valid_checked(&self, name: Option<&CStr>) -> bool;

    /// Retrieves the name of this capsule. If there is no name, the pointer will be null.
    ///
    /// Returns an error if this capsule is not valid.
    ///
    /// This method returns `*const c_char` instead of `&CStr` because it's possible for
    /// arbitrary Python code to change the capsule name. Callers can use `NonNull::from_ptr()`
    /// to get a `&CStr` if they want to, however they should beware the fact that the pointer
    /// may become invalid after arbitrary Python code has run.
    fn name(&self) -> PyResult<*const c_char>;
}

impl<'py> PyCapsuleMethods<'py> for Bound<'py, PyCapsule> {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn set_context(&self, context: *mut c_void) -> PyResult<()> {
        let result = unsafe { ffi::PyCapsule_SetContext(self.as_ptr(), context) };
        if result != 0 {
            Err(PyErr::fetch(self.py()))
        } else {
            Ok(())
        }
    }

    fn context(&self) -> PyResult<*mut c_void> {
        let ctx = unsafe { ffi::PyCapsule_GetContext(self.as_ptr()) };
        if ctx.is_null() {
            ensure_no_error(self.py())?
        }
        Ok(ctx)
    }

    #[allow(deprecated)]
    unsafe fn reference<T>(&self) -> &T {
        unsafe { &*self.pointer().cast() }
    }

    unsafe fn reference_checked<T>(&self, name: Option<&CStr>) -> PyResult<&T> {
        self.pointer_checked(name)
            .map(|ptr| unsafe { &*ptr.as_ptr().cast::<T>() })
    }

    fn pointer(&self) -> *mut c_void {
        unsafe {
            let ptr = ffi::PyCapsule_GetPointer(self.as_ptr(), name_ptr_ignore_error(self));
            if ptr.is_null() {
                ffi::PyErr_Clear();
            }
            ptr
        }
    }

    fn pointer_checked(&self, name: Option<&CStr>) -> PyResult<NonNull<c_void>> {
        let ptr = unsafe { ffi::PyCapsule_GetPointer(self.as_ptr(), name_ptr(name)) };
        match NonNull::new(ptr) {
            Some(ptr) => Ok(ptr),
            None => Err(PyErr::fetch(self.py())),
        }
    }

    fn is_valid(&self) -> bool {
        // As well as if the stored pointer is null, PyCapsule_IsValid also returns false if
        // self.as_ptr() is null or not a ptr to a PyCapsule object. Both of these are guaranteed
        // to not be the case thanks to invariants of this PyCapsule struct.
        let r = unsafe { ffi::PyCapsule_IsValid(self.as_ptr(), name_ptr_ignore_error(self)) };
        r != 0
    }

    fn is_valid_checked(&self, name: Option<&CStr>) -> bool {
        let r = unsafe { ffi::PyCapsule_IsValid(self.as_ptr(), name_ptr(name)) };
        r != 0
    }

    fn name(&self) -> PyResult<*const c_char> {
        let name = unsafe { ffi::PyCapsule_GetName(self.as_ptr()) };
        if name.is_null() {
            ensure_no_error(self.py())?;
        }
        Ok(name)
    }
}

// C layout, as PyCapsule::reference_checked() depends on `T` being first.
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
    unsafe {
        let ptr = ffi::PyCapsule_GetPointer(capsule, ffi::PyCapsule_GetName(capsule));
        let ctx = ffi::PyCapsule_GetContext(capsule);
        let CapsuleContents {
            value, destructor, ..
        } = *Box::from_raw(ptr.cast::<CapsuleContents<T, F>>());
        destructor(value, ctx)
    }
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

fn name_ptr_ignore_error(slf: &Bound<'_, PyCapsule>) -> *const c_char {
    let ptr = unsafe { ffi::PyCapsule_GetName(slf.as_ptr()) };
    if ptr.is_null() {
        unsafe { ffi::PyErr_Clear() };
    }
    ptr
}

fn name_ptr(name: Option<&CStr>) -> *const c_char {
    match name {
        Some(name) => name.as_ptr(),
        None => ptr::null(),
    }
}

#[cfg(test)]
mod tests {
    use crate::ffi;
    use crate::prelude::PyModule;
    use crate::types::capsule::PyCapsuleMethods;
    use crate::types::module::PyModuleMethods;
    use crate::{types::PyCapsule, Py, PyResult, Python};
    use std::ffi::{c_void, CStr};
    use std::sync::mpsc::{channel, Sender};

    const NAME: &CStr = ffi::c_str!("foo");

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

        Python::attach(|py| -> PyResult<()> {
            let foo = Foo { val: 123 };

            let cap = PyCapsule::new(py, foo, Some(NAME.to_owned()))?;
            assert!(cap.is_valid_checked(Some(NAME)));

            let foo_capi = unsafe { cap.reference_checked::<Foo>(Some(NAME.as_ref())) }.unwrap();
            assert_eq!(foo_capi.val, 123);
            assert_eq!(foo_capi.get_val(), 123);
            assert_eq!(unsafe { CStr::from_ptr(cap.name().unwrap()) }, NAME);
            Ok(())
        })
    }

    #[test]
    fn test_pycapsule_func() {
        fn foo(x: u32) -> u32 {
            x
        }

        let cap: Py<PyCapsule> = Python::attach(|py| {
            let cap = PyCapsule::new(py, foo as fn(u32) -> u32, Some(NAME.to_owned())).unwrap();
            cap.into()
        });

        Python::attach(move |py| {
            let f =
                unsafe { cap.bind(py).reference_checked::<fn(u32) -> u32>(Some(NAME)) }.unwrap();
            assert_eq!(f(123), 123);
        });
    }

    #[test]
    fn test_pycapsule_context() -> PyResult<()> {
        Python::attach(|py| {
            let cap = PyCapsule::new(py, 0, Some(NAME.to_owned()))?;

            let c = cap.context()?;
            assert!(c.is_null());

            let ctx = Box::new(123_u32);
            cap.set_context(Box::into_raw(ctx).cast())?;

            let ctx_ptr: *mut c_void = cap.context()?;
            let ctx = unsafe { *Box::from_raw(ctx_ptr.cast::<u32>()) };
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

        Python::attach(|py| {
            let foo = Foo { val: 123 };
            let name = ffi::c_str!("builtins.capsule");

            let capsule = PyCapsule::new(py, foo, Some(name.to_owned())).unwrap();

            let module = PyModule::import(py, "builtins").unwrap();
            module.add("capsule", capsule).unwrap();

            // check error when wrong named passed for capsule.
            let result: PyResult<&Foo> =
                unsafe { PyCapsule::import(py, ffi::c_str!("builtins.non_existant")) };
            assert!(result.is_err());

            // correct name is okay.
            let cap: &Foo = unsafe { PyCapsule::import(py, name) }.unwrap();
            assert_eq!(cap.val, 123);
            Ok(())
        })
    }

    #[test]
    fn test_vec_storage() {
        let cap: Py<PyCapsule> = Python::attach(|py| {
            let stuff: Vec<u8> = vec![1, 2, 3, 4];
            let cap = PyCapsule::new(py, stuff, Some(NAME.to_owned())).unwrap();
            cap.into()
        });

        Python::attach(move |py| {
            let stuff: &Vec<u8> = unsafe { cap.bind(py).reference_checked(Some(NAME)) }.unwrap();
            assert_eq!(stuff, &[1, 2, 3, 4]);
        })
    }

    #[test]
    fn test_vec_context() {
        let context: Vec<u8> = vec![1, 2, 3, 4];

        let cap: Py<PyCapsule> = Python::attach(|py| {
            let cap = PyCapsule::new(py, 0, Some(NAME.to_owned())).unwrap();
            cap.set_context(Box::into_raw(Box::new(&context)).cast())
                .unwrap();

            cap.into()
        });

        Python::attach(move |py| {
            let ctx_ptr: *mut c_void = cap.bind(py).context().unwrap();
            let ctx = unsafe { *Box::from_raw(ctx_ptr.cast::<&Vec<u8>>()) };
            assert_eq!(ctx, &vec![1_u8, 2, 3, 4]);
        })
    }

    #[test]
    fn test_pycapsule_destructor() {
        let (tx, rx) = channel::<bool>();

        fn destructor(_val: u32, ctx: *mut c_void) {
            assert!(!ctx.is_null());
            let context = unsafe { *Box::from_raw(ctx.cast::<Sender<bool>>()) };
            context.send(true).unwrap();
        }

        Python::attach(move |py| {
            let cap =
                PyCapsule::new_with_destructor(py, 0, Some(NAME.to_owned()), destructor).unwrap();
            cap.set_context(Box::into_raw(Box::new(tx)).cast()).unwrap();
        });

        // the destructor was called.
        assert_eq!(rx.recv(), Ok(true));
    }

    #[test]
    fn test_pycapsule_no_name() {
        Python::attach(|py| {
            let cap = PyCapsule::new(py, 0usize, None).unwrap();

            assert_eq!(
                unsafe { cap.reference_checked::<usize>(None) }.unwrap(),
                &0usize
            );
            assert_eq!(cap.name().unwrap(), std::ptr::null());
            assert_eq!(cap.context().unwrap(), std::ptr::null_mut());
        });
    }
}
