use crate::object::*;
use crate::rustpython_runtime;
use rustpython_vm::builtins::PyType;
use rustpython_vm::object::MaybeTraverse;
use rustpython_vm::{AsObject, Context, Py, PyObjectRef, PyPayload, PyRef};
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};

pub static mut PyCapsule_Type: PyTypeObject = PyTypeObject { _opaque: [] };

pub type PyCapsule_Destructor = unsafe extern "C" fn(o: *mut PyObject);

#[derive(Debug)]
struct PyCapsulePayload {
    pointer: AtomicPtr<c_void>,
    context: AtomicPtr<c_void>,
    name: Mutex<Option<CString>>,
    destructor: Mutex<Option<PyCapsule_Destructor>>,
    self_ptr: AtomicPtr<rustpython_vm::PyObject>,
}

impl PyCapsulePayload {
    fn new(
        pointer: *mut c_void,
        name: Option<CString>,
        destructor: Option<PyCapsule_Destructor>,
    ) -> Self {
        Self {
            pointer: AtomicPtr::new(pointer),
            context: AtomicPtr::new(std::ptr::null_mut()),
            name: Mutex::new(name),
            destructor: Mutex::new(destructor),
            self_ptr: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    fn name_matches(&self, requested: *const c_char) -> bool {
        let stored = self.name.lock().unwrap();
        match (stored.as_ref(), requested.is_null()) {
            (None, true) => true,
            (Some(_), true) => false,
            (None, false) => false,
            (Some(stored), false) => unsafe { CStr::from_ptr(requested).to_bytes() == stored.as_bytes() },
        }
    }
}

impl Drop for PyCapsulePayload {
    fn drop(&mut self) {
        let destructor = self.destructor.lock().unwrap().take();
        let self_ptr = self.self_ptr.load(Ordering::Relaxed);
        if let (Some(destructor), false) = (destructor, self_ptr.is_null()) {
            unsafe { destructor(self_ptr.cast()) };
        }
    }
}

impl MaybeTraverse for PyCapsulePayload {
    fn try_traverse(&self, _traverse_fn: &mut rustpython_vm::object::TraverseFn<'_>) {}
}

impl PyPayload for PyCapsulePayload {
    fn class(ctx: &Context) -> &'static Py<PyType> {
        ctx.types.capsule_type
    }
}

fn capsule_payload<'a>(capsule: &'a PyObjectRef) -> Option<&'a PyCapsulePayload> {
    capsule.downcast_ref::<PyCapsulePayload>().map(|payload| &**payload)
}

#[inline]
pub unsafe fn PyCapsule_CheckExact(ob: *mut PyObject) -> c_int {
    if ob.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    rustpython_runtime::with_vm(|vm| {
        capsule_payload(&obj)
            .is_some_and(|_| obj.class().is(vm.ctx.types.capsule_type))
            .into()
    })
}

#[inline]
pub unsafe fn PyCapsule_New(
    pointer: *mut c_void,
    name: *const c_char,
    destructor: Option<PyCapsule_Destructor>,
) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let name = (!name.is_null()).then(|| CStr::from_ptr(name).to_owned());
        let payload = PyCapsulePayload::new(pointer, name, destructor);
        let capsule: PyRef<PyCapsulePayload> =
            PyRef::new_ref(payload, vm.ctx.types.capsule_type.to_owned(), None);
        capsule
            .self_ptr
            .store(capsule.as_object() as *const _ as *mut _, Ordering::Relaxed);
        pyobject_ref_to_ptr(capsule.into())
    })
}

#[inline]
pub unsafe fn PyCapsule_GetPointer(capsule: *mut PyObject, name: *const c_char) -> *mut c_void {
    if capsule.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(capsule);
    let Some(payload) = capsule_payload(&obj) else {
        return std::ptr::null_mut();
    };
    if !payload.name_matches(name) {
        return std::ptr::null_mut();
    }
    payload.pointer.load(Ordering::Relaxed)
}

#[inline]
pub unsafe fn PyCapsule_GetDestructor(
    capsule: *mut PyObject,
) -> Option<PyCapsule_Destructor> {
    if capsule.is_null() {
        return None;
    }
    let obj = ptr_to_pyobject_ref_borrowed(capsule);
    capsule_payload(&obj).and_then(|payload| *payload.destructor.lock().unwrap())
}

#[inline]
pub unsafe fn PyCapsule_GetName(capsule: *mut PyObject) -> *const c_char {
    if capsule.is_null() {
        return std::ptr::null();
    }
    let obj = ptr_to_pyobject_ref_borrowed(capsule);
    capsule_payload(&obj)
        .and_then(|payload| payload.name.lock().unwrap().as_ref().map(|name| name.as_ptr()))
        .unwrap_or(std::ptr::null())
}

#[inline]
pub unsafe fn PyCapsule_GetContext(capsule: *mut PyObject) -> *mut c_void {
    if capsule.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(capsule);
    capsule_payload(&obj)
        .map(|payload| payload.context.load(Ordering::Relaxed))
        .unwrap_or(std::ptr::null_mut())
}

#[inline]
pub unsafe fn PyCapsule_IsValid(capsule: *mut PyObject, name: *const c_char) -> c_int {
    if capsule.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(capsule);
    capsule_payload(&obj)
        .is_some_and(|payload| {
            !payload.pointer.load(Ordering::Relaxed).is_null() && payload.name_matches(name)
        })
        .into()
}

#[inline]
pub unsafe fn PyCapsule_SetPointer(capsule: *mut PyObject, pointer: *mut c_void) -> c_int {
    if capsule.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(capsule);
    let Some(payload) = capsule_payload(&obj) else {
        return -1;
    };
    payload.pointer.store(pointer, Ordering::Relaxed);
    0
}

#[inline]
pub unsafe fn PyCapsule_SetDestructor(
    capsule: *mut PyObject,
    destructor: Option<PyCapsule_Destructor>,
) -> c_int {
    if capsule.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(capsule);
    let Some(payload) = capsule_payload(&obj) else {
        return -1;
    };
    *payload.destructor.lock().unwrap() = destructor;
    0
}

#[inline]
pub unsafe fn PyCapsule_SetName(capsule: *mut PyObject, name: *const c_char) -> c_int {
    if capsule.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(capsule);
    let Some(payload) = capsule_payload(&obj) else {
        return -1;
    };
    *payload.name.lock().unwrap() = if name.is_null() {
        None
    } else {
        Some(CStr::from_ptr(name).to_owned())
    };
    0
}

#[inline]
pub unsafe fn PyCapsule_SetContext(capsule: *mut PyObject, context: *mut c_void) -> c_int {
    if capsule.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(capsule);
    let Some(payload) = capsule_payload(&obj) else {
        return -1;
    };
    payload.context.store(context, Ordering::Relaxed);
    0
}

#[inline]
pub unsafe fn PyCapsule_Import(name: *const c_char, _no_block: c_int) -> *mut c_void {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    let Ok(path) = CStr::from_ptr(name).to_str() else {
        return std::ptr::null_mut();
    };
    let Some((module_name, attr_name)) = path.rsplit_once('.') else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(|vm| {
        let Ok(module) = vm.import(module_name, 0) else {
            return std::ptr::null_mut();
        };
        let Ok(capsule) = module.get_attr(attr_name, vm) else {
            return std::ptr::null_mut();
        };
        capsule_payload(&capsule)
            .and_then(|payload| {
                payload
                    .name_matches(name)
                    .then_some(payload.pointer.load(Ordering::Relaxed))
            })
            .unwrap_or(std::ptr::null_mut())
    })
}
