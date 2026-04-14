use crate::object::{ptr_to_pyobject_ref_borrowed, pyobject_ref_to_ptr, PyObject, PyTypeObject};
use crate::pyerrors::{set_vm_exception, PyErr_Clear, PyErr_SetString, PyExc_BufferError, PyExc_TypeError};
use crate::pyport::Py_ssize_t;
use crate::rustpython_runtime;
use crate::{PyErr_Clear as FfiPyErrClear, Py_TYPE};
use rustpython_vm::AsObject;
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

pub(crate) struct HeapTypeBufferView {
    pub(crate) releasebuffer: Option<releasebufferproc>,
}

pub(crate) enum BufferViewState {
    RustPython(RustPythonBufferView),
    HeapType(HeapTypeBufferView),
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
    if internal.is_null() {
        return None;
    }
    match &*(internal as *const BufferViewState) {
        BufferViewState::RustPython(internal) => Some(internal),
        BufferViewState::HeapType(_) => None,
    }
}

unsafe fn view_from_mut_ptr<'a>(view: *mut Py_buffer) -> Option<&'a mut RustPythonBufferView> {
    let internal = (*view).internal;
    if internal.is_null() {
        return None;
    }
    match &mut *(internal as *mut BufferViewState) {
        BufferViewState::RustPython(internal) => Some(internal),
        BufferViewState::HeapType(_) => None,
    }
}

unsafe fn raw_view_pointer_at(
    view: *const Py_buffer,
    indices: Option<&[Py_ssize_t]>,
) -> Option<*mut u8> {
    if view.is_null() || unsafe { (*view).buf.is_null() } {
        return None;
    }

    let ndim = unsafe { (*view).ndim.max(0) as usize };
    let mut ptr = unsafe { (*view).buf.cast::<u8>() };
    let indices = indices.unwrap_or(&[]);

    if indices.len() > ndim {
        return None;
    }

    if !unsafe { (*view).strides.is_null() } {
        let strides = unsafe { slice::from_raw_parts((*view).strides, ndim) };
        let suboffsets = if unsafe { (*view).suboffsets.is_null() } {
            None
        } else {
            Some(unsafe { slice::from_raw_parts((*view).suboffsets, ndim) })
        };
        for (i, index) in indices.iter().enumerate() {
            ptr = unsafe { ptr.offset(strides[i] * *index) };
            if let Some(suboffsets) = suboffsets {
                if suboffsets[i] >= 0 {
                    ptr = unsafe { (*(ptr as *mut *mut u8)).offset(suboffsets[i]) };
                }
            }
        }
        Some(ptr)
    } else if indices.is_empty() {
        Some(ptr)
    } else {
        let itemsize = unsafe { (*view).itemsize.max(1) };
        let linear_index = indices.iter().copied().sum::<Py_ssize_t>();
        Some(unsafe { ptr.offset(itemsize * linear_index) })
    }
}

unsafe fn set_buffer_error(exc: *mut PyObject, msg: &str) {
    let msg = CString::new(msg).expect("static error messages never contain NUL");
    PyErr_SetString(exc, msg.as_ptr());
}

unsafe fn heap_buffer_metadata(obj: *mut PyObject) -> crate::object::HeapTypeMetadata {
    let cls_ptr = Py_TYPE(obj) as *mut PyTypeObject;
    crate::object::heap_type_metadata_for_ptr(cls_ptr)
}

pub unsafe fn PyObject_CheckBuffer(obj: *mut PyObject) -> c_int {
    if obj.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(obj);
    rustpython_runtime::with_vm(|vm| {
        RpBuffer::try_from_borrowed_object(vm, &obj)
            .map(|_| 1)
            .unwrap_or_else(|_| {
                let metadata = unsafe { heap_buffer_metadata(pyobject_ref_to_ptr(obj.to_owned())) };
                (metadata.bf_getbuffer != 0) as c_int
            })
    })
}

pub unsafe fn PyObject_GetBuffer(obj: *mut PyObject, view: *mut Py_buffer, flags: c_int) -> c_int {
    if obj.is_null() || view.is_null() {
        set_buffer_error(PyExc_BufferError, "PyObject_GetBuffer received a null pointer");
        return -1;
    }

    let metadata = unsafe { heap_buffer_metadata(obj) };
    if metadata.bf_getbuffer != 0 {
        let getbuffer: getbufferproc = unsafe { std::mem::transmute(metadata.bf_getbuffer) };
        let rc = unsafe { getbuffer(obj, view, flags) };
        if rc != 0 {
            return -1;
        }
        if unsafe { (*view).obj.is_null() } {
            unsafe {
                (*view).obj = obj;
                crate::Py_INCREF((*view).obj);
            }
        }
        unsafe {
            (*view).internal = Box::into_raw(Box::new(BufferViewState::HeapType(
                HeapTypeBufferView {
                    releasebuffer: (metadata.bf_releasebuffer != 0)
                        .then(|| std::mem::transmute(metadata.bf_releasebuffer)),
                },
            ))) as *mut c_void;
        }
        unsafe { PyErr_Clear() };
        return 0;
    }

    let obj_ref = ptr_to_pyobject_ref_borrowed(obj);
    let result = rustpython_runtime::with_vm(|vm| {
        match RpBuffer::try_from_borrowed_object(vm, &obj_ref) {
            Ok(buffer) => Ok(buffer),
            Err(_) => Err(vm.new_type_error("object does not support the buffer protocol")),
        }
    });
    let buffer = match result {
        Ok(buffer) => buffer,
        Err(exc) => {
            set_vm_exception(exc);
            return -1;
        }
    };

    if (flags & PyBUF_WRITABLE) != 0 && buffer.desc.readonly {
        set_buffer_error(PyExc_BufferError, "buffer is not writable");
        return -1;
    }

    let mut internal = RustPythonBufferView::new(buffer);
    let obj_ptr = pyobject_ref_to_ptr(internal.buffer.obj.clone());
    let suboffsets_ptr = if internal.suboffsets.iter().all(|offset| *offset <= 0) {
        ptr::null_mut()
    } else {
        internal.suboffsets.as_mut_ptr()
    };

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
        suboffsets: suboffsets_ptr,
        internal: Box::into_raw(Box::new(BufferViewState::RustPython(internal))).cast(),
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
    if let Some(internal) = view_from_ptr(view) {
        let indices = slice::from_raw_parts(indices, internal.buffer.desc.ndim());
        return internal.pointer_at(indices);
    }
    let ndim = unsafe { (*view).ndim.max(0) as usize };
    let indices = unsafe { slice::from_raw_parts(indices, ndim) };
    unsafe { raw_view_pointer_at(view, Some(indices)) }
        .map(|ptr| ptr.cast::<c_void>())
        .unwrap_or(ptr::null_mut())
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
    let len = len.max(0) as usize;
    if let Some(internal) = view_from_ptr(view) {
        if len > internal.contiguous.len() {
            set_buffer_error(PyExc_BufferError, "requested length exceeds buffer size");
            return -1;
        }
        ptr::copy_nonoverlapping(internal.contiguous.as_ptr(), buf.cast::<u8>(), len);
        return 0;
    }
    if unsafe { (*view).len.max(0) as usize } < len {
        set_buffer_error(PyExc_BufferError, "requested length exceeds buffer size");
        return -1;
    }
    if !unsafe { PyBuffer_IsContiguous(view, b'C' as c_char) != 0 } {
        set_buffer_error(PyExc_BufferError, "buffer view is not contiguous");
        return -1;
    }
    let Some(src) = (unsafe { raw_view_pointer_at(view, None) }) else {
        set_buffer_error(PyExc_BufferError, "buffer view is not initialized");
        return -1;
    };
    unsafe { ptr::copy_nonoverlapping(src, buf.cast::<u8>(), len) };
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
    let len = len.max(0) as usize;
    if let Some(internal) = view_from_mut_ptr(view) {
        if internal.buffer.desc.readonly {
            set_buffer_error(PyExc_BufferError, "buffer is not writable");
            return -1;
        }
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
        return 0;
    }
    if unsafe { (*view).readonly != 0 } {
        set_buffer_error(PyExc_BufferError, "buffer is not writable");
        return -1;
    }
    if unsafe { (*view).len.max(0) as usize } < len {
        set_buffer_error(PyExc_BufferError, "source length exceeds buffer size");
        return -1;
    }
    if !unsafe { PyBuffer_IsContiguous(view, b'C' as c_char) != 0 } {
        set_buffer_error(PyExc_BufferError, "buffer view is not contiguous");
        return -1;
    }
    let Some(dest) = (unsafe { raw_view_pointer_at(view, None) }) else {
        set_buffer_error(PyExc_BufferError, "buffer view is not initialized");
        return -1;
    };
    unsafe { ptr::copy_nonoverlapping(buf.cast::<u8>(), dest, len) };
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
    let can_release = rustpython_runtime::is_attached()
        || rustpython_runtime::runtime_thread_id() == Some(std::thread::current().id());
    if !can_release {
        // After embedded interpreter shutdown there is no valid RustPython attach context.
        // Releasing RustPython-owned buffer state would re-enter the VM during drop and panic.
        // Leak the final buffer bookkeeping instead; process-shutdown soundness matters more
        // than reclaiming these last references once the interpreter context is gone.
        *view = Py_buffer::new();
        return;
    }
    if !(*view).internal.is_null() {
        match *Box::from_raw((*view).internal.cast::<BufferViewState>()) {
            BufferViewState::RustPython(internal) => {
                drop(internal);
            }
            BufferViewState::HeapType(internal) => {
                if let Some(releasebuffer) = internal.releasebuffer {
                    releasebuffer((*view).obj, view);
                }
            }
        }
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
