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
    let raw = b.raw();
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

/// Type-safe buffer request flags. The const parameters encode which fields
/// the exporter is required to fill.
pub struct PyBufferFlags<
    const FORMAT: bool = false,
    const SHAPE: bool = false,
    const STRIDE: bool = false,
    const WRITABLE: bool = false,
    const C_CONTIGUOUS: bool = false,
    const F_CONTIGUOUS: bool = false,
>(c_int);

mod py_buffer_flags_sealed {
    pub trait Sealed {}
    impl<
            const FORMAT: bool,
            const SHAPE: bool,
            const STRIDE: bool,
            const WRITABLE: bool,
            const C_CONTIGUOUS: bool,
            const F_CONTIGUOUS: bool,
        > Sealed for super::PyBufferFlags<FORMAT, SHAPE, STRIDE, WRITABLE, C_CONTIGUOUS, F_CONTIGUOUS>
    {
    }
}

/// Trait implemented by all [`PyBufferFlags`] instantiations.
pub trait PyBufferFlagsType: py_buffer_flags_sealed::Sealed {}

impl<
        const FORMAT: bool,
        const SHAPE: bool,
        const STRIDE: bool,
        const WRITABLE: bool,
        const C_CONTIGUOUS: bool,
        const F_CONTIGUOUS: bool,
    > PyBufferFlagsType for PyBufferFlags<FORMAT, SHAPE, STRIDE, WRITABLE, C_CONTIGUOUS, F_CONTIGUOUS>
{
}

// PyBufferFlags::FORMAT | <any non-format flags>
impl<const SHAPE: bool, const STRIDE: bool, const WRITABLE: bool, const CCONTIGUOUS: bool, const FCONTIGUOUS: bool>
    std::ops::BitOr<PyBufferFlags<false, SHAPE, STRIDE, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>>
    for PyBufferFlags<true, false, false>
{
    type Output = PyBufferFlags<true, SHAPE, STRIDE, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>;
    fn bitor(self, rhs: PyBufferFlags<false, SHAPE, STRIDE, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>) -> Self::Output {
        PyBufferFlags(self.0 | rhs.0)
    }
}

// <any non-format flags> | PyBufferFlags::FORMAT
impl<const SHAPE: bool, const STRIDE: bool, const WRITABLE: bool, const CCONTIGUOUS: bool, const FCONTIGUOUS: bool>
    std::ops::BitOr<PyBufferFlags<true, false, false>>
    for PyBufferFlags<false, SHAPE, STRIDE, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>
{
    type Output = PyBufferFlags<true, SHAPE, STRIDE, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>;
    fn bitor(self, rhs: PyBufferFlags<true, false, false>) -> Self::Output {
        PyBufferFlags(self.0 | rhs.0)
    }
}

#[allow(non_upper_case_globals)]
impl PyBufferFlags {
    /// Request a simple buffer with no shape, strides, or format information.
    pub const SIMPLE: PyBufferFlags = PyBufferFlags(ffi::PyBUF_SIMPLE);
    /// Request format information only.
    pub const FORMAT: PyBufferFlags<true> = PyBufferFlags(ffi::PyBUF_FORMAT);
    /// Request shape information.
    pub const ND: PyBufferFlags<false, true> = PyBufferFlags(ffi::PyBUF_ND);
    /// Request shape and strides.
    pub const STRIDES: PyBufferFlags<false, true, true> = PyBufferFlags(ffi::PyBUF_STRIDES);
    /// Request C-contiguous buffer with shape and strides.
    pub const C_CONTIGUOUS: PyBufferFlags<false, true, true, false, true, false> =
        PyBufferFlags(ffi::PyBUF_C_CONTIGUOUS);
    /// Request Fortran-contiguous buffer with shape and strides.
    pub const F_CONTIGUOUS: PyBufferFlags<false, true, true, false, false, true> =
        PyBufferFlags(ffi::PyBUF_F_CONTIGUOUS);
    /// Request contiguous buffer (C or Fortran) with shape and strides.
    pub const ANY_CONTIGUOUS: PyBufferFlags<false, true, true> =
        PyBufferFlags(ffi::PyBUF_ANY_CONTIGUOUS);
    /// Request shape, strides, and suboffsets.
    pub const INDIRECT: PyBufferFlags<false, true, true> = PyBufferFlags(ffi::PyBUF_INDIRECT);
    /// Request writable buffer with shape.
    pub const CONTIG: PyBufferFlags<false, true, false, true> = PyBufferFlags(ffi::PyBUF_CONTIG);
    /// Request shape (read-only, equivalent to [`Self::ND`]).
    pub const CONTIG_RO: PyBufferFlags<false, true> = PyBufferFlags(ffi::PyBUF_CONTIG_RO);
    /// Request writable buffer with shape and strides.
    pub const STRIDED: PyBufferFlags<false, true, true, true> =
        PyBufferFlags(ffi::PyBUF_STRIDED);
    /// Request shape and strides (read-only, equivalent to [`Self::STRIDES`]).
    pub const STRIDED_RO: PyBufferFlags<false, true, true> =
        PyBufferFlags(ffi::PyBUF_STRIDED_RO);
    /// Request writable buffer with shape, strides, and format.
    pub const RECORDS: PyBufferFlags<true, true, true, true> =
        PyBufferFlags(ffi::PyBUF_RECORDS);
    /// Request shape, strides, and format.
    pub const RECORDS_RO: PyBufferFlags<true, true, true> = PyBufferFlags(ffi::PyBUF_RECORDS_RO);
    /// Request writable buffer with all information including suboffsets.
    pub const FULL: PyBufferFlags<true, true, true, true> = PyBufferFlags(ffi::PyBUF_FULL);
    /// Request all buffer information including suboffsets.
    pub const FULL_RO: PyBufferFlags<true, true, true> = PyBufferFlags(ffi::PyBUF_FULL_RO);
}

/// A typed form of [`PyUntypedBufferView`]. Not constructible directly — use
/// [`PyBufferView::with()`] or [`PyBufferView::with_flags()`].
#[repr(transparent)]
pub struct PyBufferView<T, Flags: PyBufferFlagsType = PyBufferFlags<true, true, true>>(
    PyUntypedBufferView<Flags>,
    PhantomData<[T]>,
);

/// Stack-allocated untyped buffer view.
///
/// Unlike [`PyUntypedBuffer`] which heap-allocates, this places the `Py_buffer` on the
/// stack. The scoped closure API ensures the buffer cannot be moved.
///
/// Use [`with_flags()`](Self::with_flags) with a [`PyBufferFlags`] constant to acquire a view.
/// The available accessors depend on the flags used.
pub struct PyUntypedBufferView<Flags: PyBufferFlagsType = PyBufferFlags> {
    raw: ffi::Py_buffer,
    _flags: PhantomData<Flags>,
}

impl<Flags: PyBufferFlagsType> PyUntypedBufferView<Flags> {
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
        self.raw.readonly != 0
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

    /// Returns the suboffsets array.
    ///
    /// May return `None` even with `PyBUF_INDIRECT` if the exporter sets `suboffsets` to NULL.
    #[inline]
    pub fn suboffsets(&self) -> Option<&[isize]> {
        if self.raw.suboffsets.is_null() {
            return None;
        }

        Some(unsafe { slice::from_raw_parts(self.raw.suboffsets, self.raw.ndim as usize) })
    }

    /// Gets whether the buffer is contiguous in C-style order.
    #[inline]
    pub fn is_c_contiguous(&self) -> bool {
        unsafe { ffi::PyBuffer_IsContiguous(&self.raw, b'C' as std::ffi::c_char) != 0 }
    }

    /// Gets whether the buffer is contiguous in Fortran-style order.
    #[inline]
    pub fn is_fortran_contiguous(&self) -> bool {
        unsafe { ffi::PyBuffer_IsContiguous(&self.raw, b'F' as std::ffi::c_char) != 0 }
    }
}

impl<const SHAPE: bool, const STRIDE: bool, const WRITABLE: bool, const CCONTIGUOUS: bool, const FCONTIGUOUS: bool>
    PyUntypedBufferView<PyBufferFlags<true, SHAPE, STRIDE, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>>
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
    ) -> PyResult<&PyBufferView<T, PyBufferFlags<true, SHAPE, STRIDE, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>>> {
        self.ensure_compatible_with::<T>()?;
        // SAFETY: PyBufferView<T, ..> is repr(transparent) around PyUntypedBufferView<..>
        Ok(unsafe {
            NonNull::from(self)
                .cast::<PyBufferView<T, PyBufferFlags<true, SHAPE, STRIDE, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>>>()
                .as_ref()
        })
    }

    fn ensure_compatible_with<T: Element>(&self) -> PyResult<()> {
        check_buffer_compatibility::<T>(self.raw.buf, self.item_size(), self.format())
    }
}

impl<const FORMAT: bool, const STRIDE: bool, const WRITABLE: bool, const CCONTIGUOUS: bool, const FCONTIGUOUS: bool>
    PyUntypedBufferView<PyBufferFlags<FORMAT, true, STRIDE, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>>
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

impl<const FORMAT: bool, const SHAPE: bool, const WRITABLE: bool, const CCONTIGUOUS: bool, const FCONTIGUOUS: bool>
    PyUntypedBufferView<PyBufferFlags<FORMAT, SHAPE, true, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>>
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
    /// Use predefined flag constants like [`PyBufferFlags::SIMPLE`], [`PyBufferFlags::ND`],
    /// [`PyBufferFlags::STRIDES`], [`PyBufferFlags::FULL_RO`], etc.
    pub fn with_flags<
        const FORMAT: bool,
        const SHAPE: bool,
        const STRIDE: bool,
        const WRITABLE: bool,
        const CCONTIGUOUS: bool,
        const FCONTIGUOUS: bool,
        R,
    >(
        obj: &Bound<'_, PyAny>,
        flags: PyBufferFlags<FORMAT, SHAPE, STRIDE, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>,
        f: impl FnOnce(
            &PyUntypedBufferView<PyBufferFlags<FORMAT, SHAPE, STRIDE, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>>,
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

fn debug_buffer_view(
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

impl<Flags: PyBufferFlagsType> Drop for PyUntypedBufferView<Flags> {
    fn drop(&mut self) {
        unsafe { ffi::PyBuffer_Release(&mut self.raw) }
    }
}

impl<Flags: PyBufferFlagsType> Debug for PyUntypedBufferView<Flags> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_buffer_view("PyUntypedBufferView", &self.raw, f)
    }
}

impl<T: Element> PyBufferView<T> {
    /// Acquire a typed buffer view with `PyBufferFlags::FULL_RO` flags,
    /// validating that the buffer format is compatible with `T`.
    pub fn with<R>(obj: &Bound<'_, PyAny>, f: impl FnOnce(&PyBufferView<T>) -> R) -> PyResult<R> {
        PyUntypedBufferView::with_flags(obj, PyBufferFlags::FULL_RO, |view| {
            view.as_typed::<T>().map(f)
        })?
    }

    /// Acquire a typed buffer view with the given flags.
    ///
    /// [`ffi::PyBUF_FORMAT`] is implicitly added for type validation.
    pub fn with_flags<
        const SHAPE: bool,
        const STRIDE: bool,
        const WRITABLE: bool,
        const CCONTIGUOUS: bool,
        const FCONTIGUOUS: bool,
        R,
    >(
        obj: &Bound<'_, PyAny>,
        flags: PyBufferFlags<false, SHAPE, STRIDE, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>,
        f: impl FnOnce(
            &PyBufferView<T, PyBufferFlags<true, SHAPE, STRIDE, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>>,
        ) -> R,
    ) -> PyResult<R> {
        let mut raw = mem::MaybeUninit::<ffi::Py_buffer>::uninit();

        err::error_on_minusone(obj.py(), unsafe {
            ffi::PyObject_GetBuffer(
                obj.as_ptr(),
                raw.as_mut_ptr(),
                flags.0 | ffi::PyBUF_FORMAT,
            )
        })?;

        let view =
            PyUntypedBufferView::<PyBufferFlags<true, SHAPE, STRIDE, WRITABLE, CCONTIGUOUS, FCONTIGUOUS>> {
                raw: unsafe { raw.assume_init() },
                _flags: PhantomData,
            };

        view.as_typed::<T>().map(f)
    }
}

impl<T: Element, Flags: PyBufferFlagsType> PyBufferView<T, Flags> {
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
        const WRITABLE: bool,
        const FCONTIGUOUS: bool,
    > PyBufferView<T, PyBufferFlags<FORMAT, SHAPE, STRIDE, WRITABLE, true, FCONTIGUOUS>>
{
    /// Gets the buffer memory as a slice. The buffer is guaranteed C-contiguous.
    pub fn as_contiguous_slice<'a>(&'a self, _py: Python<'a>) -> &'a [ReadOnlyCell<T>] {
        unsafe { slice::from_raw_parts(self.0.raw.buf.cast(), self.item_count()) }
    }
}

// C-contiguous + writable guaranteed — no checks needed.
impl<T: Element, const FORMAT: bool, const SHAPE: bool, const STRIDE: bool, const FCONTIGUOUS: bool>
    PyBufferView<T, PyBufferFlags<FORMAT, SHAPE, STRIDE, true, true, FCONTIGUOUS>>
{
    /// Gets the buffer memory as a mutable slice.
    /// The buffer is guaranteed C-contiguous and writable.
    pub fn as_contiguous_mut_slice<'a>(&'a self, _py: Python<'a>) -> &'a [cell::Cell<T>] {
        unsafe { slice::from_raw_parts(self.0.raw.buf.cast(), self.item_count()) }
    }
}

// Fortran-contiguous guaranteed — no contiguity check needed.
impl<
        T: Element,
        const FORMAT: bool,
        const SHAPE: bool,
        const STRIDE: bool,
        const WRITABLE: bool,
        const CCONTIGUOUS: bool,
    > PyBufferView<T, PyBufferFlags<FORMAT, SHAPE, STRIDE, WRITABLE, CCONTIGUOUS, true>>
{
    /// Gets the buffer memory as a slice. The buffer is guaranteed Fortran-contiguous.
    pub fn as_fortran_contiguous_slice<'a>(&'a self, _py: Python<'a>) -> &'a [ReadOnlyCell<T>] {
        unsafe { slice::from_raw_parts(self.0.raw.buf.cast(), self.item_count()) }
    }
}

// Fortran-contiguous + writable guaranteed — no checks needed.
impl<T: Element, const FORMAT: bool, const SHAPE: bool, const STRIDE: bool, const CCONTIGUOUS: bool>
    PyBufferView<T, PyBufferFlags<FORMAT, SHAPE, STRIDE, true, CCONTIGUOUS, true>>
{
    /// Gets the buffer memory as a mutable slice.
    /// The buffer is guaranteed Fortran-contiguous and writable.
    pub fn as_fortran_contiguous_mut_slice<'a>(
        &'a self,
        _py: Python<'a>,
    ) -> &'a [cell::Cell<T>] {
        unsafe { slice::from_raw_parts(self.0.raw.buf.cast(), self.item_count()) }
    }
}

impl<T, Flags: PyBufferFlagsType> std::ops::Deref for PyBufferView<T, Flags> {
    type Target = PyUntypedBufferView<Flags>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, Flags: PyBufferFlagsType> Debug for PyBufferView<T, Flags> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_buffer_view("PyBufferView", &self.0.raw, f)
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
                    "ndim: 1, format: \"B\", shape: [5], ",
                    "strides: [1], suboffsets: None, internal: {:?} }}",
                ),
                buffer.raw().buf,
                buffer.raw().obj,
                buffer.raw().internal
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
            PyUntypedBufferView::with_flags(&bytes, PyBufferFlags::FULL_RO, |view| {
                assert!(!view.buf_ptr().is_null());
                assert_eq!(view.len_bytes(), 5);
                assert_eq!(view.item_size(), 1);
                assert_eq!(view.item_count(), 5);
                assert!(view.readonly());
                assert_eq!(view.dimensions(), 1);
                // with() uses PyBufferFlags::FULL_RO — all Known, direct return types
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
                // PyBufferView::with uses PyBufferFlags::FULL_RO — all Known
                assert_eq!(view.format().to_str().unwrap(), "B");
                assert_eq!(view.shape(), [5]);

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

            PyUntypedBufferView::with_flags(&bytes, PyBufferFlags::SIMPLE, |view| {
                assert_eq!(view.item_count(), 5);
                assert_eq!(view.len_bytes(), 5);
                assert!(view.readonly());
                assert!(view.suboffsets().is_none());
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&bytes, PyBufferFlags::ND, |view| {
                assert_eq!(view.item_count(), 5);
                assert_eq!(view.shape(), [5]);
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&bytes, PyBufferFlags::STRIDES, |view| {
                assert_eq!(view.shape(), [5]);
                assert_eq!(view.strides(), [1]);
            })
            .unwrap();

            PyUntypedBufferView::with_flags(&bytes, PyBufferFlags::FORMAT, |view| {
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

            PyBufferView::<f32>::with_flags(&array, PyBufferFlags::ND, |view| {
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
            let result = PyBufferView::<f32>::with_flags(&bytes, PyBufferFlags::ND, |_view| {});
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
            PyBufferView::<f32>::with_flags(&array, PyBufferFlags::C_CONTIGUOUS, |view| {
                let slice = view.as_contiguous_slice(py);
                assert_eq!(slice.len(), 3);
                assert_eq!(slice[0].get(), 1.0);
                assert_eq!(slice[2].get(), 2.0);
            })
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
            let result = PyUntypedBufferView::with_flags(&list, PyBufferFlags::FULL_RO, |_view| {});
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_buffer_view_debug() {
        Python::attach(|py| {
            let bytes = PyBytes::new(py, b"abcde");

            // Debug always uses raw_format/raw_shape/raw_strides (Option in output)
            PyUntypedBufferView::with_flags(&bytes, PyBufferFlags::FULL_RO, |view| {
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
