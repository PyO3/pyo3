use crate::err::{PyErr, PyResult};
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::{Borrowed, Bound};
use crate::py_result_ext::PyResultExt;
use crate::sync::with_critical_section;
use crate::{ffi, PyAny, Python};
use std::slice;

/// Represents a Python `bytearray`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyByteArray>`][crate::Py] or [`Bound<'py, PyByteArray>`][Bound].
///
/// For APIs available on `bytearray` objects, see the [`PyByteArrayMethods`] trait which is implemented for
/// [`Bound<'py, PyByteArray>`][Bound].
#[repr(transparent)]
pub struct PyByteArray(PyAny);

pyobject_native_type_core!(PyByteArray, pyobject_native_static_type_object!(ffi::PyByteArray_Type), #checkfunction=ffi::PyByteArray_Check);

impl PyByteArray {
    /// Creates a new Python bytearray object.
    ///
    /// The byte string is initialized by copying the data from the `&[u8]`.
    pub fn new<'py>(py: Python<'py>, src: &[u8]) -> Bound<'py, PyByteArray> {
        let ptr = src.as_ptr().cast();
        let len = src.len() as ffi::Py_ssize_t;
        unsafe {
            ffi::PyByteArray_FromStringAndSize(ptr, len)
                .assume_owned(py)
                .cast_into_unchecked()
        }
    }

    /// Creates a new Python `bytearray` object with an `init` closure to write its contents.
    /// Before calling `init` the bytearray is zero-initialised.
    /// * If Python raises a MemoryError on the allocation, `new_with` will return
    ///   it inside `Err`.
    /// * If `init` returns `Err(e)`, `new_with` will return `Err(e)`.
    /// * If `init` returns `Ok(())`, `new_with` will return `Ok(&PyByteArray)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pyo3::{prelude::*, types::PyByteArray};
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| -> PyResult<()> {
    ///     let py_bytearray = PyByteArray::new_with(py, 10, |bytes: &mut [u8]| {
    ///         bytes.copy_from_slice(b"Hello Rust");
    ///         Ok(())
    ///     })?;
    ///     let bytearray: &[u8] = unsafe { py_bytearray.as_bytes() };
    ///     assert_eq!(bytearray, b"Hello Rust");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn new_with<F>(py: Python<'_>, len: usize, init: F) -> PyResult<Bound<'_, PyByteArray>>
    where
        F: FnOnce(&mut [u8]) -> PyResult<()>,
    {
        unsafe {
            // Allocate buffer and check for an error
            let pybytearray: Bound<'_, Self> =
                ffi::PyByteArray_FromStringAndSize(std::ptr::null(), len as ffi::Py_ssize_t)
                    .assume_owned_or_err(py)?
                    .cast_into_unchecked();

            let buffer: *mut u8 = ffi::PyByteArray_AsString(pybytearray.as_ptr()).cast();
            debug_assert!(!buffer.is_null());
            // Zero-initialise the uninitialised bytearray
            std::ptr::write_bytes(buffer, 0u8, len);
            // (Further) Initialise the bytearray in init
            // If init returns an Err, pypybytearray will automatically deallocate the buffer
            init(std::slice::from_raw_parts_mut(buffer, len)).map(|_| pybytearray)
        }
    }

    /// Creates a new Python `bytearray` object from another Python object that
    /// implements the buffer protocol.
    pub fn from<'py>(src: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyByteArray>> {
        unsafe {
            ffi::PyByteArray_FromObject(src.as_ptr())
                .assume_owned_or_err(src.py())
                .cast_into_unchecked()
        }
    }
}

/// Implementation of functionality for [`PyByteArray`].
///
/// These methods are defined for the `Bound<'py, PyByteArray>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyByteArray")]
pub trait PyByteArrayMethods<'py>: crate::sealed::Sealed {
    /// Gets the length of the bytearray.
    fn len(&self) -> usize;

    /// Checks if the bytearray is empty.
    fn is_empty(&self) -> bool;

    /// Gets the start of the buffer containing the contents of the bytearray.
    ///
    /// # Safety
    ///
    /// See the safety requirements of [`PyByteArrayMethods::as_bytes`] and [`PyByteArrayMethods::as_bytes_mut`].
    fn data(&self) -> *mut u8;

    /// Extracts a slice of the `ByteArray`'s entire buffer.
    ///
    /// # Safety
    ///
    /// Mutation of the `bytearray` invalidates the slice. If it is used afterwards, the behavior is
    /// undefined.
    ///
    /// These mutations may occur in Python code as well as from Rust:
    /// - Calling methods like [`PyByteArrayMethods::as_bytes_mut`] and [`PyByteArrayMethods::resize`] will
    ///   invalidate the slice.
    /// - Actions like dropping objects or raising exceptions can invoke `__del__`methods or signal
    ///   handlers, which may execute arbitrary Python code. This means that if Python code has a
    ///   reference to the `bytearray` you cannot safely use the vast majority of PyO3's API whilst
    ///   using the slice.
    ///
    /// As a result, this slice should only be used for short-lived operations without executing any
    /// Python code, such as copying into a Vec.
    /// For free-threaded Python support see also [`with_critical_section`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::exceptions::PyRuntimeError;
    /// use pyo3::sync::with_critical_section;
    /// use pyo3::types::PyByteArray;
    ///
    /// #[pyfunction]
    /// fn a_valid_function(bytes: &Bound<'_, PyByteArray>) -> PyResult<()> {
    ///     let section = with_critical_section(bytes, || {
    ///         // SAFETY: We promise to not let the interpreter regain control over the bytearray
    ///         // or invoke any PyO3 APIs while using the slice.
    ///         let slice = unsafe { bytes.as_bytes() };
    ///
    ///         // Copy only a section of `bytes` while avoiding
    ///         // `to_vec` which copies the entire thing.
    ///         slice.get(6..11)
    ///             .map(Vec::from)
    ///             .ok_or_else(|| PyRuntimeError::new_err("input is not long enough"))
    ///     })?;
    ///
    ///     // Now we can do things with `section` and call PyO3 APIs again.
    ///     // ...
    ///     # assert_eq!(&section, b"world");
    ///
    ///     Ok(())
    /// }
    /// # fn main() -> PyResult<()> {
    /// #     Python::attach(|py| -> PyResult<()> {
    /// #         let fun = wrap_pyfunction!(a_valid_function, py)?;
    /// #         let locals = pyo3::types::PyDict::new(py);
    /// #         locals.set_item("a_valid_function", fun)?;
    /// #
    /// #         py.run(pyo3::ffi::c_str!(
    /// # r#"b = bytearray(b"hello world")
    /// # a_valid_function(b)
    /// #
    /// # try:
    /// #     a_valid_function(bytearray())
    /// # except RuntimeError as e:
    /// #     assert str(e) == 'input is not long enough'"#),
    /// #             None,
    /// #             Some(&locals),
    /// #         )?;
    /// #
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    ///
    /// # Incorrect usage
    ///
    /// The following `bug` function is unsound ⚠️
    ///
    /// ```rust,no_run
    /// # use pyo3::prelude::*;
    /// # use pyo3::types::PyByteArray;
    ///
    /// # #[allow(dead_code)]
    /// #[pyfunction]
    /// fn bug(py: Python<'_>, bytes: &Bound<'_, PyByteArray>) {
    ///     // No critical section is being used.
    ///     // This means that for no-gil Python another thread could be modifying the
    ///     // bytearray concurrently and thus invalidate `slice` any time.
    ///     let slice = unsafe { bytes.as_bytes() };
    ///
    ///     // This explicitly yields control back to the Python interpreter...
    ///     // ...but it's not always this obvious. Many things do this implicitly.
    ///     py.detach(|| {
    ///         // Python code could be mutating through its handle to `bytes`,
    ///         // which makes reading it a data race, which is undefined behavior.
    ///         println!("{:?}", slice[0]);
    ///     });
    ///
    ///     // Python code might have mutated it, so we can not rely on the slice
    ///     // remaining valid. As such this is also undefined behavior.
    ///     println!("{:?}", slice[0]);
    /// }
    /// ```
    unsafe fn as_bytes(&self) -> &[u8];

    /// Extracts a mutable slice of the `ByteArray`'s entire buffer.
    ///
    /// # Safety
    ///
    /// Any other accesses of the `bytearray`'s buffer invalidate the slice. If it is used
    /// afterwards, the behavior is undefined. The safety requirements of [`PyByteArrayMethods::as_bytes`]
    /// apply to this function as well.
    #[allow(clippy::mut_from_ref)]
    unsafe fn as_bytes_mut(&self) -> &mut [u8];

    /// Copies the contents of the bytearray to a Rust vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::types::PyByteArray;
    /// # Python::attach(|py| {
    /// let bytearray = PyByteArray::new(py, b"Hello World.");
    /// let mut copied_message = bytearray.to_vec();
    /// assert_eq!(b"Hello World.", copied_message.as_slice());
    ///
    /// copied_message[11] = b'!';
    /// assert_eq!(b"Hello World!", copied_message.as_slice());
    ///
    /// pyo3::py_run!(py, bytearray, "assert bytearray == b'Hello World.'");
    /// # });
    /// ```
    fn to_vec(&self) -> Vec<u8>;

    /// Resizes the bytearray object to the new length `len`.
    ///
    /// Note that this will invalidate any pointers obtained by [PyByteArrayMethods::data], as well as
    /// any (unsafe) slices obtained from [PyByteArrayMethods::as_bytes] and [PyByteArrayMethods::as_bytes_mut].
    fn resize(&self, len: usize) -> PyResult<()>;
}

impl<'py> PyByteArrayMethods<'py> for Bound<'py, PyByteArray> {
    #[inline]
    fn len(&self) -> usize {
        // non-negative Py_ssize_t should always fit into Rust usize
        unsafe { ffi::PyByteArray_Size(self.as_ptr()) as usize }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn data(&self) -> *mut u8 {
        self.as_borrowed().data()
    }

    unsafe fn as_bytes(&self) -> &[u8] {
        unsafe { self.as_borrowed().as_bytes() }
    }

    #[allow(clippy::mut_from_ref)]
    unsafe fn as_bytes_mut(&self) -> &mut [u8] {
        unsafe { self.as_borrowed().as_bytes_mut() }
    }

    fn to_vec(&self) -> Vec<u8> {
        with_critical_section(self, || {
            // SAFETY:
            //  * `self` is a `Bound` object, which guarantees that the Python GIL is held.
            //  * For no-gil Python, a critical section is used in lieu of the GIL.
            //  * We don't interact with the interpreter
            //  * We don't mutate the underlying slice
            unsafe { self.as_bytes() }.to_vec()
        })
    }

    fn resize(&self, len: usize) -> PyResult<()> {
        unsafe {
            let result = ffi::PyByteArray_Resize(self.as_ptr(), len as ffi::Py_ssize_t);
            if result == 0 {
                Ok(())
            } else {
                Err(PyErr::fetch(self.py()))
            }
        }
    }
}

impl<'a> Borrowed<'a, '_, PyByteArray> {
    fn data(&self) -> *mut u8 {
        unsafe { ffi::PyByteArray_AsString(self.as_ptr()).cast() }
    }

    #[allow(clippy::wrong_self_convention)]
    unsafe fn as_bytes(self) -> &'a [u8] {
        unsafe { slice::from_raw_parts(self.data(), self.len()) }
    }

    #[allow(clippy::wrong_self_convention)]
    unsafe fn as_bytes_mut(self) -> &'a mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data(), self.len()) }
    }
}

impl<'py> TryFrom<&Bound<'py, PyAny>> for Bound<'py, PyByteArray> {
    type Error = crate::PyErr;

    /// Creates a new Python `bytearray` object from another Python object that
    /// implements the buffer protocol.
    fn try_from(value: &Bound<'py, PyAny>) -> Result<Self, Self::Error> {
        PyByteArray::from(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyAnyMethods, PyByteArray, PyByteArrayMethods};
    use crate::{exceptions, Bound, Py, PyAny, Python};

    #[test]
    fn test_len() {
        Python::attach(|py| {
            let src = b"Hello Python";
            let bytearray = PyByteArray::new(py, src);
            assert_eq!(src.len(), bytearray.len());
        });
    }

    #[test]
    fn test_as_bytes() {
        Python::attach(|py| {
            let src = b"Hello Python";
            let bytearray = PyByteArray::new(py, src);

            let slice = unsafe { bytearray.as_bytes() };
            assert_eq!(src, slice);
            assert_eq!(bytearray.data() as *const _, slice.as_ptr());
        });
    }

    #[test]
    fn test_as_bytes_mut() {
        Python::attach(|py| {
            let src = b"Hello Python";
            let bytearray = PyByteArray::new(py, src);

            let slice = unsafe { bytearray.as_bytes_mut() };
            assert_eq!(src, slice);
            assert_eq!(bytearray.data(), slice.as_mut_ptr());

            slice[0..5].copy_from_slice(b"Hi...");

            assert_eq!(bytearray.str().unwrap(), "bytearray(b'Hi... Python')");
        });
    }

    #[test]
    fn test_to_vec() {
        Python::attach(|py| {
            let src = b"Hello Python";
            let bytearray = PyByteArray::new(py, src);

            let vec = bytearray.to_vec();
            assert_eq!(src, vec.as_slice());
        });
    }

    #[test]
    fn test_from() {
        Python::attach(|py| {
            let src = b"Hello Python";
            let bytearray = PyByteArray::new(py, src);

            let ba: Py<PyAny> = bytearray.into();
            let bytearray = PyByteArray::from(ba.bind(py)).unwrap();

            assert_eq!(src, unsafe { bytearray.as_bytes() });
        });
    }

    #[test]
    fn test_from_err() {
        Python::attach(|py| {
            if let Err(err) = PyByteArray::from(py.None().bind(py)) {
                assert!(err.is_instance_of::<exceptions::PyTypeError>(py));
            } else {
                panic!("error");
            }
        });
    }

    #[test]
    fn test_try_from() {
        Python::attach(|py| {
            let src = b"Hello Python";
            let bytearray: &Bound<'_, PyAny> = &PyByteArray::new(py, src);
            let bytearray: Bound<'_, PyByteArray> = TryInto::try_into(bytearray).unwrap();

            assert_eq!(src, unsafe { bytearray.as_bytes() });
        });
    }

    #[test]
    fn test_resize() {
        Python::attach(|py| {
            let src = b"Hello Python";
            let bytearray = PyByteArray::new(py, src);

            bytearray.resize(20).unwrap();
            assert_eq!(20, bytearray.len());
        });
    }

    #[test]
    fn test_byte_array_new_with() -> super::PyResult<()> {
        Python::attach(|py| -> super::PyResult<()> {
            let py_bytearray = PyByteArray::new_with(py, 10, |b: &mut [u8]| {
                b.copy_from_slice(b"Hello Rust");
                Ok(())
            })?;
            let bytearray: &[u8] = unsafe { py_bytearray.as_bytes() };
            assert_eq!(bytearray, b"Hello Rust");
            Ok(())
        })
    }

    #[test]
    fn test_byte_array_new_with_zero_initialised() -> super::PyResult<()> {
        Python::attach(|py| -> super::PyResult<()> {
            let py_bytearray = PyByteArray::new_with(py, 10, |_b: &mut [u8]| Ok(()))?;
            let bytearray: &[u8] = unsafe { py_bytearray.as_bytes() };
            assert_eq!(bytearray, &[0; 10]);
            Ok(())
        })
    }

    #[test]
    fn test_byte_array_new_with_error() {
        use crate::exceptions::PyValueError;
        Python::attach(|py| {
            let py_bytearray_result = PyByteArray::new_with(py, 10, |_b: &mut [u8]| {
                Err(PyValueError::new_err("Hello Crustaceans!"))
            });
            assert!(py_bytearray_result.is_err());
            assert!(py_bytearray_result
                .err()
                .unwrap()
                .is_instance_of::<PyValueError>(py));
        })
    }

    // * wasm has no threading support
    // * CPython 3.13t is unsound => test fails
    #[cfg(all(
        not(target_family = "wasm"),
        any(Py_3_14, not(all(Py_3_13, Py_GIL_DISABLED)))
    ))]
    #[test]
    fn test_data_integrity_in_critical_section() {
        use crate::instance::Py;
        use crate::sync::{with_critical_section, MutexExt};

        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Mutex;
        use std::thread;
        use std::thread::ScopedJoinHandle;
        use std::time::Duration;

        const SIZE: usize = 1_000_000;
        const DATA_VALUE: u8 = 42;

        fn make_byte_array(py: Python<'_>, size: usize, value: u8) -> Bound<'_, PyByteArray> {
            PyByteArray::new_with(py, size, |b| {
                b.fill(value);
                Ok(())
            })
            .unwrap()
        }

        let data: Mutex<Py<PyByteArray>> = Mutex::new(Python::attach(|py| {
            make_byte_array(py, SIZE, DATA_VALUE).unbind()
        }));

        fn get_data<'py>(
            data: &Mutex<Py<PyByteArray>>,
            py: Python<'py>,
        ) -> Bound<'py, PyByteArray> {
            data.lock_py_attached(py).unwrap().bind(py).clone()
        }

        fn set_data(data: &Mutex<Py<PyByteArray>>, new: Bound<'_, PyByteArray>) {
            let py = new.py();
            *data.lock_py_attached(py).unwrap() = new.unbind()
        }

        let running = AtomicBool::new(true);
        let extending = AtomicBool::new(false);

        // continuously extends and resets the bytearray in data
        let worker1 = || {
            let mut rounds = 0;
            while running.load(Ordering::SeqCst) && rounds < 50 {
                Python::attach(|py| {
                    let byte_array = get_data(&data, py);
                    extending.store(true, Ordering::SeqCst);
                    byte_array
                        .call_method("extend", (&byte_array,), None)
                        .unwrap();
                    extending.store(false, Ordering::SeqCst);
                    set_data(&data, make_byte_array(py, SIZE, DATA_VALUE));
                    rounds += 1;
                });
            }
        };

        // continuously checks the integrity of bytearray in data
        let worker2 = || {
            while running.load(Ordering::SeqCst) {
                if !extending.load(Ordering::SeqCst) {
                    // wait until we have a chance to read inconsistent state
                    continue;
                }
                Python::attach(|py| {
                    let read = get_data(&data, py);
                    if read.len() == SIZE {
                        // extend is still not done => wait even more
                        return;
                    }
                    with_critical_section(&read, || {
                        // SAFETY: we are in a critical section
                        // This is the whole point of the test: make sure that a
                        // critical section is sufficient to ensure that the data
                        // read is consistent.
                        unsafe {
                            let bytes = read.as_bytes();
                            assert!(bytes.iter().rev().take(50).all(|v| *v == DATA_VALUE
                                && bytes.iter().take(50).all(|v| *v == DATA_VALUE)));
                        }
                    });
                });
            }
        };

        thread::scope(|s| {
            let mut handle1 = Some(s.spawn(worker1));
            let mut handle2 = Some(s.spawn(worker2));
            let mut handles = [&mut handle1, &mut handle2];

            let t0 = std::time::Instant::now();
            while t0.elapsed() < Duration::from_secs(1) {
                for handle in &mut handles {
                    if handle
                        .as_ref()
                        .map(ScopedJoinHandle::is_finished)
                        .unwrap_or(false)
                    {
                        let res = handle.take().unwrap().join();
                        if res.is_err() {
                            running.store(false, Ordering::SeqCst);
                        }
                        res.unwrap();
                    }
                }
                if handles.iter().any(|handle| handle.is_none()) {
                    break;
                }
            }
            running.store(false, Ordering::SeqCst);
            for handle in &mut handles {
                if let Some(handle) = handle.take() {
                    handle.join().unwrap()
                }
            }
        });
    }
}
