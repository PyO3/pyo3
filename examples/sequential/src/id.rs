use core::sync::atomic::{AtomicU64, Ordering};
use core::{mem, ptr};
use std::os::raw::{c_char, c_int, c_uint, c_ulonglong, c_void};

use pyo3_ffi::*;

#[repr(C)]
pub struct PyId {
    _ob_base: PyObject,
    id: Id,
}

static COUNT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct Id(u64);

impl Id {
    fn new() -> Self {
        Id(COUNT.fetch_add(1, Ordering::Relaxed))
    }
}

unsafe extern "C" fn id_new(
    subtype: *mut PyTypeObject,
    args: *mut PyObject,
    kwds: *mut PyObject,
) -> *mut PyObject {
    if PyTuple_Size(args) != 0 || !kwds.is_null() {
        PyErr_SetString(
            PyExc_TypeError,
            "Id() takes no arguments\0".as_ptr().cast::<c_char>(),
        );
        return ptr::null_mut();
    }

    let f: allocfunc = (*subtype).tp_alloc.unwrap_or(PyType_GenericAlloc);
    let slf = f(subtype, 0);

    if slf.is_null() {
        return ptr::null_mut();
    } else {
        let id = Id::new();
        let slf = slf.cast::<PyId>();
        ptr::addr_of_mut!((*slf).id).write(id);
    }

    slf
}

unsafe extern "C" fn id_repr(slf: *mut PyObject) -> *mut PyObject {
    let slf = slf.cast::<PyId>();
    let id = (*slf).id.0;
    let string = format!("Id({})", id);
    PyUnicode_FromStringAndSize(string.as_ptr().cast::<c_char>(), string.len() as Py_ssize_t)
}

unsafe extern "C" fn id_int(slf: *mut PyObject) -> *mut PyObject {
    let slf = slf.cast::<PyId>();
    let id = (*slf).id.0;
    PyLong_FromUnsignedLongLong(id as c_ulonglong)
}

unsafe extern "C" fn id_richcompare(
    slf: *mut PyObject,
    other: *mut PyObject,
    op: c_int,
) -> *mut PyObject {
    let pytype = Py_TYPE(slf); // guaranteed to be `sequential.Id`
    if Py_TYPE(other) != pytype {
        return Py_NewRef(Py_NotImplemented());
    }
    let slf = (*slf.cast::<PyId>()).id;
    let other = (*other.cast::<PyId>()).id;

    let cmp = match op {
        pyo3_ffi::Py_LT => slf < other,
        pyo3_ffi::Py_LE => slf <= other,
        pyo3_ffi::Py_EQ => slf == other,
        pyo3_ffi::Py_NE => slf != other,
        pyo3_ffi::Py_GT => slf > other,
        pyo3_ffi::Py_GE => slf >= other,
        unrecognized => {
            let msg = format!("unrecognized richcompare opcode {}\0", unrecognized);
            PyErr_SetString(PyExc_SystemError, msg.as_ptr().cast::<c_char>());
            return ptr::null_mut();
        }
    };

    if cmp {
        Py_NewRef(Py_True())
    } else {
        Py_NewRef(Py_False())
    }
}

static mut SLOTS: &[PyType_Slot] = &[
    PyType_Slot {
        slot: Py_tp_new,
        pfunc: id_new as *mut c_void,
    },
    PyType_Slot {
        slot: Py_tp_doc,
        pfunc: "An id that is increased every time an instance is created\0".as_ptr()
            as *mut c_void,
    },
    PyType_Slot {
        slot: Py_tp_repr,
        pfunc: id_repr as *mut c_void,
    },
    PyType_Slot {
        slot: Py_nb_int,
        pfunc: id_int as *mut c_void,
    },
    PyType_Slot {
        slot: Py_tp_richcompare,
        pfunc: id_richcompare as *mut c_void,
    },
    PyType_Slot {
        slot: 0,
        pfunc: ptr::null_mut(),
    },
];

pub static mut ID_SPEC: PyType_Spec = PyType_Spec {
    name: "sequential.Id\0".as_ptr().cast::<c_char>(),
    basicsize: mem::size_of::<PyId>() as c_int,
    itemsize: 0,
    flags: (Py_TPFLAGS_DEFAULT | Py_TPFLAGS_IMMUTABLETYPE) as c_uint,
    slots: unsafe { SLOTS as *const [PyType_Slot] as *mut PyType_Slot },
};
