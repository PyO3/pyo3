use crate::pyport::Py_ssize_t;
use crate::PyObject;
#[cfg(py_sys_config = "Py_REF_DEBUG")]
use std::ffi::c_char;
#[cfg(Py_3_12)]
use std::ffi::c_int;
#[cfg(all(Py_3_14, any(not(Py_GIL_DISABLED), target_pointer_width = "32")))]
use std::ffi::c_long;
#[cfg(any(Py_GIL_DISABLED, all(Py_3_12, not(Py_3_14))))]
use std::ffi::c_uint;
#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
use std::ffi::c_ulong;
use std::ptr;
#[cfg(Py_GIL_DISABLED)]
use std::sync::atomic::Ordering::Relaxed;

#[cfg(Py_3_14)]
const _Py_STATICALLY_ALLOCATED_FLAG: c_int = 1 << 7;

#[cfg(all(Py_3_12, not(Py_3_14)))]
const _Py_IMMORTAL_REFCNT: Py_ssize_t = {
    if cfg!(target_pointer_width = "64") {
        c_uint::MAX as Py_ssize_t
    } else {
        // for 32-bit systems, use the lower 30 bits (see comment in CPython's object.h)
        (c_uint::MAX >> 2) as Py_ssize_t
    }
};

// comments in Python.h about the choices for these constants

#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
const _Py_IMMORTAL_INITIAL_REFCNT: Py_ssize_t = {
    if cfg!(target_pointer_width = "64") {
        ((3 as c_ulong) << (30 as c_ulong)) as Py_ssize_t
    } else {
        ((5 as c_long) << (28 as c_long)) as Py_ssize_t
    }
};

#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
const _Py_STATIC_IMMORTAL_INITIAL_REFCNT: Py_ssize_t = {
    if cfg!(target_pointer_width = "64") {
        _Py_IMMORTAL_INITIAL_REFCNT
            | ((_Py_STATICALLY_ALLOCATED_FLAG as Py_ssize_t) << (32 as Py_ssize_t))
    } else {
        ((7 as c_long) << (28 as c_long)) as Py_ssize_t
    }
};

#[cfg(all(Py_3_14, target_pointer_width = "32"))]
const _Py_IMMORTAL_MINIMUM_REFCNT: Py_ssize_t = ((1 as c_long) << (30 as c_long)) as Py_ssize_t;

#[cfg(all(Py_3_14, target_pointer_width = "32"))]
const _Py_STATIC_IMMORTAL_MINIMUM_REFCNT: Py_ssize_t =
    ((6 as c_long) << (28 as c_long)) as Py_ssize_t;

#[cfg(all(Py_3_14, Py_GIL_DISABLED))]
const _Py_IMMORTAL_INITIAL_REFCNT: Py_ssize_t = c_uint::MAX as Py_ssize_t;

#[cfg(Py_GIL_DISABLED)]
pub(crate) const _Py_IMMORTAL_REFCNT_LOCAL: u32 = u32::MAX;

#[cfg(Py_GIL_DISABLED)]
const _Py_REF_SHARED_SHIFT: isize = 2;
// skipped private _Py_REF_SHARED_FLAG_MASK

// skipped private _Py_REF_SHARED_INIT
// skipped private _Py_REF_MAYBE_WEAKREF
// skipped private _Py_REF_QUEUED
// skipped private _Py_REF_MERGED

// skipped private _Py_REF_SHARED

extern "C" {
    #[cfg(all(Py_3_14, Py_LIMITED_API))]
    pub fn Py_REFCNT(ob: *mut PyObject) -> Py_ssize_t;
}

#[cfg(not(all(Py_3_14, Py_LIMITED_API)))]
#[inline]
pub unsafe fn Py_REFCNT(ob: *mut PyObject) -> Py_ssize_t {
    #[cfg(Py_GIL_DISABLED)]
    {
        let local = (*ob).ob_ref_local.load(Relaxed);
        if local == _Py_IMMORTAL_REFCNT_LOCAL {
            #[cfg(not(Py_3_14))]
            return _Py_IMMORTAL_REFCNT;
            #[cfg(Py_3_14)]
            return _Py_IMMORTAL_INITIAL_REFCNT;
        }
        let shared = (*ob).ob_ref_shared.load(Relaxed);
        local as Py_ssize_t + Py_ssize_t::from(shared >> _Py_REF_SHARED_SHIFT)
    }

    #[cfg(all(Py_LIMITED_API, Py_3_14))]
    {
        Py_REFCNT(ob)
    }

    #[cfg(all(not(Py_GIL_DISABLED), not(all(Py_LIMITED_API, Py_3_14)), Py_3_12))]
    {
        (*ob).ob_refcnt.ob_refcnt
    }

    #[cfg(all(not(Py_GIL_DISABLED), not(Py_3_12), not(GraalPy)))]
    {
        (*ob).ob_refcnt
    }

    #[cfg(all(not(Py_GIL_DISABLED), not(Py_3_12), GraalPy))]
    {
        _Py_REFCNT(ob)
    }
}

#[cfg(Py_3_12)]
#[inline(always)]
unsafe fn _Py_IsImmortal(op: *mut PyObject) -> c_int {
    #[cfg(all(target_pointer_width = "64", not(Py_GIL_DISABLED)))]
    {
        (((*op).ob_refcnt.ob_refcnt as crate::PY_INT32_T) < 0) as c_int
    }

    #[cfg(all(target_pointer_width = "32", not(Py_GIL_DISABLED)))]
    {
        #[cfg(not(Py_3_14))]
        {
            ((*op).ob_refcnt.ob_refcnt == _Py_IMMORTAL_REFCNT) as c_int
        }

        #[cfg(Py_3_14)]
        {
            ((*op).ob_refcnt.ob_refcnt >= _Py_IMMORTAL_MINIMUM_REFCNT) as c_int
        }
    }

    #[cfg(Py_GIL_DISABLED)]
    {
        ((*op).ob_ref_local.load(Relaxed) == _Py_IMMORTAL_REFCNT_LOCAL) as c_int
    }
}

// skipped _Py_IsStaticImmortal

// TODO: Py_SET_REFCNT

extern "C" {
    #[cfg(all(py_sys_config = "Py_REF_DEBUG", not(Py_LIMITED_API)))]
    fn _Py_NegativeRefcount(filename: *const c_char, lineno: c_int, op: *mut PyObject);
    #[cfg(all(Py_3_12, py_sys_config = "Py_REF_DEBUG", not(Py_LIMITED_API)))]
    fn _Py_INCREF_IncRefTotal();
    #[cfg(all(Py_3_12, py_sys_config = "Py_REF_DEBUG", not(Py_LIMITED_API)))]
    fn _Py_DECREF_DecRefTotal();

    #[cfg_attr(PyPy, link_name = "_PyPy_Dealloc")]
    fn _Py_Dealloc(arg1: *mut PyObject);

    #[cfg_attr(PyPy, link_name = "PyPy_IncRef")]
    #[cfg_attr(GraalPy, link_name = "_Py_IncRef")]
    pub fn Py_IncRef(o: *mut PyObject);
    #[cfg_attr(PyPy, link_name = "PyPy_DecRef")]
    #[cfg_attr(GraalPy, link_name = "_Py_DecRef")]
    pub fn Py_DecRef(o: *mut PyObject);

    #[cfg(all(Py_3_10, not(PyPy)))]
    fn _Py_IncRef(o: *mut PyObject);
    #[cfg(all(Py_3_10, not(PyPy)))]
    fn _Py_DecRef(o: *mut PyObject);

    #[cfg(GraalPy)]
    fn _Py_REFCNT(arg1: *const PyObject) -> Py_ssize_t;
}

#[inline(always)]
pub unsafe fn Py_INCREF(op: *mut PyObject) {
    // On limited API, the free-threaded build, or with refcount debugging, let the interpreter do refcounting
    // TODO: reimplement the logic in the header in the free-threaded build, for a little bit of performance.
    #[cfg(any(
        Py_GIL_DISABLED,
        Py_LIMITED_API,
        py_sys_config = "Py_REF_DEBUG",
        GraalPy
    ))]
    {
        // _Py_IncRef was added to the ABI in 3.10; skips null checks
        #[cfg(all(Py_3_10, not(PyPy)))]
        {
            _Py_IncRef(op);
        }

        #[cfg(any(not(Py_3_10), PyPy))]
        {
            Py_IncRef(op);
        }
    }

    // version-specific builds are allowed to directly manipulate the reference count
    #[cfg(not(any(
        Py_GIL_DISABLED,
        Py_LIMITED_API,
        py_sys_config = "Py_REF_DEBUG",
        GraalPy
    )))]
    {
        #[cfg(all(Py_3_14, target_pointer_width = "64"))]
        {
            let cur_refcnt = (*op).ob_refcnt.ob_refcnt;
            if (cur_refcnt as i32) < 0 {
                return;
            }
            (*op).ob_refcnt.ob_refcnt = cur_refcnt.wrapping_add(1);
        }

        #[cfg(all(Py_3_12, not(Py_3_14), target_pointer_width = "64"))]
        {
            let cur_refcnt = (*op).ob_refcnt.ob_refcnt_split[crate::PY_BIG_ENDIAN];
            let new_refcnt = cur_refcnt.wrapping_add(1);
            if new_refcnt == 0 {
                return;
            }
            (*op).ob_refcnt.ob_refcnt_split[crate::PY_BIG_ENDIAN] = new_refcnt;
        }

        #[cfg(all(Py_3_12, target_pointer_width = "32"))]
        {
            if _Py_IsImmortal(op) != 0 {
                return;
            }
            (*op).ob_refcnt.ob_refcnt += 1
        }

        #[cfg(not(Py_3_12))]
        {
            (*op).ob_refcnt += 1
        }

        // Skipped _Py_INCREF_STAT_INC - if anyone wants this, please file an issue
        // or submit a PR supporting Py_STATS build option and pystats.h
    }
}

// skipped _Py_DecRefShared
// skipped _Py_DecRefSharedDebug
// skipped _Py_MergeZeroLocalRefcount

#[inline(always)]
#[cfg_attr(
    all(py_sys_config = "Py_REF_DEBUG", Py_3_12, not(Py_LIMITED_API)),
    track_caller
)]
pub unsafe fn Py_DECREF(op: *mut PyObject) {
    // On limited API, the free-threaded build, or with refcount debugging, let the interpreter do refcounting
    // On 3.12+ we implement refcount debugging to get better assertion locations on negative refcounts
    // TODO: reimplement the logic in the header in the free-threaded build, for a little bit of performance.
    #[cfg(any(
        Py_GIL_DISABLED,
        Py_LIMITED_API,
        all(py_sys_config = "Py_REF_DEBUG", not(Py_3_12)),
        GraalPy
    ))]
    {
        // _Py_DecRef was added to the ABI in 3.10; skips null checks
        #[cfg(all(Py_3_10, not(PyPy)))]
        {
            _Py_DecRef(op);
        }

        #[cfg(any(not(Py_3_10), PyPy))]
        {
            Py_DecRef(op);
        }
    }

    #[cfg(not(any(
        Py_GIL_DISABLED,
        Py_LIMITED_API,
        all(py_sys_config = "Py_REF_DEBUG", not(Py_3_12)),
        GraalPy
    )))]
    {
        #[cfg(Py_3_12)]
        if _Py_IsImmortal(op) != 0 {
            return;
        }

        // Skipped _Py_DECREF_STAT_INC - if anyone needs this, please file an issue
        // or submit a PR supporting Py_STATS build option and pystats.h

        #[cfg(py_sys_config = "Py_REF_DEBUG")]
        _Py_DECREF_DecRefTotal();

        #[cfg(Py_3_12)]
        {
            (*op).ob_refcnt.ob_refcnt -= 1;

            #[cfg(py_sys_config = "Py_REF_DEBUG")]
            if (*op).ob_refcnt.ob_refcnt < 0 {
                let location = std::panic::Location::caller();
                let filename = std::ffi::CString::new(location.file()).unwrap();
                _Py_NegativeRefcount(filename.as_ptr(), location.line() as i32, op);
            }

            if (*op).ob_refcnt.ob_refcnt == 0 {
                _Py_Dealloc(op);
            }
        }

        #[cfg(not(Py_3_12))]
        {
            (*op).ob_refcnt -= 1;

            if (*op).ob_refcnt == 0 {
                _Py_Dealloc(op);
            }
        }
    }
}

#[inline]
pub unsafe fn Py_CLEAR(op: *mut *mut PyObject) {
    let tmp = *op;
    if !tmp.is_null() {
        *op = ptr::null_mut();
        Py_DECREF(tmp);
    }
}

#[inline]
pub unsafe fn Py_XINCREF(op: *mut PyObject) {
    if !op.is_null() {
        Py_INCREF(op)
    }
}

#[inline]
pub unsafe fn Py_XDECREF(op: *mut PyObject) {
    if !op.is_null() {
        Py_DECREF(op)
    }
}

extern "C" {
    #[cfg(all(Py_3_10, Py_LIMITED_API, not(PyPy)))]
    #[cfg_attr(docsrs, doc(cfg(Py_3_10)))]
    pub fn Py_NewRef(obj: *mut PyObject) -> *mut PyObject;
    #[cfg(all(Py_3_10, Py_LIMITED_API, not(PyPy)))]
    #[cfg_attr(docsrs, doc(cfg(Py_3_10)))]
    pub fn Py_XNewRef(obj: *mut PyObject) -> *mut PyObject;
}

// macro _Py_NewRef not public; reimplemented directly inside Py_NewRef here
// macro _Py_XNewRef not public; reimplemented directly inside Py_XNewRef here

#[cfg(all(Py_3_10, any(not(Py_LIMITED_API), PyPy)))]
#[cfg_attr(docsrs, doc(cfg(Py_3_10)))]
#[inline]
pub unsafe fn Py_NewRef(obj: *mut PyObject) -> *mut PyObject {
    Py_INCREF(obj);
    obj
}

#[cfg(all(Py_3_10, any(not(Py_LIMITED_API), PyPy)))]
#[cfg_attr(docsrs, doc(cfg(Py_3_10)))]
#[inline]
pub unsafe fn Py_XNewRef(obj: *mut PyObject) -> *mut PyObject {
    Py_XINCREF(obj);
    obj
}
