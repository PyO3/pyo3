#![cfg(any(not(Py_LIMITED_API), Py_3_11))]
// Copyright (c) 2017 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

//! `PyBuffer` implementation
#[cfg(feature = "experimental-inspect")]
use crate::inspect::{type_hint_identifier, PyStaticExpr};
use crate::{err, exceptions::PyBufferError, ffi, FromPyObject, PyAny, PyResult, Python};
use crate::{Borrowed, Bound, PyErr};
use std::ffi::{
    c_char, c_int, c_long, c_longlong, c_schar, c_short, c_uchar, c_uint, c_ulong, c_ulonglong,
    c_ushort, c_void,
};
use std::marker::{PhantomData, PhantomPinned};
use std::pin::Pin;
use std::ptr::NonNull;
use std::{cell, mem, ptr, slice};
use std::{ffi::CStr, fmt::Debug};

/// A typed form of [`PyUntypedBuffer`].
#[repr(transparent)]
pub struct PyBuffer<T>(PyUntypedBuffer, PhantomData<[T]>);

/// Allows access to the underlying buffer used by a python object such as `bytes`, `bytearray` or `array.array`.
#[repr(transparent)]
pub struct PyUntypedBuffer(
    // It is common for exporters filling `Py_buffer` struct to make it self-referential, e.g. see
    // implementation of
    // [`PyBuffer_FillInfo`](https://github.com/python/cpython/blob/2fd43a1ffe4ff1f6c46f6045bc327d6085c40fbf/Objects/abstract.c#L798-L802).
    //
    // Therefore we use `Pin<Box<...>>` to document for ourselves that the memory address of the `Py_buffer` is expected to be stable
    Pin<Box<RawBuffer>>,
);

/// Wrapper around `ffi::Py_buffer` to be `!Unpin`.
#[repr(transparent)]
struct RawBuffer(ffi::Py_buffer, PhantomPinned);

// PyBuffer send & sync guarantees are upheld by Python.
unsafe impl Send for PyUntypedBuffer {}
unsafe impl Sync for PyUntypedBuffer {}

fn debug_buffer(
    name: &str,
    raw: &ffi::Py_buffer,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let ndim = raw.ndim as usize;
    let format = NonNull::new(raw.format).map(|p| unsafe { CStr::from_ptr(p.as_ptr()) });
    let shape = NonNull::new(raw.shape)
        .map(|p| unsafe { slice::from_raw_parts(p.as_ptr().cast::<usize>(), ndim) });
    let strides =
        NonNull::new(raw.strides).map(|p| unsafe { slice::from_raw_parts(p.as_ptr(), ndim) });
    let suboffsets =
        NonNull::new(raw.suboffsets).map(|p| unsafe { slice::from_raw_parts(p.as_ptr(), ndim) });

    f.debug_struct(name)
        .field("buf", &raw.buf)
        .field("obj", &raw.obj)
        .field("len", &raw.len)
        .field("itemsize", &raw.itemsize)
        .field("readonly", &raw.readonly)
        .field("ndim", &raw.ndim)
        .field("format", &format)
        .field("shape", &shape)
        .field("strides", &strides)
        .field("suboffsets", &suboffsets)
        .field("internal", &raw.internal)
        .finish()
}

impl<T> Debug for PyBuffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_buffer("PyBuffer", self.raw(), f)
    }
}

impl Debug for PyUntypedBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_buffer("PyUntypedBuffer", self.raw(), f)
    }
}

/// Represents the type of a Python buffer element.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ElementType {
    /// A signed integer type.
    SignedInteger {
        /// The width of the signed integer in bytes.
        bytes: usize,
    },
    /// An unsigned integer type.
    UnsignedInteger {
        /// The width of the unsigned integer in bytes.
        bytes: usize,
    },
    /// A boolean type.
    Bool,
    /// A float type.
    Float {
        /// The width of the float in bytes.
        bytes: usize,
    },
    /// An unknown type. This may occur when parsing has failed.
    Unknown,
}

impl ElementType {
    /// Determines the `ElementType` from a Python `struct` module format string.
    ///
    /// See <https://docs.python.org/3/library/struct.html#format-strings> for more information
    /// about struct format strings.
    pub fn from_format(format: &CStr) -> ElementType {
        match format.to_bytes() {
            [size] | [b'@', size] => native_element_type_from_type_char(*size),
            [b'=' | b'<' | b'>' | b'!', size] => standard_element_type_from_type_char(*size),
            _ => ElementType::Unknown,
        }
    }
}

fn native_element_type_from_type_char(type_char: u8) -> ElementType {
    use self::ElementType::*;
    match type_char {
        b'c' => UnsignedInteger {
            bytes: mem::size_of::<c_char>(),
        },
        b'b' => SignedInteger {
            bytes: mem::size_of::<c_schar>(),
        },
        b'B' => UnsignedInteger {
            bytes: mem::size_of::<c_uchar>(),
        },
        b'?' => Bool,
        b'h' => SignedInteger {
            bytes: mem::size_of::<c_short>(),
        },
        b'H' => UnsignedInteger {
            bytes: mem::size_of::<c_ushort>(),
        },
        b'i' => SignedInteger {
            bytes: mem::size_of::<c_int>(),
        },
        b'I' => UnsignedInteger {
            bytes: mem::size_of::<c_uint>(),
        },
        b'l' => SignedInteger {
            bytes: mem::size_of::<c_long>(),
        },
        b'L' => UnsignedInteger {
            bytes: mem::size_of::<c_ulong>(),
        },
        b'q' => SignedInteger {
            bytes: mem::size_of::<c_longlong>(),
        },
        b'Q' => UnsignedInteger {
            bytes: mem::size_of::<c_ulonglong>(),
        },
        b'n' => SignedInteger {
            bytes: mem::size_of::<libc::ssize_t>(),
        },
        b'N' => UnsignedInteger {
            bytes: mem::size_of::<libc::size_t>(),
        },
        b'e' => Float { bytes: 2 },
        b'f' => Float { bytes: 4 },
        b'd' => Float { bytes: 8 },
        _ => Unknown,
    }
}

fn standard_element_type_from_type_char(type_char: u8) -> ElementType {
    use self::ElementType::*;
    match type_char {
        b'c' | b'B' => UnsignedInteger { bytes: 1 },
        b'b' => SignedInteger { bytes: 1 },
        b'?' => Bool,
        b'h' => SignedInteger { bytes: 2 },
        b'H' => UnsignedInteger { bytes: 2 },
        b'i' | b'l' => SignedInteger { bytes: 4 },
        b'I' | b'L' => UnsignedInteger { bytes: 4 },
        b'q' => SignedInteger { bytes: 8 },
        b'Q' => UnsignedInteger { bytes: 8 },
        b'e' => Float { bytes: 2 },
        b'f' => Float { bytes: 4 },
        b'd' => Float { bytes: 8 },
        _ => Unknown,
    }
}

#[cfg(target_endian = "little")]
fn is_matching_endian(c: u8) -> bool {
    c == b'@' || c == b'=' || c == b'>'
}

#[cfg(target_endian = "big")]
fn is_matching_endian(c: u8) -> bool {
    c == b'@' || c == b'=' || c == b'>' || c == b'!'
}

/// Trait implemented for possible element types of `PyBuffer`.
///
/// # Safety
///
/// This trait must only be implemented for types which represent valid elements of Python buffers.
pub unsafe trait Element: Copy {
    /// Gets whether the element specified in the format string is potentially compatible.
    /// Alignment and size are checked separately from this function.
    fn is_compatible_format(format: &CStr) -> bool;
}

impl<T: Element> FromPyObject<'_, '_> for PyBuffer<T> {
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = type_hint_identifier!("collections.abc", "Buffer");

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<PyBuffer<T>, Self::Error> {
        Self::get(&obj)
    }
}

impl<T: Element> PyBuffer<T> {
    /// Gets the underlying buffer from the specified python object.
    pub fn get(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        PyUntypedBuffer::get(obj)?.into_typed()
    }

    /// Gets the buffer memory as a slice.
    ///
    /// This function succeeds if:
    /// * the buffer format is compatible with `T`
    /// * alignment and size of buffer elements is matching the expectations for type `T`
    /// * the buffer is C-style contiguous
    ///
    /// The returned slice uses type `Cell<T>` because it's theoretically possible for any call into the Python runtime
    /// to modify the values in the slice.
    pub fn as_slice<'a>(&'a self, _py: Python<'a>) -> Option<&'a [ReadOnlyCell<T>]> {
        if self.is_c_contiguous() {
            unsafe {
                Some(slice::from_raw_parts(
                    self.raw().buf.cast(),
                    self.item_count(),
                ))
            }
        } else {
            None
        }
    }

    /// Gets the buffer memory as a slice.
    ///
    /// This function succeeds if:
    /// * the buffer is not read-only
    /// * the buffer format is compatible with `T`
    /// * alignment and size of buffer elements is matching the expectations for type `T`
    /// * the buffer is C-style contiguous
    ///
    /// The returned slice uses type `Cell<T>` because it's theoretically possible for any call into the Python runtime
    /// to modify the values in the slice.
    pub fn as_mut_slice<'a>(&'a self, _py: Python<'a>) -> Option<&'a [cell::Cell<T>]> {
        if !self.readonly() && self.is_c_contiguous() {
            unsafe {
                Some(slice::from_raw_parts(
                    self.raw().buf.cast(),
                    self.item_count(),
                ))
            }
        } else {
            None
        }
    }

    /// Gets the buffer memory as a slice.
    ///
    /// This function succeeds if:
    /// * the buffer format is compatible with `T`
    /// * alignment and size of buffer elements is matching the expectations for type `T`
    /// * the buffer is Fortran-style contiguous
    ///
    /// The returned slice uses type `Cell<T>` because it's theoretically possible for any call into the Python runtime
    /// to modify the values in the slice.
    pub fn as_fortran_slice<'a>(&'a self, _py: Python<'a>) -> Option<&'a [ReadOnlyCell<T>]> {
        if mem::size_of::<T>() == self.item_size() && self.is_fortran_contiguous() {
            unsafe {
                Some(slice::from_raw_parts(
                    self.raw().buf.cast(),
                    self.item_count(),
                ))
            }
        } else {
            None
        }
    }

    /// Gets the buffer memory as a slice.
    ///
    /// This function succeeds if:
    /// * the buffer is not read-only
    /// * the buffer format is compatible with `T`
    /// * alignment and size of buffer elements is matching the expectations for type `T`
    /// * the buffer is Fortran-style contiguous
    ///
    /// The returned slice uses type `Cell<T>` because it's theoretically possible for any call into the Python runtime
    /// to modify the values in the slice.
    pub fn as_fortran_mut_slice<'a>(&'a self, _py: Python<'a>) -> Option<&'a [cell::Cell<T>]> {
        if !self.readonly() && self.is_fortran_contiguous() {
            unsafe {
                Some(slice::from_raw_parts(
                    self.raw().buf.cast(),
                    self.item_count(),
                ))
            }
        } else {
            None
        }
    }

    /// Copies the buffer elements to the specified slice.
    /// If the buffer is multi-dimensional, the elements are written in C-style order.
    ///
    ///  * Fails if the slice does not have the correct length (`buf.item_count()`).
    ///  * Fails if the buffer format is not compatible with type `T`.
    ///
    /// To check whether the buffer format is compatible before calling this method,
    /// you can use `<T as buffer::Element>::is_compatible_format(buf.format())`.
    /// Alternatively, `match buffer::ElementType::from_format(buf.format())`.
    pub fn copy_to_slice(&self, py: Python<'_>, target: &mut [T]) -> PyResult<()> {
        self._copy_to_slice(py, target, b'C')
    }

    /// Copies the buffer elements to the specified slice.
    /// If the buffer is multi-dimensional, the elements are written in Fortran-style order.
    ///
    ///  * Fails if the slice does not have the correct length (`buf.item_count()`).
    ///  * Fails if the buffer format is not compatible with type `T`.
    ///
    /// To check whether the buffer format is compatible before calling this method,
    /// you can use `<T as buffer::Element>::is_compatible_format(buf.format())`.
    /// Alternatively, `match buffer::ElementType::from_format(buf.format())`.
    pub fn copy_to_fortran_slice(&self, py: Python<'_>, target: &mut [T]) -> PyResult<()> {
        self._copy_to_slice(py, target, b'F')
    }

    fn _copy_to_slice(&self, py: Python<'_>, target: &mut [T], fort: u8) -> PyResult<()> {
        if mem::size_of_val(target) != self.len_bytes() {
            return Err(PyBufferError::new_err(format!(
                "slice to copy to (of length {}) does not match buffer length of {}",
                target.len(),
                self.item_count()
            )));
        }

        err::error_on_minusone(py, unsafe {
            ffi::PyBuffer_ToContiguous(
                target.as_mut_ptr().cast(),
                #[cfg(Py_3_11)]
                self.raw(),
                #[cfg(not(Py_3_11))]
                ptr::from_ref(self.raw()).cast_mut(),
                self.raw().len,
                fort as std::ffi::c_char,
            )
        })
    }

    /// Copies the buffer elements to a newly allocated vector.
    /// If the buffer is multi-dimensional, the elements are written in C-style order.
    ///
    /// Fails if the buffer format is not compatible with type `T`.
    pub fn to_vec(&self, py: Python<'_>) -> PyResult<Vec<T>> {
        self._to_vec(py, b'C')
    }

    /// Copies the buffer elements to a newly allocated vector.
    /// If the buffer is multi-dimensional, the elements are written in Fortran-style order.
    ///
    /// Fails if the buffer format is not compatible with type `T`.
    pub fn to_fortran_vec(&self, py: Python<'_>) -> PyResult<Vec<T>> {
        self._to_vec(py, b'F')
    }

    fn _to_vec(&self, py: Python<'_>, fort: u8) -> PyResult<Vec<T>> {
        let item_count = self.item_count();
        let mut vec: Vec<T> = Vec::with_capacity(item_count);

        // Copy the buffer into the uninitialized space in the vector.
        // Due to T:Copy, we don't need to be concerned with Drop impls.
        err::error_on_minusone(py, unsafe {
            ffi::PyBuffer_ToContiguous(
                vec.as_mut_ptr().cast(),
                #[cfg(Py_3_11)]
                self.raw(),
                #[cfg(not(Py_3_11))]
                ptr::from_ref(self.raw()).cast_mut(),
                self.raw().len,
                fort as std::ffi::c_char,
            )
        })?;
        // set vector length to mark the now-initialized space as usable
        unsafe { vec.set_len(item_count) };
        Ok(vec)
    }

    /// Copies the specified slice into the buffer.
    /// If the buffer is multi-dimensional, the elements in the slice are expected to be in C-style order.
    ///
    ///  * Fails if the buffer is read-only.
    ///  * Fails if the slice does not have the correct length (`buf.item_count()`).
    ///  * Fails if the buffer format is not compatible with type `T`.
    ///
    /// To check whether the buffer format is compatible before calling this method,
    /// use `<T as buffer::Element>::is_compatible_format(buf.format())`.
    /// Alternatively, `match buffer::ElementType::from_format(buf.format())`.
    pub fn copy_from_slice(&self, py: Python<'_>, source: &[T]) -> PyResult<()> {
        self._copy_from_slice(py, source, b'C')
    }

    /// Copies the specified slice into the buffer.
    /// If the buffer is multi-dimensional, the elements in the slice are expected to be in Fortran-style order.
    ///
    ///  * Fails if the buffer is read-only.
    ///  * Fails if the slice does not have the correct length (`buf.item_count()`).
    ///  * Fails if the buffer format is not compatible with type `T`.
    ///
    /// To check whether the buffer format is compatible before calling this method,
    /// use `<T as buffer::Element>::is_compatible_format(buf.format())`.
    /// Alternatively, `match buffer::ElementType::from_format(buf.format())`.
    pub fn copy_from_fortran_slice(&self, py: Python<'_>, source: &[T]) -> PyResult<()> {
        self._copy_from_slice(py, source, b'F')
    }

    fn _copy_from_slice(&self, py: Python<'_>, source: &[T], fort: u8) -> PyResult<()> {
        if self.readonly() {
            return Err(PyBufferError::new_err("cannot write to read-only buffer"));
        } else if mem::size_of_val(source) != self.len_bytes() {
            return Err(PyBufferError::new_err(format!(
                "slice to copy from (of length {}) does not match buffer length of {}",
                source.len(),
                self.item_count()
            )));
        }

        err::error_on_minusone(py, unsafe {
            ffi::PyBuffer_FromContiguous(
                #[cfg(Py_3_11)]
                self.raw(),
                #[cfg(not(Py_3_11))]
                ptr::from_ref(self.raw()).cast_mut(),
                #[cfg(Py_3_11)]
                {
                    source.as_ptr().cast()
                },
                #[cfg(not(Py_3_11))]
                {
                    source.as_ptr().cast::<c_void>().cast_mut()
                },
                self.raw().len,
                fort as std::ffi::c_char,
            )
        })
    }
}

impl<T> std::ops::Deref for PyBuffer<T> {
    type Target = PyUntypedBuffer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PyUntypedBuffer {
    /// Gets the underlying buffer from the specified python object.
    pub fn get(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let buf = {
            let mut buf = Box::<RawBuffer>::new_uninit();
            // SAFETY: RawBuffer is `#[repr(transparent)]` around FFI struct
            err::error_on_minusone(obj.py(), unsafe {
                ffi::PyObject_GetBuffer(
                    obj.as_ptr(),
                    buf.as_mut_ptr().cast::<ffi::Py_buffer>(),
                    ffi::PyBUF_FULL_RO,
                )
            })?;
            // Safety: buf is initialized by PyObject_GetBuffer.
            unsafe { buf.assume_init() }
        };
        // Create PyBuffer immediately so that if validation checks fail, the PyBuffer::drop code
        // will call PyBuffer_Release (thus avoiding any leaks).
        let buf = Self(Pin::from(buf));
        let raw = buf.raw();

        if raw.shape.is_null() {
            Err(PyBufferError::new_err("shape is null"))
        } else if raw.strides.is_null() {
            Err(PyBufferError::new_err("strides is null"))
        } else {
            Ok(buf)
        }
    }

    /// Returns a `[PyBuffer]` instance if the buffer can be interpreted as containing elements of type `T`.
    pub fn into_typed<T: Element>(self) -> PyResult<PyBuffer<T>> {
        self.ensure_compatible_with::<T>()?;
        Ok(PyBuffer(self, PhantomData))
    }

    /// Non-owning equivalent of [`into_typed()`][Self::into_typed].
    pub fn as_typed<T: Element>(&self) -> PyResult<&PyBuffer<T>> {
        self.ensure_compatible_with::<T>()?;
        // SAFETY: PyBuffer<T> is repr(transparent) around PyUntypedBuffer
        Ok(unsafe { NonNull::from(self).cast::<PyBuffer<T>>().as_ref() })
    }

    fn ensure_compatible_with<T: Element>(&self) -> PyResult<()> {
        if mem::size_of::<T>() != self.item_size() || !T::is_compatible_format(self.format()) {
            Err(PyBufferError::new_err(format!(
                "buffer contents are not compatible with {}",
                std::any::type_name::<T>()
            )))
        } else if self.raw().buf.align_offset(mem::align_of::<T>()) != 0 {
            Err(PyBufferError::new_err(format!(
                "buffer contents are insufficiently aligned for {}",
                std::any::type_name::<T>()
            )))
        } else {
            Ok(())
        }
    }

    /// Releases the buffer object, freeing the reference to the Python object
    /// which owns the buffer.
    ///
    /// This will automatically be called on drop.
    pub fn release(self, _py: Python<'_>) {
        // First move self into a ManuallyDrop, so that PyBuffer::drop will
        // never be called. (It would attach to the interpreter and call PyBuffer_Release
        // again.)
        let mut mdself = mem::ManuallyDrop::new(self);
        unsafe {
            // Next, make the actual PyBuffer_Release call.
            // Fine to get a mutable reference to the inner ffi::Py_buffer here, as we're destroying it.
            mdself.0.release();

            // Finally, drop the contained Pin<Box<_>> in place, to free the
            // Box memory.
            ptr::drop_in_place::<Pin<Box<RawBuffer>>>(&mut mdself.0);
        }
    }

    /// Gets the pointer to the start of the buffer memory.
    ///
    /// Warning: the buffer memory can be mutated by other code (including
    /// other Python functions, if the GIL is released, or other extension
    /// modules even if the GIL is held). You must either access memory
    /// atomically, or ensure there are no data races yourself. See
    /// [this blog post] for more details.
    ///
    /// [this blog post]: https://alexgaynor.net/2022/oct/23/buffers-on-the-edge/
    #[inline]
    pub fn buf_ptr(&self) -> *mut c_void {
        self.raw().buf
    }

    /// Returns the Python object that owns the buffer data.
    ///
    /// This is the object that was passed to [`PyBuffer::get()`]
    /// when the buffer was created.
    /// Calling this before [`release()`][Self::release] and cloning the result
    /// allows you to keep the object alive after the buffer is released.
    #[inline]
    pub fn obj<'py>(&self, py: Python<'py>) -> Option<&Bound<'py, PyAny>> {
        unsafe { Bound::ref_from_ptr_or_opt(py, &self.raw().obj).as_ref() }
    }

    /// Gets a pointer to the specified item.
    ///
    /// If `indices.len() < self.dimensions()`, returns the start address of the sub-array at the specified dimension.
    pub fn get_ptr(&self, indices: &[usize]) -> *mut c_void {
        let shape = &self.shape()[..indices.len()];
        for i in 0..indices.len() {
            assert!(indices[i] < shape[i]);
        }
        unsafe {
            ffi::PyBuffer_GetPointer(
                #[cfg(Py_3_11)]
                self.raw(),
                #[cfg(not(Py_3_11))]
                ptr::from_ref(self.raw()).cast_mut(),
                #[cfg(Py_3_11)]
                indices.as_ptr().cast(),
                #[cfg(not(Py_3_11))]
                indices.as_ptr().cast_mut().cast(),
            )
        }
    }

    /// Gets whether the underlying buffer is read-only.
    #[inline]
    pub fn readonly(&self) -> bool {
        self.raw().readonly != 0
    }

    /// Gets the size of a single element, in bytes.
    /// Important exception: when requesting an unformatted buffer, item_size still has the value
    #[inline]
    pub fn item_size(&self) -> usize {
        self.raw().itemsize as usize
    }

    /// Gets the total number of items.
    #[inline]
    pub fn item_count(&self) -> usize {
        (self.raw().len as usize) / (self.raw().itemsize as usize)
    }

    /// `item_size() * item_count()`.
    /// For contiguous arrays, this is the length of the underlying memory block.
    /// For non-contiguous arrays, it is the length that the logical structure would have if it were copied to a contiguous representation.
    #[inline]
    pub fn len_bytes(&self) -> usize {
        self.raw().len as usize
    }

    /// Gets the number of dimensions.
    ///
    /// May be 0 to indicate a single scalar value.
    #[inline]
    pub fn dimensions(&self) -> usize {
        self.raw().ndim as usize
    }

    /// Returns an array of length `dimensions`. `shape()[i]` is the length of the array in dimension number `i`.
    ///
    /// May return None for single-dimensional arrays or scalar values (`dimensions() <= 1`);
    /// You can call `item_count()` to get the length of the single dimension.
    ///
    /// Despite Python using an array of signed integers, the values are guaranteed to be non-negative.
    /// However, dimensions of length 0 are possible and might need special attention.
    #[inline]
    pub fn shape(&self) -> &[usize] {
        unsafe { slice::from_raw_parts(self.raw().shape.cast(), self.raw().ndim as usize) }
    }

    /// Returns an array that holds, for each dimension, the number of bytes to skip to get to the next element in the dimension.
    ///
    /// Stride values can be any integer. For regular arrays, strides are usually positive,
    /// but a consumer MUST be able to handle the case `strides[n] <= 0`.
    #[inline]
    pub fn strides(&self) -> &[isize] {
        unsafe { slice::from_raw_parts(self.raw().strides, self.raw().ndim as usize) }
    }

    /// An array of length ndim.
    /// If `suboffsets[n] >= 0`, the values stored along the nth dimension are pointers and the suboffset value dictates how many bytes to add to each pointer after de-referencing.
    /// A suboffset value that is negative indicates that no de-referencing should occur (striding in a contiguous memory block).
    ///
    /// If all suboffsets are negative (i.e. no de-referencing is needed), then this field must be NULL (the default value).
    #[inline]
    pub fn suboffsets(&self) -> Option<&[isize]> {
        unsafe {
            if self.raw().suboffsets.is_null() {
                None
            } else {
                Some(slice::from_raw_parts(
                    self.raw().suboffsets,
                    self.raw().ndim as usize,
                ))
            }
        }
    }

    /// A string in struct module style syntax describing the contents of a single item.
    #[inline]
    pub fn format(&self) -> &CStr {
        if self.raw().format.is_null() {
            ffi::c_str!("B")
        } else {
            unsafe { CStr::from_ptr(self.raw().format) }
        }
    }

    /// Gets whether the buffer is contiguous in C-style order (last index varies fastest when visiting items in order of memory address).
    #[inline]
    pub fn is_c_contiguous(&self) -> bool {
        unsafe { ffi::PyBuffer_IsContiguous(self.raw(), b'C' as std::ffi::c_char) != 0 }
    }

    /// Gets whether the buffer is contiguous in Fortran-style order (first index varies fastest when visiting items in order of memory address).
    #[inline]
    pub fn is_fortran_contiguous(&self) -> bool {
        unsafe { ffi::PyBuffer_IsContiguous(self.raw(), b'F' as std::ffi::c_char) != 0 }
    }

    fn raw(&self) -> &ffi::Py_buffer {
        &self.0 .0
    }
}

impl RawBuffer {
    /// Release the contents of this pinned buffer.
    ///
    /// # Safety
    ///
    /// - The buffer must not be used after calling this function.
    /// - This function can only be called once.
    /// - Must be attached to the interpreter.
    ///
    unsafe fn release(self: &mut Pin<Box<Self>>) {
        unsafe {
            ffi::PyBuffer_Release(&mut Pin::get_unchecked_mut(self.as_mut()).0);
        }
    }
}

impl Drop for PyUntypedBuffer {
    fn drop(&mut self) {
        if Python::try_attach(|_| unsafe { self.0.release() }).is_none()
            && crate::internal::state::is_in_gc_traversal()
        {
            eprintln!("Warning: PyBuffer dropped while in GC traversal, this is a bug and will leak memory.");
        }
        // If `try_attach` failed and `is_in_gc_traversal()` is false, then probably the interpreter has
        // already finalized and we can just assume that the underlying memory has already been freed.
        //
        // So we don't handle that case here.
    }
}

/// Like [std::cell::Cell], but only provides read-only access to the data.
///
/// `&ReadOnlyCell<T>` is basically a safe version of `*const T`:
///  The data cannot be modified through the reference, but other references may
///  be modifying the data.
#[repr(transparent)]
pub struct ReadOnlyCell<T: Element>(cell::UnsafeCell<T>);

impl<T: Element> ReadOnlyCell<T> {
    /// Returns a copy of the current value.
    #[inline]
    pub fn get(&self) -> T {
        unsafe { *self.0.get() }
    }

    /// Returns a pointer to the current value.
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.0.get()
    }
}

macro_rules! impl_element(
    ($t:ty, $f:ident) => {
        unsafe impl Element for $t {
            fn is_compatible_format(format: &CStr) -> bool {
                let slice = format.to_bytes();
                if slice.len() > 1 && !is_matching_endian(slice[0]) {
                    return false;
                }
                ElementType::from_format(format) == ElementType::$f { bytes: mem::size_of::<$t>() }
            }
        }
    }
);

impl_element!(u8, UnsignedInteger);
impl_element!(u16, UnsignedInteger);
impl_element!(u32, UnsignedInteger);
impl_element!(u64, UnsignedInteger);
impl_element!(usize, UnsignedInteger);
impl_element!(i8, SignedInteger);
impl_element!(i16, SignedInteger);
impl_element!(i32, SignedInteger);
impl_element!(i64, SignedInteger);
impl_element!(isize, SignedInteger);
impl_element!(f32, Float);
impl_element!(f64, Float);

#[repr(u8)]
enum PyBufferContiguity {
    Undefined = 0,
    C = 1,
    F = 2,
    Any = 3,
}

const CONTIGUITY_UNDEFINED: u8 = PyBufferContiguity::Undefined as u8;
const CONTIGUITY_C: u8 = PyBufferContiguity::C as u8;
const CONTIGUITY_F: u8 = PyBufferContiguity::F as u8;
const CONTIGUITY_ANY: u8 = PyBufferContiguity::Any as u8;

/// Type-safe buffer request. The state parameter is intentionally hidden
/// behind this wrapper so the internal encoding can evolve.
///
/// The requested flags constrain what exporters are allowed to return. For example,
/// without shape information, only 1-dimensional buffers are permitted, and accessors
/// for unrequested metadata are unavailable on the typed view.
pub struct PyBufferRequest<
    Flags: PyBufferRequestType = RequestFlags<
        false,
        false,
        false,
        false,
        false,
        CONTIGUITY_UNDEFINED,
    >,
>(c_int, PhantomData<Flags>);

mod py_buffer_flags {
    pub struct PyBufferFlags<
        const FORMAT: bool,
        const SHAPE: bool,
        const STRIDE: bool,
        const INDIRECT: bool,
        const WRITABLE: bool,
        const CONTIGUITY: u8,
    >;

    pub trait Sealed {}
    impl<
            const FORMAT: bool,
            const SHAPE: bool,
            const STRIDE: bool,
            const INDIRECT: bool,
            const WRITABLE: bool,
            const CONTIGUITY: u8,
        > Sealed for PyBufferFlags<FORMAT, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>
    {
    }
}

use self::py_buffer_flags::PyBufferFlags as RequestFlags;

/// Trait implemented by all hidden [`PyBufferRequest`] states.
pub trait PyBufferRequestType: py_buffer_flags::Sealed {
    /// The contiguity requirement encoded by these flags.
    const CONTIGUITY: u8;

    /// Whether these flags require a writable buffer.
    const WRITABLE: bool;
}

impl<
        const FORMAT: bool,
        const SHAPE: bool,
        const STRIDE: bool,
        const INDIRECT: bool,
        const WRITABLE: bool,
        const CONTIGUITY_REQ: u8,
    > PyBufferRequestType
    for RequestFlags<FORMAT, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY_REQ>
{
    const CONTIGUITY: u8 = CONTIGUITY_REQ;
    const WRITABLE: bool = WRITABLE;
}

impl<
        const SHAPE: bool,
        const STRIDE: bool,
        const INDIRECT: bool,
        const WRITABLE: bool,
        const CONTIGUITY: u8,
    > PyBufferRequest<RequestFlags<false, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>>
{
    /// Request format information.
    pub const fn format(
        self,
    ) -> PyBufferRequest<RequestFlags<true, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>> {
        PyBufferRequest(self.0 | ffi::PyBUF_FORMAT, PhantomData)
    }
}

impl<
        const FORMAT: bool,
        const STRIDE: bool,
        const INDIRECT: bool,
        const WRITABLE: bool,
        const CONTIGUITY: u8,
    > PyBufferRequest<RequestFlags<FORMAT, false, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>>
{
    /// Request shape information.
    pub const fn nd(
        self,
    ) -> PyBufferRequest<RequestFlags<FORMAT, true, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>> {
        PyBufferRequest(self.0 | ffi::PyBUF_ND, PhantomData)
    }
}

impl<
        const FORMAT: bool,
        const SHAPE: bool,
        const INDIRECT: bool,
        const WRITABLE: bool,
        const CONTIGUITY: u8,
    > PyBufferRequest<RequestFlags<FORMAT, SHAPE, false, INDIRECT, WRITABLE, CONTIGUITY>>
{
    /// Request strides information. Implies shape.
    pub const fn strides(
        self,
    ) -> PyBufferRequest<RequestFlags<FORMAT, true, true, INDIRECT, WRITABLE, CONTIGUITY>> {
        PyBufferRequest(self.0 | ffi::PyBUF_STRIDES, PhantomData)
    }
}

impl<const FORMAT: bool, const SHAPE: bool, const STRIDE: bool, const WRITABLE: bool>
    PyBufferRequest<RequestFlags<FORMAT, SHAPE, STRIDE, false, WRITABLE, CONTIGUITY_UNDEFINED>>
{
    /// Request suboffsets (indirect). Implies shape and strides.
    pub const fn indirect(
        self,
    ) -> PyBufferRequest<RequestFlags<FORMAT, true, true, true, WRITABLE, CONTIGUITY_UNDEFINED>>
    {
        PyBufferRequest(self.0 | ffi::PyBUF_INDIRECT, PhantomData)
    }
}

impl<
        const FORMAT: bool,
        const SHAPE: bool,
        const STRIDE: bool,
        const INDIRECT: bool,
        const CONTIGUITY: u8,
    > PyBufferRequest<RequestFlags<FORMAT, SHAPE, STRIDE, INDIRECT, false, CONTIGUITY>>
{
    /// Request a writable buffer.
    pub const fn writable(
        self,
    ) -> PyBufferRequest<RequestFlags<FORMAT, SHAPE, STRIDE, INDIRECT, true, CONTIGUITY>> {
        PyBufferRequest(self.0 | ffi::PyBUF_WRITABLE, PhantomData)
    }
}

impl<
        const FORMAT: bool,
        const SHAPE: bool,
        const STRIDE: bool,
        const INDIRECT: bool,
        const WRITABLE: bool,
    >
    PyBufferRequest<RequestFlags<FORMAT, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY_UNDEFINED>>
{
    /// Require C-contiguous layout. Implies shape and strides.
    pub const fn c_contiguous(
        self,
    ) -> PyBufferRequest<RequestFlags<FORMAT, true, true, false, WRITABLE, CONTIGUITY_C>> {
        PyBufferRequest(self.0 | ffi::PyBUF_C_CONTIGUOUS, PhantomData)
    }

    /// Require Fortran-contiguous layout. Implies shape and strides.
    pub const fn f_contiguous(
        self,
    ) -> PyBufferRequest<RequestFlags<FORMAT, true, true, false, WRITABLE, CONTIGUITY_F>> {
        PyBufferRequest(self.0 | ffi::PyBUF_F_CONTIGUOUS, PhantomData)
    }

    /// Require contiguous layout (C or Fortran). Implies shape and strides.
    ///
    /// The specific contiguity order is not known at compile time,
    /// so this does not unlock non-Option slice accessors.
    pub const fn any_contiguous(
        self,
    ) -> PyBufferRequest<RequestFlags<FORMAT, true, true, false, WRITABLE, CONTIGUITY_ANY>> {
        PyBufferRequest(self.0 | ffi::PyBUF_ANY_CONTIGUOUS, PhantomData)
    }
}

impl PyBufferRequest {
    /// Create a base buffer request. Chain builder methods to add flags.
    pub const fn simple(
    ) -> PyBufferRequest<RequestFlags<false, false, false, false, false, CONTIGUITY_UNDEFINED>>
    {
        PyBufferRequest(ffi::PyBUF_SIMPLE, PhantomData)
    }

    /// Create a writable request for all buffer information including suboffsets.
    pub const fn full(
    ) -> PyBufferRequest<RequestFlags<true, true, true, true, true, CONTIGUITY_UNDEFINED>> {
        PyBufferRequest(ffi::PyBUF_FULL, PhantomData)
    }

    /// Create a read-only request for all buffer information including suboffsets.
    pub const fn full_ro(
    ) -> PyBufferRequest<RequestFlags<true, true, true, true, false, CONTIGUITY_UNDEFINED>> {
        PyBufferRequest(ffi::PyBUF_FULL_RO, PhantomData)
    }

    /// Create a writable request for format, shape, and strides.
    pub const fn records(
    ) -> PyBufferRequest<RequestFlags<true, true, true, false, true, CONTIGUITY_UNDEFINED>> {
        PyBufferRequest(ffi::PyBUF_RECORDS, PhantomData)
    }

    /// Create a read-only request for format, shape, and strides.
    pub const fn records_ro(
    ) -> PyBufferRequest<RequestFlags<true, true, true, false, false, CONTIGUITY_UNDEFINED>> {
        PyBufferRequest(ffi::PyBUF_RECORDS_RO, PhantomData)
    }

    /// Create a writable request for shape and strides.
    pub const fn strided(
    ) -> PyBufferRequest<RequestFlags<false, true, true, false, true, CONTIGUITY_UNDEFINED>> {
        PyBufferRequest(ffi::PyBUF_STRIDED, PhantomData)
    }

    /// Create a read-only request for shape and strides.
    pub const fn strided_ro(
    ) -> PyBufferRequest<RequestFlags<false, true, true, false, false, CONTIGUITY_UNDEFINED>> {
        PyBufferRequest(ffi::PyBUF_STRIDED_RO, PhantomData)
    }

    /// Create a writable C-contiguous request.
    pub const fn contig(
    ) -> PyBufferRequest<RequestFlags<false, true, false, false, true, CONTIGUITY_C>> {
        PyBufferRequest(ffi::PyBUF_CONTIG, PhantomData)
    }

    /// Create a read-only C-contiguous request.
    pub const fn contig_ro(
    ) -> PyBufferRequest<RequestFlags<false, true, false, false, false, CONTIGUITY_C>> {
        PyBufferRequest(ffi::PyBUF_CONTIG_RO, PhantomData)
    }
}

/// A typed form of [`PyUntypedBufferView`]. Not constructible directly — use
/// [`PyBufferView::with()`] or [`PyBufferView::with_flags()`].
#[repr(transparent)]
pub struct PyBufferView<
    T,
    Flags: PyBufferRequestType = RequestFlags<true, true, true, true, false, CONTIGUITY_UNDEFINED>,
>(PyUntypedBufferView<Flags>, PhantomData<[T]>);

/// Stack-allocated untyped buffer view.
///
/// Unlike [`PyUntypedBuffer`] which heap-allocates, this places the `Py_buffer` on the
/// stack. The scoped closure API ensures the buffer cannot be moved.
///
/// Use [`with_flags()`](Self::with_flags) with a [`PyBufferRequest`] value to acquire a view.
/// The available accessors depend on the flags used.
pub struct PyUntypedBufferView<
    Flags: PyBufferRequestType = RequestFlags<
        false,
        false,
        false,
        false,
        false,
        CONTIGUITY_UNDEFINED,
    >,
> {
    raw: ffi::Py_buffer,
    _flags: PhantomData<Flags>,
}

impl<Flags: PyBufferRequestType> PyUntypedBufferView<Flags> {
    /// Gets the pointer to the start of the buffer memory.
    #[inline]
    pub fn buf_ptr(&self) -> *mut c_void {
        self.raw.buf
    }

    /// Returns the Python object that owns the buffer data.
    #[inline]
    pub fn obj<'py>(&self, py: Python<'py>) -> Option<&Bound<'py, PyAny>> {
        unsafe { Bound::ref_from_ptr_or_opt(py, &self.raw.obj).as_ref() }
    }

    /// Gets whether the underlying buffer is read-only.
    #[inline]
    pub fn readonly(&self) -> bool {
        !Flags::WRITABLE && self.raw.readonly != 0
    }

    /// Gets the size of a single element, in bytes.
    #[inline]
    pub fn item_size(&self) -> usize {
        self.raw.itemsize as usize
    }

    /// Gets the total number of items.
    #[inline]
    pub fn item_count(&self) -> usize {
        (self.raw.len as usize) / (self.raw.itemsize as usize)
    }

    /// `item_size() * item_count()`.
    /// For contiguous arrays, this is the length of the underlying memory block.
    #[inline]
    pub fn len_bytes(&self) -> usize {
        self.raw.len as usize
    }

    /// Gets the number of dimensions.
    ///
    /// May be 0 to indicate a single scalar value.
    #[inline]
    pub fn dimensions(&self) -> usize {
        self.raw.ndim as usize
    }

    /// Gets whether the buffer is contiguous in C-style order.
    #[inline]
    pub fn is_c_contiguous(&self) -> bool {
        Flags::CONTIGUITY == CONTIGUITY_C
            || unsafe { ffi::PyBuffer_IsContiguous(&self.raw, b'C' as std::ffi::c_char) != 0 }
    }

    /// Gets whether the buffer is contiguous in Fortran-style order.
    #[inline]
    pub fn is_fortran_contiguous(&self) -> bool {
        Flags::CONTIGUITY == CONTIGUITY_F
            || unsafe { ffi::PyBuffer_IsContiguous(&self.raw, b'F' as std::ffi::c_char) != 0 }
    }
}

impl<
        const SHAPE: bool,
        const STRIDE: bool,
        const INDIRECT: bool,
        const WRITABLE: bool,
        const CONTIGUITY: u8,
    > PyUntypedBufferView<RequestFlags<true, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>>
{
    /// A [struct module style](https://docs.python.org/3/c-api/buffer.html#c.Py_buffer.format)
    /// string describing the contents of a single item.
    #[inline]
    pub fn format(&self) -> &CStr {
        debug_assert!(!self.raw.format.is_null());
        unsafe { CStr::from_ptr(self.raw.format) }
    }

    /// Attempt to interpret this untyped view as containing elements of type `T`.
    pub fn as_typed<T: Element>(
        &self,
    ) -> PyResult<&PyBufferView<T, RequestFlags<true, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>>>
    {
        self.ensure_compatible_with::<T>()?;
        // SAFETY: PyBufferView<T, ..> is repr(transparent) around PyUntypedBufferView<..>
        Ok(unsafe {
            NonNull::from(self)
                .cast::<PyBufferView<
                    T,
                    RequestFlags<true, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>,
                >>()
                .as_ref()
        })
    }

    fn ensure_compatible_with<T: Element>(&self) -> PyResult<()> {
        check_buffer_compatibility::<T>(self.raw.buf, self.item_size(), self.format())
    }
}

impl<
        const FORMAT: bool,
        const STRIDE: bool,
        const INDIRECT: bool,
        const WRITABLE: bool,
        const CONTIGUITY: u8,
    > PyUntypedBufferView<RequestFlags<FORMAT, true, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>>
{
    /// Returns the shape array. `shape[i]` is the length of dimension `i`.
    ///
    /// Despite Python using an array of signed integers, the values are guaranteed to be
    /// non-negative. However, dimensions of length 0 are possible and might need special
    /// attention.
    #[inline]
    pub fn shape(&self) -> &[usize] {
        debug_assert!(!self.raw.shape.is_null());
        unsafe { slice::from_raw_parts(self.raw.shape.cast(), self.raw.ndim as usize) }
    }
}

impl<
        const FORMAT: bool,
        const SHAPE: bool,
        const INDIRECT: bool,
        const WRITABLE: bool,
        const CONTIGUITY: u8,
    > PyUntypedBufferView<RequestFlags<FORMAT, SHAPE, true, INDIRECT, WRITABLE, CONTIGUITY>>
{
    /// Returns the strides array.
    ///
    /// Stride values can be any integer. For regular arrays, strides are usually positive,
    /// but a consumer MUST be able to handle the case `strides[n] <= 0`.
    #[inline]
    pub fn strides(&self) -> &[isize] {
        debug_assert!(!self.raw.strides.is_null());
        unsafe { slice::from_raw_parts(self.raw.strides, self.raw.ndim as usize) }
    }
}

impl<
        const FORMAT: bool,
        const SHAPE: bool,
        const STRIDE: bool,
        const WRITABLE: bool,
        const CONTIGUITY: u8,
    > PyUntypedBufferView<RequestFlags<FORMAT, SHAPE, STRIDE, true, WRITABLE, CONTIGUITY>>
{
    /// Returns the suboffsets array.
    ///
    /// May return `None` even when suboffsets were requested if the exporter sets
    /// `suboffsets` to `NULL`.
    #[inline]
    pub fn suboffsets(&self) -> Option<&[isize]> {
        if self.raw.suboffsets.is_null() {
            return None;
        }

        Some(unsafe { slice::from_raw_parts(self.raw.suboffsets, self.raw.ndim as usize) })
    }
}

// SIMPLE and WRITABLE requests guarantee the implicit "B" format.
impl<const WRITABLE: bool>
    PyUntypedBufferView<RequestFlags<false, false, false, false, WRITABLE, CONTIGUITY_UNDEFINED>>
{
    /// Returns the format string for a simple byte buffer, which is always `"B"`.
    #[inline]
    pub fn format(&self) -> &CStr {
        ffi::c_str!("B")
    }
}

/// Check that a buffer is compatible with element type `T`.
fn check_buffer_compatibility<T: Element>(
    buf: *mut c_void,
    itemsize: usize,
    format: &CStr,
) -> PyResult<()> {
    let name = std::any::type_name::<T>();

    if mem::size_of::<T>() != itemsize || !T::is_compatible_format(format) {
        return Err(PyBufferError::new_err(format!(
            "buffer contents are not compatible with {name}"
        )));
    }

    if buf.align_offset(mem::align_of::<T>()) != 0 {
        return Err(PyBufferError::new_err(format!(
            "buffer contents are insufficiently aligned for {name}"
        )));
    }

    Ok(())
}

impl PyUntypedBufferView {
    /// Acquire a buffer view with the given flags,
    /// pass it to `f`, then release the buffer.
    ///
    /// Use [`PyBufferRequest::simple()`] or one of the compound-request constructors such as
    /// [`PyBufferRequest::full_ro()`] to acquire a view.
    ///
    /// The requested flags constrain what exporters may return. For example, without shape
    /// information only 1-dimensional buffers are permitted.
    pub fn with_flags<
        const FORMAT: bool,
        const SHAPE: bool,
        const STRIDE: bool,
        const INDIRECT: bool,
        const WRITABLE: bool,
        const CONTIGUITY: u8,
        R,
    >(
        obj: &Bound<'_, PyAny>,
        flags: PyBufferRequest<RequestFlags<FORMAT, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>>,
        f: impl FnOnce(
            &PyUntypedBufferView<
                RequestFlags<FORMAT, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>,
            >,
        ) -> R,
    ) -> PyResult<R> {
        let mut raw = mem::MaybeUninit::<ffi::Py_buffer>::uninit();

        err::error_on_minusone(obj.py(), unsafe {
            ffi::PyObject_GetBuffer(obj.as_ptr(), raw.as_mut_ptr(), flags.0)
        })?;

        let view = PyUntypedBufferView {
            raw: unsafe { raw.assume_init() },
            _flags: PhantomData,
        };

        Ok(f(&view))
    }
}

impl<Flags: PyBufferRequestType> Drop for PyUntypedBufferView<Flags> {
    fn drop(&mut self) {
        unsafe { ffi::PyBuffer_Release(&mut self.raw) }
    }
}

impl<Flags: PyBufferRequestType> Debug for PyUntypedBufferView<Flags> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_buffer("PyUntypedBufferView", &self.raw, f)
    }
}

impl<T: Element> PyBufferView<T> {
    /// Acquire a typed buffer view with `PyBufferRequest::full_ro()` flags,
    /// validating that the buffer format is compatible with `T`.
    pub fn with<R>(obj: &Bound<'_, PyAny>, f: impl FnOnce(&PyBufferView<T>) -> R) -> PyResult<R> {
        PyUntypedBufferView::with_flags(obj, PyBufferRequest::full_ro(), |view| {
            view.as_typed::<T>().map(f)
        })?
    }

    /// Acquire a typed buffer view with the given flags.
    ///
    /// [`ffi::PyBUF_FORMAT`] is implicitly added for type validation. As with
    /// [`PyUntypedBufferView::with_flags`], the requested flags also constrain what exporters
    /// may return.
    pub fn with_flags<
        const FORMAT: bool,
        const SHAPE: bool,
        const STRIDE: bool,
        const INDIRECT: bool,
        const WRITABLE: bool,
        const CONTIGUITY: u8,
        R,
    >(
        obj: &Bound<'_, PyAny>,
        flags: PyBufferRequest<RequestFlags<FORMAT, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>>,
        f: impl FnOnce(
            &PyBufferView<T, RequestFlags<true, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>>,
        ) -> R,
    ) -> PyResult<R> {
        let mut raw = mem::MaybeUninit::<ffi::Py_buffer>::uninit();

        err::error_on_minusone(obj.py(), unsafe {
            ffi::PyObject_GetBuffer(obj.as_ptr(), raw.as_mut_ptr(), flags.0 | ffi::PyBUF_FORMAT)
        })?;

        let view = PyUntypedBufferView::<
            RequestFlags<true, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>,
        > {
            raw: unsafe { raw.assume_init() },
            _flags: PhantomData,
        };

        view.as_typed::<T>().map(f)
    }
}

impl<T: Element, Flags: PyBufferRequestType> PyBufferView<T, Flags> {
    /// Gets the buffer memory as a slice.
    ///
    /// Returns `None` if the buffer is not C-contiguous.
    ///
    /// The returned slice uses type [`ReadOnlyCell<T>`] because it's theoretically possible
    /// for any call into the Python runtime to modify the values in the slice.
    pub fn as_slice<'a>(&'a self, _py: Python<'a>) -> Option<&'a [ReadOnlyCell<T>]> {
        if !self.is_c_contiguous() {
            return None;
        }

        Some(unsafe { slice::from_raw_parts(self.0.raw.buf.cast(), self.item_count()) })
    }

    /// Gets the buffer memory as a mutable slice.
    ///
    /// Returns `None` if the buffer is read-only or not C-contiguous.
    ///
    /// The returned slice uses type [`Cell<T>`](cell::Cell) because it's theoretically possible
    /// for any call into the Python runtime to modify the values in the slice.
    pub fn as_mut_slice<'a>(&'a self, _py: Python<'a>) -> Option<&'a [cell::Cell<T>]> {
        if self.readonly() || !self.is_c_contiguous() {
            return None;
        }

        Some(unsafe { slice::from_raw_parts(self.0.raw.buf.cast(), self.item_count()) })
    }
}

// C-contiguous guaranteed — no contiguity check needed.
impl<
        T: Element,
        const FORMAT: bool,
        const SHAPE: bool,
        const STRIDE: bool,
        const INDIRECT: bool,
        const WRITABLE: bool,
    > PyBufferView<T, RequestFlags<FORMAT, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY_C>>
{
    /// Gets the buffer memory as a slice. The buffer is guaranteed C-contiguous.
    pub fn as_contiguous_slice<'a>(&'a self, _py: Python<'a>) -> &'a [ReadOnlyCell<T>] {
        unsafe { slice::from_raw_parts(self.0.raw.buf.cast(), self.item_count()) }
    }
}

// C-contiguous + writable guaranteed — no checks needed.
impl<
        T: Element,
        const FORMAT: bool,
        const SHAPE: bool,
        const STRIDE: bool,
        const INDIRECT: bool,
    > PyBufferView<T, RequestFlags<FORMAT, SHAPE, STRIDE, INDIRECT, true, CONTIGUITY_C>>
{
    /// Gets the buffer memory as a mutable slice.
    /// The buffer is guaranteed C-contiguous and writable.
    pub fn as_contiguous_mut_slice<'a>(&'a self, _py: Python<'a>) -> &'a [cell::Cell<T>] {
        unsafe { slice::from_raw_parts(self.0.raw.buf.cast(), self.item_count()) }
    }
}

// Fortran-contiguous guaranteed.
impl<
        T: Element,
        const FORMAT: bool,
        const SHAPE: bool,
        const STRIDE: bool,
        const INDIRECT: bool,
        const WRITABLE: bool,
    > PyBufferView<T, RequestFlags<FORMAT, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY_F>>
{
    /// Gets the buffer memory as a slice. The buffer is guaranteed Fortran-contiguous.
    pub fn as_fortran_contiguous_slice<'a>(&'a self, _py: Python<'a>) -> &'a [ReadOnlyCell<T>] {
        unsafe { slice::from_raw_parts(self.0.raw.buf.cast(), self.item_count()) }
    }
}

// Fortran-contiguous + writable guaranteed.
impl<
        T: Element,
        const FORMAT: bool,
        const SHAPE: bool,
        const STRIDE: bool,
        const INDIRECT: bool,
    > PyBufferView<T, RequestFlags<FORMAT, SHAPE, STRIDE, INDIRECT, true, CONTIGUITY_F>>
{
    /// Gets the buffer memory as a mutable slice.
    /// The buffer is guaranteed Fortran-contiguous and writable.
    pub fn as_fortran_contiguous_mut_slice<'a>(&'a self, _py: Python<'a>) -> &'a [cell::Cell<T>] {
        unsafe { slice::from_raw_parts(self.0.raw.buf.cast(), self.item_count()) }
    }
}

impl<T, Flags: PyBufferRequestType> std::ops::Deref for PyBufferView<T, Flags> {
    type Target = PyUntypedBufferView<Flags>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, Flags: PyBufferRequestType> Debug for PyBufferView<T, Flags> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_buffer("PyBufferView", &self.0.raw, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ffi;
    use crate::types::any::PyAnyMethods;
    use crate::types::PyBytes;
    use crate::Python;

    #[test]
    fn test_debug() {
        Python::attach(|py| {
            let bytes = PyBytes::new(py, b"abcde");
            let buffer: PyBuffer<u8> = PyBuffer::get(&bytes).unwrap();
            let expected = format!(
                concat!(
                    "PyBuffer {{ buf: {:?}, obj: {:?}, ",
                    "len: 5, itemsize: 1, readonly: 1, ",
                    "ndim: 1, format: Some(\"B\"), shape: Some([5]), ",
                    "strides: Some([1]), suboffsets: None, internal: {:?} }}",
                ),
                buffer.raw().buf,
                buffer.raw().obj,
                buffer.raw().internal
            );
            let debug_repr = format!("{:?}", buffer);
            assert_eq!(debug_repr, expected);

            let untyped = PyUntypedBuffer::get(&bytes).unwrap();
            let expected = format!(
                concat!(
                    "PyUntypedBuffer {{ buf: {:?}, obj: {:?}, ",
                    "len: 5, itemsize: 1, readonly: 1, ",
                    "ndim: 1, format: Some(\"B\"), shape: Some([5]), ",
                    "strides: Some([1]), suboffsets: None, internal: {:?} }}",
                ),
                untyped.raw().buf,
                untyped.raw().obj,
                untyped.raw().internal
            );
            let debug_repr = format!("{:?}", untyped);
            assert_eq!(debug_repr, expected);
        });
    }

    #[test]
    fn test_element_type_from_format() {
        use super::ElementType::*;
        use std::mem::size_of;

        for (cstr, expected) in [
            // @ prefix goes to native_element_type_from_type_char
            (
                c"@b",
                SignedInteger {
                    bytes: size_of::<c_schar>(),
                },
            ),
            (
                c"@c",
                UnsignedInteger {
                    bytes: size_of::<c_char>(),
                },
            ),
            (
                c"@b",
                SignedInteger {
                    bytes: size_of::<c_schar>(),
                },
            ),
            (
                c"@B",
                UnsignedInteger {
                    bytes: size_of::<c_uchar>(),
                },
            ),
            (c"@?", Bool),
            (
                c"@h",
                SignedInteger {
                    bytes: size_of::<c_short>(),
                },
            ),
            (
                c"@H",
                UnsignedInteger {
                    bytes: size_of::<c_ushort>(),
                },
            ),
            (
                c"@i",
                SignedInteger {
                    bytes: size_of::<c_int>(),
                },
            ),
            (
                c"@I",
                UnsignedInteger {
                    bytes: size_of::<c_uint>(),
                },
            ),
            (
                c"@l",
                SignedInteger {
                    bytes: size_of::<c_long>(),
                },
            ),
            (
                c"@L",
                UnsignedInteger {
                    bytes: size_of::<c_ulong>(),
                },
            ),
            (
                c"@q",
                SignedInteger {
                    bytes: size_of::<c_longlong>(),
                },
            ),
            (
                c"@Q",
                UnsignedInteger {
                    bytes: size_of::<c_ulonglong>(),
                },
            ),
            (
                c"@n",
                SignedInteger {
                    bytes: size_of::<libc::ssize_t>(),
                },
            ),
            (
                c"@N",
                UnsignedInteger {
                    bytes: size_of::<libc::size_t>(),
                },
            ),
            (c"@e", Float { bytes: 2 }),
            (c"@f", Float { bytes: 4 }),
            (c"@d", Float { bytes: 8 }),
            (c"@z", Unknown),
            // = prefix goes to standard_element_type_from_type_char
            (c"=b", SignedInteger { bytes: 1 }),
            (c"=c", UnsignedInteger { bytes: 1 }),
            (c"=B", UnsignedInteger { bytes: 1 }),
            (c"=?", Bool),
            (c"=h", SignedInteger { bytes: 2 }),
            (c"=H", UnsignedInteger { bytes: 2 }),
            (c"=l", SignedInteger { bytes: 4 }),
            (c"=l", SignedInteger { bytes: 4 }),
            (c"=I", UnsignedInteger { bytes: 4 }),
            (c"=L", UnsignedInteger { bytes: 4 }),
            (c"=q", SignedInteger { bytes: 8 }),
            (c"=Q", UnsignedInteger { bytes: 8 }),
            (c"=e", Float { bytes: 2 }),
            (c"=f", Float { bytes: 4 }),
            (c"=d", Float { bytes: 8 }),
            (c"=z", Unknown),
            (c"=0", Unknown),
            // bare char (no prefix) goes to native_element_type_from_type_char
            (
                c"b",
                SignedInteger {
                    bytes: size_of::<c_schar>(),
                },
            ),
            (
                c"B",
                UnsignedInteger {
                    bytes: size_of::<c_uchar>(),
                },
            ),
            (c"?", Bool),
            (c"f", Float { bytes: 4 }),
            (c"d", Float { bytes: 8 }),
            (c"z", Unknown),
            // <, >, ! prefixes go to standard_element_type_from_type_char
            (c"<i", SignedInteger { bytes: 4 }),
            (c">H", UnsignedInteger { bytes: 2 }),
            (c"!q", SignedInteger { bytes: 8 }),
            // unknown prefix -> Unknown
            (c":b", Unknown),
        ] {
            assert_eq!(
                ElementType::from_format(cstr),
                expected,
                "element from format &Cstr: {cstr:?}",
            );
        }
    }

    #[test]
    fn test_compatible_size() {
        // for the cast in PyBuffer::shape()
        assert_eq!(
            std::mem::size_of::<ffi::Py_ssize_t>(),
            std::mem::size_of::<usize>()
        );
    }

    #[test]
    fn test_bytes_buffer() {
        Python::attach(|py| {
            let bytes = PyBytes::new(py, b"abcde");
            let buffer = PyBuffer::get(&bytes).unwrap();
            assert_eq!(buffer.dimensions(), 1);
            assert_eq!(buffer.item_count(), 5);
            assert_eq!(buffer.format().to_str().unwrap(), "B");
            assert_eq!(buffer.shape(), [5]);
            // single-dimensional buffer is always contiguous
            assert!(buffer.is_c_contiguous());
            assert!(buffer.is_fortran_contiguous());

            let slice = buffer.as_slice(py).unwrap();
            assert_eq!(slice.len(), 5);
            assert_eq!(slice[0].get(), b'a');
            assert_eq!(slice[2].get(), b'c');
            assert_eq!(unsafe { *slice[0].as_ptr() }, b'a');

            assert_eq!(unsafe { *(buffer.get_ptr(&[1]).cast::<u8>()) }, b'b');

            assert!(buffer.as_mut_slice(py).is_none());

            assert!(buffer.copy_to_slice(py, &mut [0u8]).is_err());
            let mut arr = [0; 5];
            buffer.copy_to_slice(py, &mut arr).unwrap();
            assert_eq!(arr, b"abcde" as &[u8]);

            assert!(buffer.copy_from_slice(py, &[0u8; 5]).is_err());
            assert_eq!(buffer.to_vec(py).unwrap(), b"abcde");
        });
    }

    #[test]
    fn test_array_buffer() {
        Python::attach(|py| {
            let array = py
                .import("array")
                .unwrap()
                .call_method("array", ("f", (1.0, 1.5, 2.0, 2.5)), None)
                .unwrap();
            let buffer = PyBuffer::get(&array).unwrap();
            assert_eq!(buffer.dimensions(), 1);
            assert_eq!(buffer.item_count(), 4);
            assert_eq!(buffer.format().to_str().unwrap(), "f");
            assert_eq!(buffer.shape(), [4]);

            // array creates a 1D contiguous buffer, so it's both C and F contiguous.  This would
            // be more interesting if we can come up with a 2D buffer but I think it would need a
            // third-party lib or a custom class.

            // C-contiguous fns
            let slice = buffer.as_slice(py).unwrap();
            assert_eq!(slice.len(), 4);
            assert_eq!(slice[0].get(), 1.0);
            assert_eq!(slice[3].get(), 2.5);

            let mut_slice = buffer.as_mut_slice(py).unwrap();
            assert_eq!(mut_slice.len(), 4);
            assert_eq!(mut_slice[0].get(), 1.0);
            mut_slice[3].set(2.75);
            assert_eq!(slice[3].get(), 2.75);

            buffer
                .copy_from_slice(py, &[10.0f32, 11.0, 12.0, 13.0])
                .unwrap();
            assert_eq!(slice[2].get(), 12.0);

            assert_eq!(buffer.to_vec(py).unwrap(), [10.0, 11.0, 12.0, 13.0]);

            // F-contiguous fns
            let buffer = PyBuffer::get(&array).unwrap();
            let slice = buffer.as_fortran_slice(py).unwrap();
            assert_eq!(slice.len(), 4);
            assert_eq!(slice[1].get(), 11.0);

            let mut_slice = buffer.as_fortran_mut_slice(py).unwrap();
            assert_eq!(mut_slice.len(), 4);
            assert_eq!(mut_slice[2].get(), 12.0);
            mut_slice[3].set(2.75);
            assert_eq!(slice[3].get(), 2.75);

            buffer
                .copy_from_fortran_slice(py, &[10.0f32, 11.0, 12.0, 13.0])
                .unwrap();
            assert_eq!(slice[2].get(), 12.0);

            assert_eq!(buffer.to_fortran_vec(py).unwrap(), [10.0, 11.0, 12.0, 13.0]);
        });
    }

    #[test]
    fn test_obj_getter() {
        Python::attach(|py| {
            let bytes = PyBytes::new(py, b"hello");
            let buf = PyUntypedBuffer::get(bytes.as_any()).unwrap();

            // obj() returns the same object that owns the buffer
            let owner = buf.obj(py).unwrap();
            assert!(owner.is_instance_of::<PyBytes>());
            assert!(owner.is(&bytes));

            // can keep the owner alive after releasing the buffer
            let owner_ref: crate::Py<PyAny> = owner.clone().unbind();
            buf.release(py);
            drop(bytes);
            // owner_ref still valid after buffer and original are dropped
            Python::attach(|py| {
                let rebound = owner_ref.bind(py);
                assert!(rebound.is_instance_of::<PyBytes>());
            });
        });
    }

    #[test]
    fn test_copy_to_fortran_slice() {
        Python::attach(|py| {
            let array = py
                .import("array")
                .unwrap()
                .call_method("array", ("f", (1.0, 1.5, 2.0, 2.5)), None)
                .unwrap();
            let buffer = PyBuffer::get(&array).unwrap();

            // wrong length
            assert!(buffer.copy_to_fortran_slice(py, &mut [0.0f32]).is_err());
            // correct length
            let mut arr = [0.0f32; 4];
            buffer.copy_to_fortran_slice(py, &mut arr).unwrap();
            assert_eq!(arr, [1.0, 1.5, 2.0, 2.5]);
        });
    }

    #[test]
    fn test_copy_from_slice_wrong_length() {
        Python::attach(|py| {
            let array = py
                .import("array")
                .unwrap()
                .call_method("array", ("f", (1.0, 1.5, 2.0, 2.5)), None)
                .unwrap();
            let buffer = PyBuffer::get(&array).unwrap();
            // writable buffer, but wrong length
            assert!(!buffer.readonly());
            assert!(buffer.copy_from_slice(py, &[0.0f32; 2]).is_err());
            assert!(buffer.copy_from_fortran_slice(py, &[0.0f32; 2]).is_err());
        });
    }

    #[test]
    fn test_untyped_buffer() {
        Python::attach(|py| {
            let bytes = PyBytes::new(py, b"abcde");
            let buffer = PyUntypedBuffer::get(&bytes).unwrap();
            assert_eq!(buffer.dimensions(), 1);
            assert_eq!(buffer.item_count(), 5);
            assert_eq!(buffer.format().to_str().unwrap(), "B");
            assert_eq!(buffer.shape(), [5]);
            assert!(!buffer.buf_ptr().is_null());
            assert_eq!(buffer.strides(), &[1]);
            assert_eq!(buffer.len_bytes(), 5);
            assert_eq!(buffer.item_size(), 1);
            assert!(buffer.readonly());
            assert!(buffer.suboffsets().is_none());

            assert!(format!("{:?}", buffer).starts_with("PyUntypedBuffer { buf: "));

            let typed: &PyBuffer<u8> = buffer.as_typed().unwrap();
            assert_eq!(typed.dimensions(), 1);
            assert_eq!(typed.item_count(), 5);
            assert_eq!(typed.format().to_str().unwrap(), "B");
            assert_eq!(typed.shape(), [5]);
        });
    }

    #[test]
    fn test_untyped_buffer_view() {
        Python::attach(|py| {
            let bytes = PyBytes::new(py, b"abcde");
            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::full_ro(), |view| {
                assert!(!view.buf_ptr().is_null());
                assert_eq!(view.len_bytes(), 5);
                assert_eq!(view.item_size(), 1);
                assert_eq!(view.item_count(), 5);
                assert!(view.readonly());
                assert_eq!(view.dimensions(), 1);
                // with_flags() uses PyBufferRequest::full_ro() — all Known, direct return types
                assert_eq!(view.format().to_str().unwrap(), "B");
                assert_eq!(view.shape(), [5]);
                assert_eq!(view.strides(), [1]);
                assert!(view.suboffsets().is_none());
                assert!(view.is_c_contiguous());
                assert!(view.is_fortran_contiguous());
                assert!(view.obj(py).unwrap().is(&bytes));
            })
            .unwrap();
        });
    }

    #[test]
    fn test_typed_buffer_view() {
        Python::attach(|py| {
            let bytes = PyBytes::new(py, b"abcde");
            PyBufferView::<u8>::with(&bytes, |view| {
                assert_eq!(view.dimensions(), 1);
                assert_eq!(view.item_count(), 5);
                // PyBufferView::with uses PyBufferRequest::full_ro() — all Known
                assert_eq!(view.format().to_str().unwrap(), "B");
                assert_eq!(view.shape(), [5]);
                assert!(view.suboffsets().is_none());

                let slice = view.as_slice(py).unwrap();
                assert_eq!(slice.len(), 5);
                assert_eq!(slice[0].get(), b'a');
                assert_eq!(slice[4].get(), b'e');

                // bytes are read-only
                assert!(view.as_mut_slice(py).is_none());
            })
            .unwrap();
        });
    }

    #[test]
    fn test_buffer_view_array() {
        Python::attach(|py| {
            let array = py
                .import("array")
                .unwrap()
                .call_method("array", ("f", (1.0, 1.5, 2.0, 2.5)), None)
                .unwrap();
            PyBufferView::<f32>::with(&array, |view| {
                assert_eq!(view.dimensions(), 1);
                assert_eq!(view.item_count(), 4);
                assert_eq!(view.format().to_str().unwrap(), "f");
                assert_eq!(view.shape(), [4]);

                let slice = view.as_slice(py).unwrap();
                assert_eq!(slice.len(), 4);
                assert_eq!(slice[0].get(), 1.0);
                assert_eq!(slice[3].get(), 2.5);

                // array.array is writable
                let mut_slice = view.as_mut_slice(py).unwrap();
                assert_eq!(mut_slice[0].get(), 1.0);
                mut_slice[3].set(2.75);
                assert_eq!(slice[3].get(), 2.75);
            })
            .unwrap();
        });
    }

    #[test]
    fn test_buffer_view_with_flags() {
        Python::attach(|py| {
            let bytes = PyBytes::new(py, b"abcde");

            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::simple(), |view| {
                assert_eq!(view.item_count(), 5);
                assert_eq!(view.len_bytes(), 5);
                assert!(view.readonly());
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::simple().nd(), |view| {
                assert_eq!(view.item_count(), 5);
                assert_eq!(view.shape(), [5]);
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::simple().strides(), |view| {
                assert_eq!(view.shape(), [5]);
                assert_eq!(view.strides(), [1]);
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::simple().indirect(), |view| {
                assert_eq!(view.shape(), [5]);
                assert_eq!(view.strides(), [1]);
                assert!(view.suboffsets().is_none());
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::simple().format(), |view| {
                assert_eq!(view.item_count(), 5);
                assert_eq!(view.format().to_str().unwrap(), "B");
            })
            .unwrap();
        });
    }

    #[test]
    fn test_typed_buffer_view_with_flags() {
        Python::attach(|py| {
            let array = py
                .import("array")
                .unwrap()
                .call_method("array", ("f", (1.0, 1.5, 2.0, 2.5)), None)
                .unwrap();

            PyBufferView::<f32>::with_flags(&array, PyBufferRequest::simple().nd(), |view| {
                assert_eq!(view.item_count(), 4);
                assert_eq!(view.format().to_str().unwrap(), "f");
                assert_eq!(view.shape(), [4]);

                let slice = view.as_slice(py).unwrap();
                assert_eq!(slice[0].get(), 1.0);
                assert_eq!(slice[3].get(), 2.5);

                let mut_slice = view.as_mut_slice(py).unwrap();
                mut_slice[0].set(9.0);
                assert_eq!(slice[0].get(), 9.0);
            })
            .unwrap();
        });
    }

    #[test]
    fn test_typed_buffer_view_with_flags_incompatible() {
        Python::attach(|py| {
            let bytes = PyBytes::new(py, b"abcde");
            let result =
                PyBufferView::<f32>::with_flags(&bytes, PyBufferRequest::simple().nd(), |_view| {});
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_c_contiguous_slice() {
        Python::attach(|py| {
            let array = py
                .import("array")
                .unwrap()
                .call_method("array", ("f", (1.0, 1.5, 2.0)), None)
                .unwrap();

            // C_CONTIGUOUS: guaranteed contiguous readonly access (no Option)
            PyBufferView::<f32>::with_flags(
                &array,
                PyBufferRequest::simple().c_contiguous(),
                |view| {
                    let slice = view.as_contiguous_slice(py);
                    assert_eq!(slice.len(), 3);
                    assert_eq!(slice[0].get(), 1.0);
                    assert_eq!(slice[2].get(), 2.0);
                },
            )
            .unwrap();

            // C_CONTIGUOUS | WRITABLE (via CONTIG combined with STRIDES-level):
            // no predefined constant, but we can use PyBufferView::with on a writable array
            // and the Option-based as_mut_slice still works
            PyBufferView::<f32>::with(&array, |view| {
                let mut_slice = view.as_mut_slice(py).unwrap();
                mut_slice[2].set(9.0);
                assert_eq!(view.as_slice(py).unwrap()[2].get(), 9.0);
            })
            .unwrap();
        });
    }

    #[test]
    fn test_buffer_view_error() {
        Python::attach(|py| {
            let list = crate::types::PyList::empty(py);
            let result =
                PyUntypedBufferView::with_flags(&list, PyBufferRequest::full_ro(), |_view| {});
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_flag_builders() {
        fn assert_direct<
            const FORMAT: bool,
            const SHAPE: bool,
            const STRIDE: bool,
            const WRITABLE: bool,
            const CONTIGUITY: u8,
        >(
            _: PyBufferRequest<RequestFlags<FORMAT, SHAPE, STRIDE, false, WRITABLE, CONTIGUITY>>,
        ) {
        }

        fn assert_indirect<
            const FORMAT: bool,
            const SHAPE: bool,
            const STRIDE: bool,
            const WRITABLE: bool,
            const CONTIGUITY: u8,
        >(
            _: PyBufferRequest<RequestFlags<FORMAT, SHAPE, STRIDE, true, WRITABLE, CONTIGUITY>>,
        ) {
        }

        assert_direct(PyBufferRequest::simple());
        assert_direct(PyBufferRequest::records_ro());
        assert_direct(PyBufferRequest::strided_ro());
        assert_direct(PyBufferRequest::contig_ro());
        assert_indirect(PyBufferRequest::simple().indirect());
        assert_indirect(PyBufferRequest::full_ro());
        assert_indirect(PyBufferRequest::full());
        assert_direct(PyBufferRequest::full_ro().c_contiguous());
        assert_direct(PyBufferRequest::full().c_contiguous());

        Python::attach(|py| {
            let bytes = PyBytes::new(py, b"abcde");
            let array = py
                .import("array")
                .unwrap()
                .call_method("array", ("f", (1.0, 1.5, 2.0, 2.5)), None)
                .unwrap();

            // Primitive builders
            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::simple(), |view| {
                assert_eq!(view.format().to_str().unwrap(), "B");
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::simple().format(), |view| {
                assert_eq!(view.format().to_str().unwrap(), "B");
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::simple().nd(), |view| {
                assert_eq!(view.shape(), [5]);
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::simple().strides(), |view| {
                assert_eq!(view.shape(), [5]);
                assert_eq!(view.strides(), [1]);
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::simple().indirect(), |view| {
                assert_eq!(view.shape(), [5]);
                assert_eq!(view.strides(), [1]);
                assert!(view.suboffsets().is_none());
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&array, PyBufferRequest::simple().writable(), |view| {
                assert_eq!(view.format().to_str().unwrap(), "B");
                assert!(!view.readonly());
            })
            .unwrap();

            PyUntypedBufferView::with_flags(
                &array,
                PyBufferRequest::simple().writable().nd(),
                |view| {
                    assert_eq!(view.shape(), [4]);
                    assert!(!view.readonly());
                },
            )
            .unwrap();

            // Chained primitive builders
            PyUntypedBufferView::with_flags(
                &bytes,
                PyBufferRequest::simple().nd().format(),
                |view| {
                    assert_eq!(view.shape(), [5]);
                    assert_eq!(view.format().to_str().unwrap(), "B");
                },
            )
            .unwrap();

            PyUntypedBufferView::with_flags(
                &bytes,
                PyBufferRequest::simple().strides().format(),
                |view| {
                    assert_eq!(view.shape(), [5]);
                    assert_eq!(view.strides(), [1]);
                    assert_eq!(view.format().to_str().unwrap(), "B");
                },
            )
            .unwrap();

            // Contiguity builders
            PyUntypedBufferView::with_flags(
                &bytes,
                PyBufferRequest::simple().c_contiguous(),
                |view| {
                    assert_eq!(view.shape(), [5]);
                    assert_eq!(view.strides(), [1]);
                },
            )
            .unwrap();

            PyUntypedBufferView::with_flags(
                &bytes,
                PyBufferRequest::simple().f_contiguous(),
                |view| {
                    assert_eq!(view.shape(), [5]);
                    assert_eq!(view.strides(), [1]);
                },
            )
            .unwrap();

            PyUntypedBufferView::with_flags(
                &bytes,
                PyBufferRequest::simple().any_contiguous(),
                |view| {
                    assert_eq!(view.shape(), [5]);
                    assert_eq!(view.strides(), [1]);
                },
            )
            .unwrap();

            // Compound requests (read-only)
            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::full_ro(), |view| {
                assert_eq!(view.format().to_str().unwrap(), "B");
                assert_eq!(view.shape(), [5]);
                assert_eq!(view.strides(), [1]);
                assert!(view.suboffsets().is_none());
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::records_ro(), |view| {
                assert_eq!(view.format().to_str().unwrap(), "B");
                assert_eq!(view.shape(), [5]);
                assert_eq!(view.strides(), [1]);
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::strided_ro(), |view| {
                assert_eq!(view.shape(), [5]);
                assert_eq!(view.strides(), [1]);
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::contig_ro(), |view| {
                assert_eq!(view.shape(), [5]);
                assert!(view.is_c_contiguous());
            })
            .unwrap();

            // Writable compound requests
            PyUntypedBufferView::with_flags(&array, PyBufferRequest::full(), |view| {
                assert_eq!(view.format().to_str().unwrap(), "f");
                assert_eq!(view.shape(), [4]);
                assert_eq!(view.strides(), [4]);
                assert!(view.suboffsets().is_none());
                assert!(!view.readonly());
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&array, PyBufferRequest::records(), |view| {
                assert_eq!(view.format().to_str().unwrap(), "f");
                assert_eq!(view.shape(), [4]);
                assert!(!view.readonly());
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&array, PyBufferRequest::strided(), |view| {
                assert_eq!(view.shape(), [4]);
                assert_eq!(view.strides(), [4]);
                assert!(!view.readonly());
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&array, PyBufferRequest::contig(), |view| {
                assert_eq!(view.shape(), [4]);
                assert!(!view.readonly());
                assert!(view.is_c_contiguous());
            })
            .unwrap();

            // Compound + contiguity
            PyUntypedBufferView::with_flags(
                &bytes,
                PyBufferRequest::full_ro().c_contiguous(),
                |view| {
                    assert_eq!(view.format().to_str().unwrap(), "B");
                    assert_eq!(view.shape(), [5]);
                    assert_eq!(view.strides(), [1]);
                },
            )
            .unwrap();

            PyUntypedBufferView::with_flags(
                &array,
                PyBufferRequest::full().c_contiguous(),
                |view| {
                    assert_eq!(view.format().to_str().unwrap(), "f");
                    assert!(!view.readonly());
                },
            )
            .unwrap();

            PyUntypedBufferView::with_flags(
                &bytes,
                PyBufferRequest::strided_ro().format(),
                |view| {
                    assert_eq!(view.format().to_str().unwrap(), "B");
                    assert_eq!(view.shape(), [5]);
                    assert_eq!(view.strides(), [1]);
                },
            )
            .unwrap();

            PyUntypedBufferView::with_flags(
                &bytes,
                PyBufferRequest::simple().c_contiguous().format(),
                |view| {
                    assert_eq!(view.format().to_str().unwrap(), "B");
                    assert_eq!(view.shape(), [5]);
                },
            )
            .unwrap();

            // Contiguity builder on typed view
            PyBufferView::<u8>::with_flags(&bytes, PyBufferRequest::simple().format(), |view| {
                assert_eq!(view.format().to_str().unwrap(), "B");
                assert_eq!(view.item_count(), 5);
            })
            .unwrap();

            PyBufferView::<f32>::with_flags(&array, PyBufferRequest::contig(), |view| {
                let slice = view.as_contiguous_slice(py);
                assert_eq!(slice[0].get(), 1.0);
            })
            .unwrap();

            // Writable + contiguity on typed view
            PyBufferView::<f32>::with_flags(&array, PyBufferRequest::contig(), |view| {
                let slice = view.as_contiguous_slice(py);
                assert_eq!(slice[0].get(), 1.0);
                let mut_slice = view.as_contiguous_mut_slice(py);
                mut_slice[0].set(9.0);
                assert_eq!(slice[0].get(), 9.0);
            })
            .unwrap();

            // SIMPLE format() returns "B"
            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::simple(), |view| {
                assert_eq!(view.format().to_str().unwrap(), "B");
            })
            .unwrap();
        });
    }

    #[test]
    fn test_buffer_view_debug() {
        Python::attach(|py| {
            let bytes = PyBytes::new(py, b"abcde");

            // Debug always uses raw_format/raw_shape/raw_strides (Option in output)
            PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::full_ro(), |view| {
                let expected = format!(
                    concat!(
                        "PyUntypedBufferView {{ buf: {:?}, obj: {:?}, ",
                        "len: 5, itemsize: 1, readonly: 1, ",
                        "ndim: 1, format: Some(\"B\"), shape: Some([5]), ",
                        "strides: Some([1]), suboffsets: None, internal: {:?} }}",
                    ),
                    view.raw.buf, view.raw.obj, view.raw.internal,
                );

                let debug_repr = format!("{:?}", view);
                assert_eq!(debug_repr, expected);
            })
            .unwrap();

            PyBufferView::<u8>::with(&bytes, |view| {
                let expected = format!(
                    concat!(
                        "PyBufferView {{ buf: {:?}, obj: {:?}, ",
                        "len: 5, itemsize: 1, readonly: 1, ",
                        "ndim: 1, format: Some(\"B\"), shape: Some([5]), ",
                        "strides: Some([1]), suboffsets: None, internal: {:?} }}",
                    ),
                    view.0.raw.buf, view.0.raw.obj, view.0.raw.internal,
                );

                let debug_repr = format!("{:?}", view);
                assert_eq!(debug_repr, expected);
            })
            .unwrap();
        });
    }
}
