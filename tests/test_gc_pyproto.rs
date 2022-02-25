#![cfg(feature = "macros")]
#![cfg(feature = "pyproto")]
#![allow(deprecated)]

use pyo3::class::PyGCProtocol;
use pyo3::class::PyTraverseError;
use pyo3::class::PyVisit;
use pyo3::prelude::*;
use pyo3::type_object::PyTypeObject;
use pyo3::{py_run, AsPyPointer, PyCell, PyTryInto};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod common;

#[pyclass(freelist = 2)]
struct ClassWithFreelist {}

#[test]
fn class_with_freelist() {
    let ptr;
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let inst = Py::new(py, ClassWithFreelist {}).unwrap();
        let _inst2 = Py::new(py, ClassWithFreelist {}).unwrap();
        ptr = inst.as_ptr();
        drop(inst);
    }

    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let inst3 = Py::new(py, ClassWithFreelist {}).unwrap();
        assert_eq!(ptr, inst3.as_ptr());

        let inst4 = Py::new(py, ClassWithFreelist {}).unwrap();
        assert_ne!(ptr, inst4.as_ptr())
    }
}

struct TestDropCall {
    drop_called: Arc<AtomicBool>,
}

impl Drop for TestDropCall {
    fn drop(&mut self) {
        self.drop_called.store(true, Ordering::Relaxed);
    }
}

#[allow(dead_code)]
#[pyclass]
struct DataIsDropped {
    member1: TestDropCall,
    member2: TestDropCall,
}

#[test]
fn data_is_dropped() {
    let drop_called1 = Arc::new(AtomicBool::new(false));
    let drop_called2 = Arc::new(AtomicBool::new(false));

    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let data_is_dropped = DataIsDropped {
            member1: TestDropCall {
                drop_called: Arc::clone(&drop_called1),
            },
            member2: TestDropCall {
                drop_called: Arc::clone(&drop_called2),
            },
        };
        let inst = Py::new(py, data_is_dropped).unwrap();
        assert!(!drop_called1.load(Ordering::Relaxed));
        assert!(!drop_called2.load(Ordering::Relaxed));
        drop(inst);
    }

    assert!(drop_called1.load(Ordering::Relaxed));
    assert!(drop_called2.load(Ordering::Relaxed));
}

#[allow(dead_code)]
#[pyclass]
struct GcIntegration {
    self_ref: PyObject,
    dropped: TestDropCall,
}

#[pyproto]
impl PyGCProtocol for GcIntegration {
    fn __traverse__(&self, visit: PyVisit) -> Result<(), PyTraverseError> {
        visit.call(&self.self_ref)
    }

    fn __clear__(&mut self) {
        let gil = Python::acquire_gil();
        self.self_ref = gil.python().None();
    }
}

#[test]
fn gc_integration() {
    let drop_called = Arc::new(AtomicBool::new(false));

    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let inst = PyCell::new(
            py,
            GcIntegration {
                self_ref: py.None(),
                dropped: TestDropCall {
                    drop_called: Arc::clone(&drop_called),
                },
            },
        )
        .unwrap();

        let mut borrow = inst.borrow_mut();
        borrow.self_ref = inst.to_object(py);
    }

    let gil = Python::acquire_gil();
    let py = gil.python();
    py.run("import gc; gc.collect()", None, None).unwrap();
    assert!(drop_called.load(Ordering::Relaxed));
}

#[pyclass]
struct GcIntegration2 {}

#[pyproto]
impl PyGCProtocol for GcIntegration2 {
    fn __traverse__(&self, _visit: PyVisit) -> Result<(), PyTraverseError> {
        Ok(())
    }
    fn __clear__(&mut self) {}
}

#[test]
fn gc_integration2() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = PyCell::new(py, GcIntegration2 {}).unwrap();
    py_run!(py, inst, "import gc; assert inst in gc.get_objects()");
}

#[pyclass(subclass)]
struct BaseClassWithDrop {
    data: Option<Arc<AtomicBool>>,
}

#[pymethods]
impl BaseClassWithDrop {
    #[new]
    fn new() -> BaseClassWithDrop {
        BaseClassWithDrop { data: None }
    }
}

impl Drop for BaseClassWithDrop {
    fn drop(&mut self) {
        if let Some(data) = &self.data {
            data.store(true, Ordering::Relaxed);
        }
    }
}

#[pyclass(extends = BaseClassWithDrop)]
struct SubClassWithDrop {
    data: Option<Arc<AtomicBool>>,
}

#[pymethods]
impl SubClassWithDrop {
    #[new]
    fn new() -> (Self, BaseClassWithDrop) {
        (
            SubClassWithDrop { data: None },
            BaseClassWithDrop { data: None },
        )
    }
}

impl Drop for SubClassWithDrop {
    fn drop(&mut self) {
        if let Some(data) = &self.data {
            data.store(true, Ordering::Relaxed);
        }
    }
}

#[test]
fn inheritance_with_new_methods_with_drop() {
    let drop_called1 = Arc::new(AtomicBool::new(false));
    let drop_called2 = Arc::new(AtomicBool::new(false));

    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let _typebase = py.get_type::<BaseClassWithDrop>();
        let typeobj = py.get_type::<SubClassWithDrop>();
        let inst = typeobj.call((), None).unwrap();

        let obj: &PyCell<SubClassWithDrop> = inst.try_into().unwrap();
        let mut obj_ref_mut = obj.borrow_mut();
        obj_ref_mut.data = Some(Arc::clone(&drop_called1));
        let base: &mut BaseClassWithDrop = obj_ref_mut.as_mut();
        base.data = Some(Arc::clone(&drop_called2));
    }

    assert!(drop_called1.load(Ordering::Relaxed));
    assert!(drop_called2.load(Ordering::Relaxed));
}

#[pyclass]
struct TraversableClass {
    traversed: AtomicBool,
}

impl TraversableClass {
    fn new() -> Self {
        Self {
            traversed: AtomicBool::new(false),
        }
    }
}

#[pyproto]
impl PyGCProtocol for TraversableClass {
    fn __clear__(&mut self) {}
    fn __traverse__(&self, _visit: PyVisit) -> Result<(), PyTraverseError> {
        self.traversed.store(true, Ordering::Relaxed);
        Ok(())
    }
}

unsafe fn get_type_traverse(tp: *mut pyo3::ffi::PyTypeObject) -> Option<pyo3::ffi::traverseproc> {
    std::mem::transmute(pyo3::ffi::PyType_GetSlot(tp, pyo3::ffi::Py_tp_traverse))
}

#[test]
fn gc_during_borrow() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    unsafe {
        // declare a dummy visitor function
        extern "C" fn novisit(
            _object: *mut pyo3::ffi::PyObject,
            _arg: *mut core::ffi::c_void,
        ) -> std::os::raw::c_int {
            0
        }

        // get the traverse function
        let ty = TraversableClass::type_object(py).as_type_ptr();
        let traverse = get_type_traverse(ty).unwrap();

        // create an object and check that traversing it works normally
        // when it's not borrowed
        let cell = PyCell::new(py, TraversableClass::new()).unwrap();
        let obj = cell.to_object(py);
        assert!(!cell.borrow().traversed.load(Ordering::Relaxed));
        traverse(obj.as_ptr(), novisit, std::ptr::null_mut());
        assert!(cell.borrow().traversed.load(Ordering::Relaxed));

        // create an object and check that it is not traversed if the GC
        // is invoked while it is already borrowed mutably
        let cell2 = PyCell::new(py, TraversableClass::new()).unwrap();
        let obj2 = cell2.to_object(py);
        let guard = cell2.borrow_mut();
        assert!(!guard.traversed.load(Ordering::Relaxed));
        traverse(obj2.as_ptr(), novisit, std::ptr::null_mut());
        assert!(!guard.traversed.load(Ordering::Relaxed));
        drop(guard);
    }
}
