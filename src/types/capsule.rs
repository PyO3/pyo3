#![deny(clippy::undocumented_unsafe_blocks)]

use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::{ffi, PyAny};
use crate::{Bound, Python};
use crate::{PyErr, PyResult};
use std::ffi::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};
use std::mem::offset_of;
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
///     let capsule = PyCapsule::new(py, foo, Some(c"builtins.capsule".to_owned()))?;
///
///     let module = PyModule::import(py, "builtins")?;
///     module.add("capsule", capsule)?;
///
///     let cap: &Foo = unsafe { PyCapsule::import(py, c"builtins.capsule")? };
///     assert_eq!(cap.val, 123);
///     Ok(())
/// });
/// assert!(r.is_ok());
/// ```
#[repr(transparent)]
pub struct PyCapsule(PyAny);

pyobject_native_type_core!(PyCapsule, pyobject_native_static_type_object!(ffi::PyCapsule_Type), "types", "CapsuleType", #checkfunction=ffi::PyCapsule_CheckExact);

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
    /// use std::ptr::NonNull;
    ///
    /// // this can be c"foo" on Rust 1.77+
    /// const NAME: &CStr = c"foo";
    ///
    /// Python::attach(|py| {
    ///     let capsule = PyCapsule::new(py, 123_u32, Some(NAME.to_owned())).unwrap();
    ///     let val: NonNull<u32> = capsule.pointer_checked(Some(NAME)).unwrap().cast();
    ///     assert_eq!(unsafe { *val.as_ref() }, 123);
    /// });
    /// ```
    ///
    /// However, attempting to construct a `PyCapsule` with a zero-sized type will not compile:
    ///
    /// ```compile_fail
    /// use pyo3::{prelude::*, types::PyCapsule};
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
        debug_assert_eq!(offset_of!(CapsuleContents::<T, F>, value), 0);

        let name_ptr = name.as_ref().map_or(std::ptr::null(), |name| name.as_ptr());
        let val = Box::into_raw(Box::new(CapsuleContents {
            value,
            destructor,
            name,
        }));

        // SAFETY:
        // - `val` is a non-null pointer to valid capsule data
        // - `name_ptr` is either a valid C string or null
        // - `destructor` will delete this data when called
        // - thread is attached to the Python interpreter
        // - `PyCapsule_New` returns a new reference or null on error
        unsafe {
            ffi::PyCapsule_New(val.cast(), name_ptr, Some(capsule_destructor::<T, F>))
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }
    }

    /// Constructs a new capsule from a raw pointer.
    ///
    /// Unlike [`PyCapsule::new`], which stores a value and sets the capsule's pointer
    /// to that value's address, this method uses the pointer directly. This is useful
    /// for APIs that expect the capsule to hold a specific address (e.g., a function
    /// pointer for FFI) rather than a pointer to owned data.
    ///
    /// The capsule's name should follow Python's naming convention:
    /// `"module.attribute"` for capsules stored as module attributes.
    ///
    /// # Safety
    ///
    /// - The pointer must be valid for its intended use case.
    /// - If the pointer refers to data, that data must outlive the capsule.
    /// - No destructor is registered; use [`PyCapsule::new_with_pointer_and_destructor`]
    ///   if cleanup is needed.
    ///
    /// # Example
    ///
    /// ```
    /// use pyo3::{prelude::*, types::PyCapsule};
    /// use std::ffi::c_void;
    /// use std::ptr::NonNull;
    ///
    /// extern "C" fn my_ffi_handler(_: *mut c_void) -> *mut c_void {
    ///     std::ptr::null_mut()
    /// }
    ///
    /// Python::attach(|py| {
    ///     let ptr = NonNull::new(my_ffi_handler as *mut c_void).unwrap();
    ///
    ///     // SAFETY: `ptr` is a valid function pointer
    ///     let capsule = unsafe {
    ///         PyCapsule::new_with_pointer(py, ptr, c"my_module.my_ffi_handler")
    ///     }.unwrap();
    ///
    ///     let retrieved = capsule.pointer_checked(Some(c"my_module.my_ffi_handler")).unwrap();
    ///     assert_eq!(retrieved.as_ptr(), my_ffi_handler as *mut c_void);
    /// });
    /// ```
    pub unsafe fn new_with_pointer<'py>(
        py: Python<'py>,
        pointer: NonNull<c_void>,
        name: &'static CStr,
    ) -> PyResult<Bound<'py, Self>> {
        // SAFETY: Caller guarantees pointer validity; destructor is None.
        unsafe { Self::new_with_pointer_and_destructor(py, pointer, name, None) }
    }

    /// Constructs a new capsule from a raw pointer with an optional destructor.
    ///
    /// This is the full-featured version of [`PyCapsule::new_with_pointer`], allowing
    /// a destructor to be called when the capsule is garbage collected.
    ///
    /// Unlike [`PyCapsule::new_with_destructor`], the destructor here must be a raw
    /// `extern "C"` function pointer, not a Rust closure. This is because there is
    /// no internal storage for a closure—the capsule holds only the raw pointer you
    /// provide.
    ///
    /// # Safety
    ///
    /// - The pointer must be valid for its intended use case.
    /// - If the pointer refers to data, that data must remain valid for the capsule's
    ///   lifetime, or the destructor must clean it up.
    /// - The destructor, if provided, must be safe to call from any thread.
    /// - The destructor should not panic. Panics cannot unwind across the FFI
    ///   boundary into Python, so a panic will abort the process.
    ///
    /// # Example
    ///
    /// ```
    /// use pyo3::{prelude::*, types::PyCapsule};
    /// use std::ffi::c_void;
    /// use std::ptr::NonNull;
    ///
    /// unsafe extern "C" fn free_data(capsule: *mut pyo3::ffi::PyObject) {
    ///     let ptr = pyo3::ffi::PyCapsule_GetPointer(capsule, c"my_module.data".as_ptr());
    ///     if !ptr.is_null() {
    ///         drop(Box::from_raw(ptr as *mut u32));
    ///     }
    /// }
    ///
    /// Python::attach(|py| {
    ///     let data = Box::new(42u32);
    ///     let ptr = NonNull::new(Box::into_raw(data).cast::<c_void>()).unwrap();
    ///
    ///     // SAFETY: `ptr` is valid; `free_data` will deallocate it
    ///     let capsule = unsafe {
    ///         PyCapsule::new_with_pointer_and_destructor(
    ///             py,
    ///             ptr,
    ///             c"my_module.data",
    ///             Some(free_data),
    ///         )
    ///     }.unwrap();
    /// });
    /// ```
    pub unsafe fn new_with_pointer_and_destructor<'py>(
        py: Python<'py>,
        pointer: NonNull<c_void>,
        name: &'static CStr,
        destructor: Option<ffi::PyCapsule_Destructor>,
    ) -> PyResult<Bound<'py, Self>> {
        let name_ptr = name.as_ptr();

        // SAFETY:
        // - `pointer` is non-null (guaranteed by `NonNull`)
        // - `name_ptr` points to a valid C string (guaranteed by `&'static CStr`)
        // - `destructor` is either None or a valid function pointer (caller guarantees)
        // - Thread is attached to the Python interpreter
        unsafe {
            ffi::PyCapsule_New(pointer.as_ptr(), name_ptr, destructor)
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
        // SAFETY: `name` is a valid C string, thread is attached to the Python interpreter
        let ptr = unsafe { ffi::PyCapsule_Import(name.as_ptr(), false as c_int) };
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            // SAFETY: caller has upheld the safety contract
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
/// Capsules contain pointers to arbitrary data which is cast to a specific type at runtime. This is
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
    /// Because this method encourages dereferencing the pointer for longer than necessary, it
    /// is deprecated. Prefer to use [`pointer_checked()`][PyCapsuleMethods::pointer_checked]
    /// and dereference the pointer only for as short a time as possible.
    ///
    /// # Safety
    ///
    /// This performs a dereference of the pointer returned from [`pointer()`][PyCapsuleMethods::pointer].
    ///
    /// See the safety notes on [`pointer_checked()`][PyCapsuleMethods::pointer_checked].
    #[deprecated(since = "0.27.0", note = "to be removed, see `pointer_checked()`")]
    unsafe fn reference<T>(&self) -> &T;

    /// Gets the raw pointer stored in this capsule, without checking its name.
    #[deprecated(since = "0.27.0", note = "use `pointer_checked()` instead")]
    fn pointer(&self) -> *mut c_void;

    /// Gets the raw pointer stored in this capsule.
    ///
    /// Returns an error if the capsule is not [valid][`PyCapsuleMethods::is_valid_checked`] with the given `name`.
    ///
    /// # Safety
    ///
    /// This function itself is not `unsafe`, but dereferencing the returned pointer to produce a reference
    /// is very dangerous:
    /// - The pointer will need to be [.cast()][NonNull::cast] to a concrete type before dereferencing.
    ///   As per [name checking](#name-checking), there is no way to statically guarantee this cast is
    ///   correct, the name is the best available hint to guard against accidental misuse.
    /// - Arbitrary Python code can change the contents of the capsule, which may invalidate the
    ///   pointer. The pointer and the reference produced by dereferencing the pointer should both
    ///   be considered invalid after arbitrary Python code has run.
    ///
    /// Users should take care to cast to the correct type and consume the pointer for as little
    /// duration as possible.
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

    /// Retrieves the name of this capsule, if set.
    ///
    /// Returns an error if this capsule is not valid.
    ///
    /// See [`CapsuleName`] for details of how to consume the return value.
    fn name(&self) -> PyResult<Option<CapsuleName>>;
}

impl<'py> PyCapsuleMethods<'py> for Bound<'py, PyCapsule> {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn set_context(&self, context: *mut c_void) -> PyResult<()> {
        // SAFETY:
        // - `self.as_ptr()` is a valid object pointer
        // - `context` is user-provided
        // - thread is attached to the Python interpreter
        let result = unsafe { ffi::PyCapsule_SetContext(self.as_ptr(), context) };
        if result != 0 {
            Err(PyErr::fetch(self.py()))
        } else {
            Ok(())
        }
    }

    fn context(&self) -> PyResult<*mut c_void> {
        // SAFETY:
        // - `self.as_ptr()` is a valid object pointer
        // - thread is attached to the Python interpreter
        let ctx = unsafe { ffi::PyCapsule_GetContext(self.as_ptr()) };
        if ctx.is_null() {
            ensure_no_error(self.py())?
        }
        Ok(ctx)
    }

    #[allow(deprecated)]
    unsafe fn reference<T>(&self) -> &T {
        // SAFETY:
        // - caller has upheld the safety contract
        // - thread is attached to the Python interpreter
        unsafe { &*self.pointer().cast() }
    }

    fn pointer(&self) -> *mut c_void {
        // SAFETY: arguments to `PyCapsule_GetPointer` are valid, errors are handled properly
        unsafe {
            let ptr = ffi::PyCapsule_GetPointer(self.as_ptr(), name_ptr_ignore_error(self));
            if ptr.is_null() {
                ffi::PyErr_Clear();
            }
            ptr
        }
    }

    fn pointer_checked(&self, name: Option<&CStr>) -> PyResult<NonNull<c_void>> {
        // SAFETY:
        // - `self.as_ptr()` is a valid object pointer
        // - `name_ptr` is either a valid C string or null
        // - thread is attached to the Python interpreter
        let ptr = unsafe { ffi::PyCapsule_GetPointer(self.as_ptr(), name_ptr(name)) };
        NonNull::new(ptr).ok_or_else(|| PyErr::fetch(self.py()))
    }

    fn is_valid(&self) -> bool {
        // SAFETY: As well as if the stored pointer is null, PyCapsule_IsValid also returns false if
        // self.as_ptr() is null or not a ptr to a PyCapsule object. Both of these are guaranteed
        // to not be the case thanks to invariants of this PyCapsule struct.
        let r = unsafe { ffi::PyCapsule_IsValid(self.as_ptr(), name_ptr_ignore_error(self)) };
        r != 0
    }

    fn is_valid_checked(&self, name: Option<&CStr>) -> bool {
        // SAFETY:
        // - `self.as_ptr()` is a valid object pointer
        // - `name_ptr` is either a valid C string or null
        // - thread is attached to the Python interpreter
        let r = unsafe { ffi::PyCapsule_IsValid(self.as_ptr(), name_ptr(name)) };
        r != 0
    }

    fn name(&self) -> PyResult<Option<CapsuleName>> {
        // SAFETY:
        // - `self.as_ptr()` is a valid object pointer
        // - thread is attached to the Python interpreter
        let name = unsafe { ffi::PyCapsule_GetName(self.as_ptr()) };

        match NonNull::new(name.cast_mut()) {
            Some(name) => Ok(Some(CapsuleName { ptr: name })),
            None => {
                ensure_no_error(self.py())?;
                Ok(None)
            }
        }
    }
}

/// The name given to a `capsule` object.
///
/// This is a thin wrapper around `*const c_char`, which can be accessed with the [`as_ptr`][Self::as_ptr]
/// method. The [`as_cstr`][Self::as_cstr] method can be used as a convenience to access the name as a `&CStr`.
///
/// There is no guarantee that this capsule name pointer valid for any length of time, as arbitrary
/// Python code may change the name of a capsule object (by reaching native code which calls
/// [`PyCapsule_SetName`][ffi::PyCapsule_SetName]). See the safety notes on [`as_cstr`][Self::as_cstr].
#[derive(Clone, Copy)]
pub struct CapsuleName {
    /// Pointer to the name c-string, known to be non-null.
    ptr: NonNull<c_char>,
}

impl CapsuleName {
    /// Returns the capsule name as a `&CStr`.
    ///
    /// Note: this method is a thin wrapper around [`CStr::from_ptr`] so (as of Rust 1.91) incurs a
    /// length calculation on each call.
    ///
    /// # Safety
    ///
    /// There is no guarantee that the capsule name remains valid for any length of time, as arbitrary
    /// Python code may change the name of the capsule. The caller should be aware of any conventions
    /// of the capsule in question related to the lifetime of the name (many capsule names are
    /// statically allocated, i.e. have the `'static` lifetime, but Python does not require this).
    ///
    /// The returned lifetime `'a` is not related to the lifetime of the capsule itself, and the caller is
    /// responsible for using the `&CStr` for as short a time as possible.
    pub unsafe fn as_cstr<'a>(self) -> &'a CStr {
        // SAFETY: caller has upheld the safety contract
        unsafe { CStr::from_ptr(self.as_ptr()) }
    }

    /// Returns the raw pointer to the capsule name.
    pub fn as_ptr(self) -> *const c_char {
        self.ptr.as_ptr().cast_const()
    }
}

// C layout, as casting the capsule pointer to `T` depends on `T` being first.
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
    /// Gets the pointer and context from the capsule.
    ///
    /// # Safety
    ///
    /// - `capsule` must be a valid capsule object
    unsafe fn get_pointer_ctx(capsule: *mut ffi::PyObject) -> (*mut c_void, *mut c_void) {
        // SAFETY: `capsule` is known to be a borrowed reference to the capsule being destroyed
        let name = unsafe { ffi::PyCapsule_GetName(capsule) };

        // SAFETY:
        // - `capsule` is known to be a borrowed reference to the capsule being destroyed
        // - `name` is known to be the capsule's name
        let ptr = unsafe { ffi::PyCapsule_GetPointer(capsule, name) };

        // SAFETY:
        // - `capsule` is known to be a borrowed reference to the capsule being destroyed
        let ctx = unsafe { ffi::PyCapsule_GetContext(capsule) };

        (ptr, ctx)
    }

    // SAFETY: `capsule` is known to be a valid capsule object
    let (ptr, ctx) = unsafe { get_pointer_ctx(capsule) };

    // SAFETY: `capsule` was knowingly constructed with a boxed `CapsuleContents<T, F>`
    // and is now being destroyed, so we can move the data from the box.
    let CapsuleContents::<T, F> {
        value, destructor, ..
    } = *unsafe { Box::from_raw(ptr.cast()) };

    destructor(value, ctx);
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
    // SAFETY:
    // - `slf` is known to be a valid capsule object
    // - thread is attached to the Python interpreter
    let ptr = unsafe { ffi::PyCapsule_GetName(slf.as_ptr()) };
    if ptr.is_null() {
        // SAFETY: thread is attached to the Python interpreter
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
    use crate::prelude::PyModule;
    use crate::types::capsule::PyCapsuleMethods;
    use crate::types::module::PyModuleMethods;
    use crate::{types::PyCapsule, Py, PyResult, Python};
    use std::ffi::{c_void, CStr};
    use std::ptr::NonNull;
    use std::sync::mpsc::{channel, Sender};

    const NAME: &CStr = c"foo";

    #[test]
    fn test_pycapsule_struct() {
        #[repr(C)]
        struct Foo {
            pub val: u32,
        }

        impl Foo {
            fn get_val(&self) -> u32 {
                self.val
            }
        }

        Python::attach(|py| {
            let foo = Foo { val: 123 };

            let cap = PyCapsule::new(py, foo, Some(NAME.to_owned())).unwrap();
            assert!(cap.is_valid_checked(Some(NAME)));

            let foo_capi = cap.pointer_checked(Some(NAME)).unwrap().cast::<Foo>();
            // SAFETY: `foo_capi` contains a `Foo` and will be valid for the duration of the assert
            assert_eq!(unsafe { foo_capi.as_ref() }.val, 123);
            // SAFETY: as above
            assert_eq!(unsafe { foo_capi.as_ref() }.get_val(), 123);
            assert_eq!(
                // SAFETY: `cap.name()` has a non-null name
                unsafe { CStr::from_ptr(cap.name().unwrap().unwrap().as_ptr()) },
                NAME
            );
            // SAFETY: as above
            assert_eq!(unsafe { cap.name().unwrap().unwrap().as_cstr() }, NAME)
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
            let f = cap
                .bind(py)
                .pointer_checked(Some(NAME))
                .unwrap()
                .cast::<fn(u32) -> u32>();
            // SAFETY: `f` contains a `fn(u32) -> u32` and will be valid for the duration of the assert
            assert_eq!(unsafe { f.as_ref() }(123), 123);
        });
    }

    #[test]
    fn test_pycapsule_context() {
        Python::attach(|py| {
            let cap = PyCapsule::new(py, 0, Some(NAME.to_owned())).unwrap();

            let c = cap.context().unwrap();
            assert!(c.is_null());

            let ctx = Box::new(123_u32);
            cap.set_context(Box::into_raw(ctx).cast()).unwrap();

            let ctx_ptr: *mut c_void = cap.context().unwrap();
            // SAFETY: `ctx_ptr` contains a boxed `u32` which is being moved out of the capsule
            let ctx = unsafe { *Box::from_raw(ctx_ptr.cast::<u32>()) };
            assert_eq!(ctx, 123);
        })
    }

    #[test]
    fn test_pycapsule_import() {
        #[repr(C)]
        struct Foo {
            pub val: u32,
        }

        Python::attach(|py| {
            let foo = Foo { val: 123 };
            let name = c"builtins.capsule";

            let capsule = PyCapsule::new(py, foo, Some(name.to_owned())).unwrap();

            let module = PyModule::import(py, "builtins").unwrap();
            module.add("capsule", capsule).unwrap();

            // check error when wrong named passed for capsule.
            // SAFETY: this function will fail so the cast is never done
            let result: PyResult<&Foo> = unsafe { PyCapsule::import(py, c"builtins.non_existent") };
            assert!(result.is_err());

            // correct name is okay.
            // SAFETY: we know the capsule at `name` contains a `Foo`
            let cap: &Foo = unsafe { PyCapsule::import(py, name) }.unwrap();
            assert_eq!(cap.val, 123);
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
            let stuff = cap
                .bind(py)
                .pointer_checked(Some(NAME))
                .unwrap()
                .cast::<Vec<u8>>();
            // SAFETY: `stuff` contains a `Vec<u8>` and will be valid for the duration of the assert
            assert_eq!(unsafe { stuff.as_ref() }, &[1, 2, 3, 4]);
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
            // SAFETY: `ctx_ptr` contains a boxed `&Vec<u8>` which is being moved out of the capsule
            let ctx = unsafe { *Box::from_raw(ctx_ptr.cast::<&Vec<u8>>()) };
            assert_eq!(ctx, &vec![1_u8, 2, 3, 4]);
        })
    }

    #[test]
    fn test_pycapsule_destructor() {
        let (tx, rx) = channel::<bool>();

        fn destructor(_val: u32, ctx: *mut c_void) {
            assert!(!ctx.is_null());
            // SAFETY: `ctx` is known to be a boxed `Sender<bool>` needing deletion
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
                // SAFETY: `cap` is known to contain a `usize`
                unsafe { cap.pointer_checked(None).unwrap().cast::<usize>().as_ref() },
                &0usize
            );
            assert!(cap.name().unwrap().is_none());
            assert_eq!(cap.context().unwrap(), std::ptr::null_mut());
        });
    }

    #[test]
    fn test_pycapsule_new_with_pointer() {
        extern "C" fn dummy_handler(_: *mut c_void) -> *mut c_void {
            std::ptr::null_mut()
        }

        let fn_ptr =
            NonNull::new(dummy_handler as *mut c_void).expect("function pointer is non-null");

        Python::attach(|py| {
            // SAFETY: `fn_ptr` is known to point to `dummy_handler`
            let capsule =
                unsafe { PyCapsule::new_with_pointer(py, fn_ptr, c"test.dummy_handler") }.unwrap();

            let retrieved_ptr = capsule
                .pointer_checked(Some(c"test.dummy_handler"))
                .unwrap();
            assert_eq!(retrieved_ptr.as_ptr(), fn_ptr.as_ptr());
        });
    }

    #[test]
    fn test_pycapsule_new_with_pointer_and_destructor() {
        use std::sync::mpsc::{channel, TryRecvError};

        let (tx, rx) = channel::<bool>();

        unsafe extern "C" fn destructor_fn(capsule: *mut crate::ffi::PyObject) {
            // SAFETY:
            // - `capsule` is a valid capsule object being destroyed by Python
            // - The context was set to a valid `Box<Sender<bool>>` below
            unsafe {
                let ctx = crate::ffi::PyCapsule_GetContext(capsule);
                if !ctx.is_null() {
                    let sender: Box<Sender<bool>> = Box::from_raw(ctx.cast());
                    let _ = sender.send(true);
                }
            }
        }

        let dummy_ptr =
            NonNull::new(0xDEADBEEF as *mut c_void).expect("function pointer is non-null");

        Python::attach(|py| {
            // SAFETY:
            // - `dummy_ptr` is non-null (it's a made-up address for testing)
            // - We're providing a valid destructor function
            let capsule = unsafe {
                PyCapsule::new_with_pointer_and_destructor(
                    py,
                    dummy_ptr,
                    c"test.destructor_capsule",
                    Some(destructor_fn),
                )
            }
            .unwrap();

            // Store the sender in the capsule's context
            let sender_box = Box::new(tx);
            capsule
                .set_context(Box::into_raw(sender_box).cast())
                .unwrap();

            // The destructor hasn't fired yet
            assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
        });

        // After Python::attach scope ends, the capsule should be destroyed
        assert_eq!(rx.recv(), Ok(true));
    }

    #[test]
    fn test_pycapsule_pointer_checked_wrong_name() {
        Python::attach(|py| {
            let cap = PyCapsule::new(py, 123u32, Some(c"correct.name".to_owned())).unwrap();

            // Requesting with wrong name should fail
            let result = cap.pointer_checked(Some(c"wrong.name"));
            assert!(result.is_err());

            // Requesting with None when capsule has a name should also fail
            let result = cap.pointer_checked(None);
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_pycapsule_pointer_checked_none_vs_some() {
        Python::attach(|py| {
            // Capsule with no name
            let cap_no_name = PyCapsule::new(py, 123u32, None).unwrap();

            // Should succeed with None
            assert!(cap_no_name.pointer_checked(None).is_ok());

            // Should fail with Some(name)
            let result = cap_no_name.pointer_checked(Some(c"some.name"));
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_pycapsule_is_valid_checked_wrong_name() {
        Python::attach(|py| {
            let cap = PyCapsule::new(py, 123u32, Some(c"correct.name".to_owned())).unwrap();

            // Should be valid with correct name
            assert!(cap.is_valid_checked(Some(c"correct.name")));

            // Should be invalid with wrong name
            assert!(!cap.is_valid_checked(Some(c"wrong.name")));

            // Should be invalid with None when capsule has a name
            assert!(!cap.is_valid_checked(None));
        });
    }

    #[test]
    fn test_pycapsule_is_valid_checked_no_name() {
        Python::attach(|py| {
            let cap = PyCapsule::new(py, 123u32, None).unwrap();

            // Should be valid with None
            assert!(cap.is_valid_checked(None));

            // Should be invalid with any name
            assert!(!cap.is_valid_checked(Some(c"any.name")));
        });
    }

    #[test]
    fn test_pycapsule_context_on_invalid_capsule() {
        Python::attach(|py| {
            let cap = PyCapsule::new(py, 123u32, Some(NAME.to_owned())).unwrap();

            // Invalidate the capsule
            // SAFETY: intentionally breaking the capsule for testing
            unsafe {
                crate::ffi::PyCapsule_SetPointer(cap.as_ptr(), std::ptr::null_mut());
            }

            // context() on invalid capsule should fail
            let result = cap.context();
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_pycapsule_import_wrong_module() {
        Python::attach(|py| {
            // Try to import from a non-existent module
            // SAFETY: we expect this to fail, no cast will occur
            let result: PyResult<&u32> =
                unsafe { PyCapsule::import(py, c"nonexistent_module.capsule") };
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_pycapsule_import_wrong_attribute() {
        Python::attach(|py| {
            // Create a capsule and register it
            let cap = PyCapsule::new(py, 123u32, Some(c"builtins.test_cap".to_owned())).unwrap();
            let module = crate::prelude::PyModule::import(py, "builtins").unwrap();
            module.add("test_cap", cap).unwrap();

            // Try to import with wrong attribute name
            // SAFETY: we expect this to fail
            let result: PyResult<&u32> =
                unsafe { PyCapsule::import(py, c"builtins.wrong_attribute") };
            assert!(result.is_err());
        });
    }
}
