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
use crate::{err, exceptions::PyBufferError, ffi, FromPyObject, PyAny, PyResult, Python};
use crate::{Borrowed, Bound, PyErr};
use std::ffi::{
    c_char, c_int, c_long, c_longlong, c_schar, c_short, c_uchar, c_uint, c_ulong, c_ulonglong,
    c_ushort, c_void,
};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::{cell, mem, slice};
use std::{ffi::CStr, fmt::Debug};

/// A typed form of [`PyUntypedBuffer`].
#[repr(transparent)]
pub struct PyBuffer<T>(PyUntypedBuffer, PhantomData<[T]>);

/// Allows access to the underlying buffer used by a python object such as `bytes`, `bytearray` or `array.array`.
#[repr(transparent)]
pub struct PyUntypedBuffer(Box<ffi::Py_buffer>);

// PyBuffer send & sync guarantees are upheld by Python.
unsafe impl Send for PyUntypedBuffer {}
unsafe impl Sync for PyUntypedBuffer {}

impl<T> Debug for PyBuffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_buffer("PyBuffer", &self.0, f)
    }
}

impl Debug for PyUntypedBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_buffer("PyUntypedBuffer", self, f)
    }
}

fn debug_buffer(
    name: &str,
    b: &PyUntypedBuffer,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let raw = &b.0;
    f.debug_struct(name)
        .field("buf", &raw.buf)
        .field("obj", &raw.obj)
        .field("len", &raw.len)
        .field("itemsize", &raw.itemsize)
        .field("readonly", &raw.readonly)
        .field("ndim", &raw.ndim)
        .field("format", &b.format())
        .field("shape", &b.shape())
        .field("strides", &b.strides())
        .field("suboffsets", &b.suboffsets())
        .field("internal", &raw.internal)
        .finish()
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

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<PyBuffer<T>, Self::Error> {
        Self::get(&obj)
    }
}

impl<T: Element> PyBuffer<T> {
    /// Gets the underlying buffer from the specified python object.
    pub fn get(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        PyUntypedBuffer::get(obj)?.into_typed()
    }

    /// Releases the buffer object, freeing the reference to the Python object
    /// which owns the buffer.
    ///
    /// This will automatically be called on drop.
    #[inline]
    pub fn release(self, py: Python<'_>) {
        self.0.release(py)
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
        self.0.buf_ptr()
    }

    /// Gets a pointer to the specified item.
    ///
    /// If `indices.len() < self.dimensions()`, returns the start address of the sub-array at the specified dimension.
    #[inline]
    pub fn get_ptr(&self, indices: &[usize]) -> *mut c_void {
        self.0.get_ptr(indices)
    }

    /// Gets whether the underlying buffer is read-only.
    #[inline]
    pub fn readonly(&self) -> bool {
        self.0.readonly()
    }

    /// Gets the size of a single element, in bytes.
    /// Important exception: when requesting an unformatted buffer, item_size still has the value
    #[inline]
    pub fn item_size(&self) -> usize {
        self.0.item_size()
    }

    /// Gets the total number of items.
    #[inline]
    pub fn item_count(&self) -> usize {
        self.0.item_count()
    }

    /// `item_size() * item_count()`.
    /// For contiguous arrays, this is the length of the underlying memory block.
    /// For non-contiguous arrays, it is the length that the logical structure would have if it were copied to a contiguous representation.
    #[inline]
    pub fn len_bytes(&self) -> usize {
        self.0.len_bytes()
    }

    /// Gets the number of dimensions.
    ///
    /// May be 0 to indicate a single scalar value.
    #[inline]
    pub fn dimensions(&self) -> usize {
        self.0.dimensions()
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
        self.0.shape()
    }

    /// Returns an array that holds, for each dimension, the number of bytes to skip to get to the next element in the dimension.
    ///
    /// Stride values can be any integer. For regular arrays, strides are usually positive,
    /// but a consumer MUST be able to handle the case `strides[n] <= 0`.
    #[inline]
    pub fn strides(&self) -> &[isize] {
        self.0.strides()
    }

    /// An array of length ndim.
    /// If `suboffsets[n] >= 0`, the values stored along the nth dimension are pointers and the suboffset value dictates how many bytes to add to each pointer after de-referencing.
    /// A suboffset value that is negative indicates that no de-referencing should occur (striding in a contiguous memory block).
    ///
    /// If all suboffsets are negative (i.e. no de-referencing is needed), then this field must be NULL (the default value).
    #[inline]
    pub fn suboffsets(&self) -> Option<&[isize]> {
        self.0.suboffsets()
    }

    /// A string in struct module style syntax describing the contents of a single item.
    #[inline]
    pub fn format(&self) -> &CStr {
        self.0.format()
    }

    /// Gets whether the buffer is contiguous in C-style order (last index varies fastest when visiting items in order of memory address).
    #[inline]
    pub fn is_c_contiguous(&self) -> bool {
        self.0.is_c_contiguous()
    }

    /// Gets whether the buffer is contiguous in Fortran-style order (first index varies fastest when visiting items in order of memory address).
    #[inline]
    pub fn is_fortran_contiguous(&self) -> bool {
        self.0.is_fortran_contiguous()
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
                    self.0 .0.buf as *mut ReadOnlyCell<T>,
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
                    self.0 .0.buf as *mut cell::Cell<T>,
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
                    self.0 .0.buf as *mut ReadOnlyCell<T>,
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
                    self.0 .0.buf as *mut cell::Cell<T>,
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
                &*self.0 .0,
                #[cfg(not(Py_3_11))]
                {
                    &*self.0 as *const ffi::Py_buffer as *mut ffi::Py_buffer
                },
                self.0 .0.len,
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
                vec.as_ptr() as *mut c_void,
                #[cfg(Py_3_11)]
                &*self.0 .0,
                #[cfg(not(Py_3_11))]
                {
                    &*self.0 as *const ffi::Py_buffer as *mut ffi::Py_buffer
                },
                self.0 .0.len,
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
                &*self.0 .0,
                #[cfg(not(Py_3_11))]
                {
                    &*self.0 as *const ffi::Py_buffer as *mut ffi::Py_buffer
                },
                #[cfg(Py_3_11)]
                {
                    source.as_ptr().cast()
                },
                #[cfg(not(Py_3_11))]
                {
                    source.as_ptr() as *mut c_void
                },
                self.0 .0.len,
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
    /// See [`PyBuffer::get()`].
    pub fn get(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let mut buf = mem::MaybeUninit::uninit();
        let buf: Box<ffi::Py_buffer> = {
            err::error_on_minusone(obj.py(), unsafe {
                // TODO: add PyBufferRequest type which allows for controlling the buffer request?
                // - require writable ?
                // - require contiguous ?
                // - is there ever a case that we need to handle producers which can't fill strides or shape?
                ffi::PyObject_GetBuffer(obj.as_ptr(), buf.as_mut_ptr(), ffi::PyBUF_FULL_RO)
            })?;
            // SAFETY: PyObject_GetBuffer has initialized the buffer on success
            Box::new(unsafe { buf.assume_init() })
        };
        // Create PyBuffer immediately so that if validation checks fail, the PyBuffer::drop code
        // will call PyBuffer_Release (thus avoiding any leaks).
        let buf = Self(buf);
        let raw = &buf.0;

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
        } else if self.0.buf.align_offset(mem::align_of::<T>()) != 0 {
            Err(PyBufferError::new_err(format!(
                "buffer contents are insufficiently aligned for {}",
                std::any::type_name::<T>()
            )))
        } else {
            Ok(())
        }
    }

    /// See [`PyBuffer::release()`].
    pub fn release(self, _py: Python<'_>) {
        // SAFETY: Self is `repr(transparent)` around a Box<ffi::Py_buffer>
        let mut inner: Box<ffi::Py_buffer> = unsafe { std::mem::transmute(self) };
        // SAFETY: the ffi::Py_buffer structure is valid until this release call
        unsafe { ffi::PyBuffer_Release(&mut *inner) };
    }

    /// See [`PyBuffer::buf_ptr()`].
    #[inline]
    pub fn buf_ptr(&self) -> *mut c_void {
        self.0.buf
    }

    /// See [`PyBuffer::get_ptr()`].
    pub fn get_ptr(&self, indices: &[usize]) -> *mut c_void {
        let shape = &self.shape()[..indices.len()];
        for i in 0..indices.len() {
            assert!(indices[i] < shape[i]);
        }
        unsafe {
            ffi::PyBuffer_GetPointer(
                #[cfg(Py_3_11)]
                &*self.0,
                #[cfg(not(Py_3_11))]
                {
                    &*self.0 as *const ffi::Py_buffer as *mut ffi::Py_buffer
                },
                #[cfg(Py_3_11)]
                {
                    indices.as_ptr().cast()
                },
                #[cfg(not(Py_3_11))]
                {
                    indices.as_ptr() as *mut ffi::Py_ssize_t
                },
            )
        }
    }

    /// See [`PyBuffer::readonly()`].
    #[inline]
    pub fn readonly(&self) -> bool {
        self.0.readonly != 0
    }

    /// See [`PyBuffer::item_size()`].
    #[inline]
    pub fn item_size(&self) -> usize {
        self.0.itemsize as usize
    }

    /// See [`PyBuffer::item_count()`].
    #[inline]
    pub fn item_count(&self) -> usize {
        (self.0.len as usize) / (self.0.itemsize as usize)
    }

    /// See [`PyBuffer::len_bytes()`].
    #[inline]
    pub fn len_bytes(&self) -> usize {
        self.0.len as usize
    }

    /// See [`PyBuffer::dimensions()`].
    #[inline]
    pub fn dimensions(&self) -> usize {
        self.0.ndim as usize
    }

    /// See [`PyBuffer::shape()`].
    #[inline]
    pub fn shape(&self) -> &[usize] {
        unsafe { slice::from_raw_parts(self.0.shape.cast(), self.0.ndim as usize) }
    }

    /// See [`PyBuffer::strides()`].
    #[inline]
    pub fn strides(&self) -> &[isize] {
        unsafe { slice::from_raw_parts(self.0.strides, self.0.ndim as usize) }
    }

    /// See [`PyBuffer::suboffsets()`].
    #[inline]
    pub fn suboffsets(&self) -> Option<&[isize]> {
        unsafe {
            if self.0.suboffsets.is_null() {
                None
            } else {
                Some(slice::from_raw_parts(
                    self.0.suboffsets,
                    self.0.ndim as usize,
                ))
            }
        }
    }

    /// See [`PyBuffer::format()`].
    #[inline]
    pub fn format(&self) -> &CStr {
        if self.0.format.is_null() {
            ffi::c_str!("B")
        } else {
            unsafe { CStr::from_ptr(self.0.format) }
        }
    }

    /// See [`PyBuffer::is_c_contiguous()`].
    #[inline]
    pub fn is_c_contiguous(&self) -> bool {
        unsafe { ffi::PyBuffer_IsContiguous(&*self.0, b'C' as std::ffi::c_char) != 0 }
    }

    /// See [`PyBuffer::is_fortran_contiguous()`].
    #[inline]
    pub fn is_fortran_contiguous(&self) -> bool {
        unsafe { ffi::PyBuffer_IsContiguous(&*self.0, b'F' as std::ffi::c_char) != 0 }
    }
}

impl Drop for PyUntypedBuffer {
    fn drop(&mut self) {
        fn inner(buf: &mut Box<ffi::Py_buffer>) {
            if Python::try_attach(|_| unsafe { ffi::PyBuffer_Release(buf.as_mut()) }).is_none()
                && crate::internal::state::is_in_gc_traversal()
            {
                eprintln!("Warning: PyBuffer dropped while in GC traversal, this is a bug and will leak memory.");
            }
            // If `try_attach` failed and `is_in_gc_traversal()` is false, then probably the interpreter has
            // already finalized and we can just assume that the underlying memory has already been freed.
            //
            // So we don't handle that case here.
        }

        inner(&mut self.0);
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
                    "ndim: 1, format: \"B\", shape: [5], ",
                    "strides: [1], suboffsets: None, internal: {:?} }}",
                ),
                buffer.buf_ptr(),
                buffer.0 .0.obj,
                buffer.0 .0.internal
            );
            let debug_repr = format!("{:?}", buffer);
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
                ffi::c_str!("@b"),
                SignedInteger {
                    bytes: size_of::<c_schar>(),
                },
            ),
            (
                ffi::c_str!("@c"),
                UnsignedInteger {
                    bytes: size_of::<c_char>(),
                },
            ),
            (
                ffi::c_str!("@b"),
                SignedInteger {
                    bytes: size_of::<c_schar>(),
                },
            ),
            (
                ffi::c_str!("@B"),
                UnsignedInteger {
                    bytes: size_of::<c_uchar>(),
                },
            ),
            (ffi::c_str!("@?"), Bool),
            (
                ffi::c_str!("@h"),
                SignedInteger {
                    bytes: size_of::<c_short>(),
                },
            ),
            (
                ffi::c_str!("@H"),
                UnsignedInteger {
                    bytes: size_of::<c_ushort>(),
                },
            ),
            (
                ffi::c_str!("@i"),
                SignedInteger {
                    bytes: size_of::<c_int>(),
                },
            ),
            (
                ffi::c_str!("@I"),
                UnsignedInteger {
                    bytes: size_of::<c_uint>(),
                },
            ),
            (
                ffi::c_str!("@l"),
                SignedInteger {
                    bytes: size_of::<c_long>(),
                },
            ),
            (
                ffi::c_str!("@L"),
                UnsignedInteger {
                    bytes: size_of::<c_ulong>(),
                },
            ),
            (
                ffi::c_str!("@q"),
                SignedInteger {
                    bytes: size_of::<c_longlong>(),
                },
            ),
            (
                ffi::c_str!("@Q"),
                UnsignedInteger {
                    bytes: size_of::<c_ulonglong>(),
                },
            ),
            (
                ffi::c_str!("@n"),
                SignedInteger {
                    bytes: size_of::<libc::ssize_t>(),
                },
            ),
            (
                ffi::c_str!("@N"),
                UnsignedInteger {
                    bytes: size_of::<libc::size_t>(),
                },
            ),
            (ffi::c_str!("@e"), Float { bytes: 2 }),
            (ffi::c_str!("@f"), Float { bytes: 4 }),
            (ffi::c_str!("@d"), Float { bytes: 8 }),
            (ffi::c_str!("@z"), Unknown),
            // = prefix goes to standard_element_type_from_type_char
            (ffi::c_str!("=b"), SignedInteger { bytes: 1 }),
            (ffi::c_str!("=c"), UnsignedInteger { bytes: 1 }),
            (ffi::c_str!("=B"), UnsignedInteger { bytes: 1 }),
            (ffi::c_str!("=?"), Bool),
            (ffi::c_str!("=h"), SignedInteger { bytes: 2 }),
            (ffi::c_str!("=H"), UnsignedInteger { bytes: 2 }),
            (ffi::c_str!("=l"), SignedInteger { bytes: 4 }),
            (ffi::c_str!("=l"), SignedInteger { bytes: 4 }),
            (ffi::c_str!("=I"), UnsignedInteger { bytes: 4 }),
            (ffi::c_str!("=L"), UnsignedInteger { bytes: 4 }),
            (ffi::c_str!("=q"), SignedInteger { bytes: 8 }),
            (ffi::c_str!("=Q"), UnsignedInteger { bytes: 8 }),
            (ffi::c_str!("=e"), Float { bytes: 2 }),
            (ffi::c_str!("=f"), Float { bytes: 4 }),
            (ffi::c_str!("=d"), Float { bytes: 8 }),
            (ffi::c_str!("=z"), Unknown),
            (ffi::c_str!("=0"), Unknown),
            // unknown prefix -> Unknown
            (ffi::c_str!(":b"), Unknown),
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

            assert_eq!(unsafe { *(buffer.get_ptr(&[1]) as *mut u8) }, b'b');

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
    fn test_untyped_buffer() {
        Python::attach(|py| {
            let bytes = PyBytes::new(py, b"abcde");
            let untyped = PyUntypedBuffer::get(&bytes).unwrap();
            assert_eq!(untyped.dimensions(), 1);
            assert_eq!(untyped.item_count(), 5);
            assert_eq!(untyped.format().to_str().unwrap(), "B");
            assert_eq!(untyped.shape(), [5]);

            let typed: &PyBuffer<u8> = untyped.as_typed().unwrap();
            assert_eq!(typed.dimensions(), 1);
            assert_eq!(typed.item_count(), 5);
            assert_eq!(typed.format().to_str().unwrap(), "B");
            assert_eq!(typed.shape(), [5]);
        });
    }
}
