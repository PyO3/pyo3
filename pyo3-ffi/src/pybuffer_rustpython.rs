use crate::object::{ptr_to_pyobject_ref_borrowed, pyobject_ref_to_ptr, PyObject};
use crate::pyerrors::{PyErr_Clear, PyErr_SetString, PyExc_BufferError, PyExc_TypeError};
use crate::pyport::Py_ssize_t;
use crate::rustpython_runtime;
use rustpython_vm::protocol::PyBuffer as RpBuffer;
use rustpython_vm::TryFromBorrowedObject;
use std::ffi::{c_char, c_int, c_void, CString};
use std::ptr;
use std::slice;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Py_buffer {
    pub buf: *mut c_void,
    /// Owned reference.
    pub obj: *mut crate::PyObject,
    pub len: Py_ssize_t,
    pub itemsize: Py_ssize_t,
    pub readonly: c_int,
    pub ndim: c_int,
    pub format: *mut c_char,
    pub shape: *mut Py_ssize_t,
    pub strides: *mut Py_ssize_t,
    pub suboffsets: *mut Py_ssize_t,
    pub internal: *mut c_void,
}

impl Py_buffer {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            buf: ptr::null_mut(),
            obj: ptr::null_mut(),
            len: 0,
            itemsize: 0,
            readonly: 0,
            ndim: 0,
            format: ptr::null_mut(),
            shape: ptr::null_mut(),
            strides: ptr::null_mut(),
            suboffsets: ptr::null_mut(),
            internal: ptr::null_mut(),
        }
    }
}

pub type getbufferproc = unsafe extern "C" fn(*mut PyObject, *mut crate::Py_buffer, c_int) -> c_int;
pub type releasebufferproc = unsafe extern "C" fn(*mut PyObject, *mut crate::Py_buffer);

struct RustPythonBufferView {
    buffer: RpBuffer,
    contiguous: Vec<u8>,
    format: CString,
    shape: Box<[Py_ssize_t]>,
    strides: Box<[Py_ssize_t]>,
    suboffsets: Box<[Py_ssize_t]>,
}

impl RustPythonBufferView {
    fn new(buffer: RpBuffer) -> Self {
        let format = CString::new(buffer.desc.format.as_bytes())
            .expect("RustPython buffer format strings never contain NUL");
        let shape: Box<[Py_ssize_t]> = buffer
            .desc
            .dim_desc
            .iter()
            .map(|(shape, _, _)| *shape as Py_ssize_t)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let strides: Box<[Py_ssize_t]> = buffer
            .desc
            .dim_desc
            .iter()
            .map(|(_, stride, _)| *stride as Py_ssize_t)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let suboffsets: Box<[Py_ssize_t]> = buffer
            .desc
            .dim_desc
            .iter()
            .map(|(_, _, suboffset)| *suboffset as Py_ssize_t)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let contiguous = buffer.contiguous_or_collect(|bytes| bytes.to_vec());
        Self {
            buffer,
            contiguous,
            format,
            shape,
            strides,
            suboffsets,
        }
    }

    fn write_back(&mut self) -> Result<(), ()> {
        if self.buffer.desc.readonly {
            return Err(());
        }

        if let Some(mut bytes) = self.buffer.as_contiguous_mut() {
            if bytes.len() != self.contiguous.len() {
                return Err(());
            }
            bytes.copy_from_slice(&self.contiguous);
            return Ok(());
        }

        let mut target = self.buffer.obj_bytes_mut();
        let target = &mut *target;
        let mut offset = 0usize;
        self.buffer.desc.for_each_segment(true, |range| {
            let start = range.start as usize;
            let end = range.end as usize;
            let len = end - start;
            target[start..end].copy_from_slice(&self.contiguous[offset..offset + len]);
            offset += len;
        });
        Ok(())
    }

    fn pointer_at(&self, indices: &[Py_ssize_t]) -> *mut c_void {
        if indices.len() != self.buffer.desc.ndim() {
            return ptr::null_mut();
        }
        let mut position = 0isize;
        for (&index, &(_, stride, suboffset)) in indices.iter().zip(self.buffer.desc.dim_desc.iter()) {
            position += index as isize * stride + suboffset;
        }
        if position.is_negative() || position as usize >= self.contiguous.len() {
            return ptr::null_mut();
        }
        unsafe { self.contiguous.as_ptr().add(position as usize) as *mut c_void }
    }
}

unsafe fn view_from_ptr<'a>(view: *const Py_buffer) -> Option<&'a RustPythonBufferView> {
    let internal = (*view).internal;
    (!internal.is_null()).then(|| &*(internal as *const RustPythonBufferView))
}

unsafe fn view_from_mut_ptr<'a>(view: *mut Py_buffer) -> Option<&'a mut RustPythonBufferView> {
    let internal = (*view).internal;
    (!internal.is_null()).then(|| &mut *(internal as *mut RustPythonBufferView))
}

unsafe fn set_buffer_error(exc: *mut PyObject, msg: &str) {
    let msg = CString::new(msg).expect("static error messages never contain NUL");
    PyErr_SetString(exc, msg.as_ptr());
}

pub unsafe fn PyObject_CheckBuffer(obj: *mut PyObject) -> c_int {
    if obj.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(obj);
    rustpython_runtime::with_vm(|vm| {
        RpBuffer::try_from_borrowed_object(vm, &obj)
            .map(|_| 1)
            .unwrap_or(0)
    })
}

pub unsafe fn PyObject_GetBuffer(obj: *mut PyObject, view: *mut Py_buffer, flags: c_int) -> c_int {
    if obj.is_null() || view.is_null() {
        set_buffer_error(PyExc_BufferError, "PyObject_GetBuffer received a null pointer");
        return -1;
    }

    let obj_ref = ptr_to_pyobject_ref_borrowed(obj);
    let result = rustpython_runtime::with_vm(|vm| RpBuffer::try_from_borrowed_object(vm, &obj_ref));
    let buffer = match result {
        Ok(buffer) => buffer,
        Err(_) => {
            set_buffer_error(PyExc_TypeError, "object does not support the buffer protocol");
            return -1;
        }
    };

    if (flags & PyBUF_WRITABLE) != 0 && buffer.desc.readonly {
        set_buffer_error(PyExc_BufferError, "buffer is not writable");
        return -1;
    }

    let mut internal = Box::new(RustPythonBufferView::new(buffer));
    let obj_ptr = pyobject_ref_to_ptr(internal.buffer.obj.clone());

    *view = Py_buffer {
        buf: internal.contiguous.as_mut_ptr().cast(),
        obj: obj_ptr,
        len: internal.buffer.desc.len as Py_ssize_t,
        itemsize: internal.buffer.desc.itemsize as Py_ssize_t,
        readonly: internal.buffer.desc.readonly.into(),
        ndim: internal.buffer.desc.ndim() as c_int,
        format: internal.format.as_ptr().cast_mut(),
        shape: internal.shape.as_mut_ptr(),
        strides: internal.strides.as_mut_ptr(),
        suboffsets: internal.suboffsets.as_mut_ptr(),
        internal: Box::into_raw(internal).cast(),
    };
    PyErr_Clear();
    0
}

pub unsafe fn PyBuffer_GetPointer(
    view: *const Py_buffer,
    indices: *const Py_ssize_t,
) -> *mut c_void {
    if view.is_null() || indices.is_null() {
        return ptr::null_mut();
    }
    let Some(internal) = view_from_ptr(view) else {
        return ptr::null_mut();
    };
    let indices = slice::from_raw_parts(indices, internal.buffer.desc.ndim());
    internal.pointer_at(indices)
}

pub unsafe fn PyBuffer_SizeFromFormat(format: *const c_char) -> Py_ssize_t {
    if format.is_null() {
        return 1;
    }
    let first = *format.cast::<u8>();
    match first {
        b'?' | b'b' | b'B' | b'c' => 1,
        b'h' | b'H' | b'e' => 2,
        b'i' | b'I' | b'l' | b'L' | b'f' => 4,
        b'q' | b'Q' | b'd' => 8,
        _ => 1,
    }
}

pub unsafe fn PyBuffer_ToContiguous(
    buf: *mut c_void,
    view: *const Py_buffer,
    len: Py_ssize_t,
    _order: c_char,
) -> c_int {
    if buf.is_null() || view.is_null() {
        set_buffer_error(PyExc_BufferError, "PyBuffer_ToContiguous received a null pointer");
        return -1;
    }
    let Some(internal) = view_from_ptr(view) else {
        set_buffer_error(PyExc_BufferError, "buffer view is not initialized");
        return -1;
    };
    let len = len.max(0) as usize;
    if len > internal.contiguous.len() {
        set_buffer_error(PyExc_BufferError, "requested length exceeds buffer size");
        return -1;
    }
    ptr::copy_nonoverlapping(internal.contiguous.as_ptr(), buf.cast::<u8>(), len);
    0
}

pub unsafe fn PyBuffer_FromContiguous(
    view: *const Py_buffer,
    buf: *const c_void,
    len: Py_ssize_t,
    _order: c_char,
) -> c_int {
    if view.is_null() || buf.is_null() {
        set_buffer_error(
            PyExc_BufferError,
            "PyBuffer_FromContiguous received a null pointer",
        );
        return -1;
    }
    let view = view.cast_mut();
    let Some(internal) = view_from_mut_ptr(view) else {
        set_buffer_error(PyExc_BufferError, "buffer view is not initialized");
        return -1;
    };
    if internal.buffer.desc.readonly {
        set_buffer_error(PyExc_BufferError, "buffer is not writable");
        return -1;
    }
    let len = len.max(0) as usize;
    if len > internal.contiguous.len() {
        set_buffer_error(PyExc_BufferError, "source length exceeds buffer size");
        return -1;
    }
    ptr::copy_nonoverlapping(buf.cast::<u8>(), internal.contiguous.as_mut_ptr(), len);
    if internal.write_back().is_err() {
        set_buffer_error(PyExc_BufferError, "failed to write contiguous bytes back to source");
        return -1;
    }
    (*view).buf = internal.contiguous.as_mut_ptr().cast();
    0
}

pub unsafe fn PyObject_CopyData(dest: *mut PyObject, src: *mut PyObject) -> c_int {
    let mut source = Py_buffer::new();
    let mut target = Py_buffer::new();
    if PyObject_GetBuffer(src, &mut source, PyBUF_FULL_RO) != 0 {
        return -1;
    }
    if PyObject_GetBuffer(dest, &mut target, PyBUF_WRITABLE) != 0 {
        PyBuffer_Release(&mut source);
        return -1;
    }

    let len = source.len.min(target.len);
    let rc = if len < 0 {
        -1
    } else {
        PyBuffer_FromContiguous(&target, source.buf, len, b'C' as c_char)
    };
    PyBuffer_Release(&mut target);
    PyBuffer_Release(&mut source);
    rc
}

pub unsafe fn PyBuffer_IsContiguous(view: *const Py_buffer, fort: c_char) -> c_int {
    if view.is_null() {
        return 0;
    }
    let ndim = (*view).ndim.max(0) as usize;
    if ndim == 0 || (*view).len == 0 {
        return 1;
    }
    if (*view).shape.is_null() || (*view).strides.is_null() {
        return 0;
    }

    let shape = slice::from_raw_parts((*view).shape, ndim);
    let strides = slice::from_raw_parts((*view).strides, ndim);
    let itemsize = (*view).itemsize;
    let c_order = fort == b'C' as c_char || fort == b'A' as c_char;
    let f_order = fort == b'F' as c_char;

    let is_c = {
        let mut expected = itemsize;
        let mut ok = true;
        for (&dim, &stride) in shape.iter().rev().zip(strides.iter().rev()) {
            if dim > 1 && stride != expected {
                ok = false;
                break;
            }
            expected = expected.saturating_mul(dim);
        }
        ok
    };
    let is_f = {
        let mut expected = itemsize;
        let mut ok = true;
        for (&dim, &stride) in shape.iter().zip(strides.iter()) {
            if dim > 1 && stride != expected {
                ok = false;
                break;
            }
            expected = expected.saturating_mul(dim);
        }
        ok
    };

    if c_order {
        is_c.into()
    } else if f_order {
        is_f.into()
    } else {
        (is_c || is_f).into()
    }
}

pub unsafe fn PyBuffer_FillContiguousStrides(
    ndims: c_int,
    shape: *mut Py_ssize_t,
    strides: *mut Py_ssize_t,
    itemsize: c_int,
    fort: c_char,
) {
    if ndims <= 0 || shape.is_null() || strides.is_null() {
        return;
    }
    let ndims = ndims as usize;
    let shape = slice::from_raw_parts(shape, ndims);
    let strides_out = slice::from_raw_parts_mut(strides, ndims);
    let mut stride = itemsize as Py_ssize_t;
    if fort == b'F' as c_char {
        for (out, &dim) in strides_out.iter_mut().zip(shape.iter()) {
            *out = stride;
            stride = stride.saturating_mul(dim);
        }
    } else {
        for (out, &dim) in strides_out.iter_mut().rev().zip(shape.iter().rev()) {
            *out = stride;
            stride = stride.saturating_mul(dim);
        }
    }
}

pub unsafe fn PyBuffer_FillInfo(
    view: *mut Py_buffer,
    o: *mut PyObject,
    buf: *mut c_void,
    len: Py_ssize_t,
    readonly: c_int,
    _flags: c_int,
) -> c_int {
    if view.is_null() {
        set_buffer_error(PyExc_BufferError, "PyBuffer_FillInfo received a null view");
        return -1;
    }
    *view = Py_buffer {
        buf,
        obj: o,
        len,
        itemsize: 1,
        readonly,
        ndim: 1,
        format: ptr::null_mut(),
        shape: ptr::null_mut(),
        strides: ptr::null_mut(),
        suboffsets: ptr::null_mut(),
        internal: ptr::null_mut(),
    };
    0
}

pub unsafe fn PyBuffer_Release(view: *mut Py_buffer) {
    if view.is_null() {
        return;
    }
    if !(*view).internal.is_null() {
        drop(Box::from_raw((*view).internal.cast::<RustPythonBufferView>()));
    }
    if !(*view).obj.is_null() {
        crate::Py_DECREF((*view).obj);
    }
    *view = Py_buffer::new();
}

/// Maximum number of dimensions.
pub const PyBUF_MAX_NDIM: usize = 64;

/* Flags for getting buffers */
pub const PyBUF_SIMPLE: c_int = 0;
pub const PyBUF_WRITABLE: c_int = 0x0001;
pub const PyBUF_WRITEABLE: c_int = PyBUF_WRITABLE;
pub const PyBUF_FORMAT: c_int = 0x0004;
pub const PyBUF_ND: c_int = 0x0008;
pub const PyBUF_STRIDES: c_int = 0x0010 | PyBUF_ND;
pub const PyBUF_C_CONTIGUOUS: c_int = 0x0020 | PyBUF_STRIDES;
pub const PyBUF_F_CONTIGUOUS: c_int = 0x0040 | PyBUF_STRIDES;
pub const PyBUF_ANY_CONTIGUOUS: c_int = 0x0080 | PyBUF_STRIDES;
pub const PyBUF_INDIRECT: c_int = 0x0100 | PyBUF_STRIDES;

pub const PyBUF_CONTIG: c_int = PyBUF_ND | PyBUF_WRITABLE;
pub const PyBUF_CONTIG_RO: c_int = PyBUF_ND;

pub const PyBUF_STRIDED: c_int = PyBUF_STRIDES | PyBUF_WRITABLE;
pub const PyBUF_STRIDED_RO: c_int = PyBUF_STRIDES;

pub const PyBUF_RECORDS: c_int = PyBUF_STRIDES | PyBUF_WRITABLE | PyBUF_FORMAT;
pub const PyBUF_RECORDS_RO: c_int = PyBUF_STRIDES | PyBUF_FORMAT;

pub const PyBUF_FULL: c_int = PyBUF_INDIRECT | PyBUF_WRITABLE | PyBUF_FORMAT;
pub const PyBUF_FULL_RO: c_int = PyBUF_INDIRECT | PyBUF_FORMAT;

pub const PyBUF_READ: c_int = 0x100;
pub const PyBUF_WRITE: c_int = 0x200;
