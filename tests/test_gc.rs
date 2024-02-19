#![cfg(feature = "macros")]

use pyo3::class::PyTraverseError;
use pyo3::class::PyVisit;
use pyo3::prelude::*;
use pyo3::{py_run, PyCell};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[path = "../src/tests/common.rs"]
mod common;

#[pyclass(freelist = 2)]
struct ClassWithFreelist {}

#[test]
fn class_with_freelist() {
    let ptr = Python::with_gil(|py| {
        let inst = Py::new(py, ClassWithFreelist {}).unwrap();
        let _inst2 = Py::new(py, ClassWithFreelist {}).unwrap();
        let ptr = inst.as_ptr();
        drop(inst);
        ptr
    });

    Python::with_gil(|py| {
        let inst3 = Py::new(py, ClassWithFreelist {}).unwrap();
        assert_eq!(ptr, inst3.as_ptr());

        let inst4 = Py::new(py, ClassWithFreelist {}).unwrap();
        assert_ne!(ptr, inst4.as_ptr())
    });
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

    Python::with_gil(|py| {
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
    });

    assert!(drop_called1.load(Ordering::Relaxed));
    assert!(drop_called2.load(Ordering::Relaxed));
}

#[allow(dead_code)]
#[pyclass]
struct GcIntegration {
    self_ref: PyObject,
    dropped: TestDropCall,
}

#[pymethods]
impl GcIntegration {
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        visit.call(&self.self_ref)
    }

    fn __clear__(&mut self) {
        Python::with_gil(|py| {
            self.self_ref = py.None();
        });
    }
}

#[test]
fn gc_integration() {
    let drop_called = Arc::new(AtomicBool::new(false));

    Python::with_gil(|py| {
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

        py_run!(py, inst, "import gc; assert inst in gc.get_objects()");
    });

    Python::with_gil(|py| {
        py.run_bound("import gc; gc.collect()", None, None).unwrap();
        assert!(drop_called.load(Ordering::Relaxed));
    });
}

#[pyclass]
struct GcNullTraversal {
    cycle: Option<Py<Self>>,
    null: Option<Py<Self>>,
}

#[pymethods]
impl GcNullTraversal {
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        visit.call(&self.cycle)?;
        visit.call(&self.null)?; // Should not segfault
        Ok(())
    }

    fn __clear__(&mut self) {
        self.cycle = None;
        self.null = None;
    }
}

#[test]
fn gc_null_traversal() {
    Python::with_gil(|py| {
        let obj = Py::new(
            py,
            GcNullTraversal {
                cycle: None,
                null: None,
            },
        )
        .unwrap();
        obj.borrow_mut(py).cycle = Some(obj.clone_ref(py));

        // the object doesn't have to be cleaned up, it just needs to be traversed.
        py.run_bound("import gc; gc.collect()", None, None).unwrap();
    });
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

    Python::with_gil(|py| {
        let _typebase = py.get_type_bound::<BaseClassWithDrop>();
        let typeobj = py.get_type_bound::<SubClassWithDrop>();
        let inst = typeobj.call((), None).unwrap();

        let obj = inst.downcast::<SubClassWithDrop>().unwrap();
        let mut obj_ref_mut = obj.borrow_mut();
        obj_ref_mut.data = Some(Arc::clone(&drop_called1));
        let base: &mut BaseClassWithDrop = obj_ref_mut.as_mut();
        base.data = Some(Arc::clone(&drop_called2));
    });

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

#[pymethods]
impl TraversableClass {
    fn __clear__(&mut self) {}

    #[allow(clippy::unnecessary_wraps)]
    fn __traverse__(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        self.traversed.store(true, Ordering::Relaxed);
        Ok(())
    }
}

#[test]
fn gc_during_borrow() {
    Python::with_gil(|py| {
        unsafe {
            // get the traverse function
            let ty = py.get_type_bound::<TraversableClass>();
            let traverse = get_type_traverse(ty.as_type_ptr()).unwrap();

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
    });
}

#[pyclass]
struct PartialTraverse {
    member: PyObject,
}

impl PartialTraverse {
    fn new(py: Python<'_>) -> Self {
        Self { member: py.None() }
    }
}

#[pymethods]
impl PartialTraverse {
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        visit.call(&self.member)?;
        // In the test, we expect this to never be hit
        unreachable!()
    }
}

#[test]
fn traverse_partial() {
    Python::with_gil(|py| unsafe {
        // get the traverse function
        let ty = py.get_type_bound::<PartialTraverse>();
        let traverse = get_type_traverse(ty.as_type_ptr()).unwrap();

        // confirm that traversing errors
        let obj = Py::new(py, PartialTraverse::new(py)).unwrap();
        assert_eq!(
            traverse(obj.as_ptr(), visit_error, std::ptr::null_mut()),
            -1
        );
    })
}

#[pyclass]
struct PanickyTraverse {
    member: PyObject,
}

impl PanickyTraverse {
    fn new(py: Python<'_>) -> Self {
        Self { member: py.None() }
    }
}

#[pymethods]
impl PanickyTraverse {
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        visit.call(&self.member)?;
        panic!("at the disco");
    }
}

#[test]
fn traverse_panic() {
    Python::with_gil(|py| unsafe {
        // get the traverse function
        let ty = py.get_type_bound::<PanickyTraverse>();
        let traverse = get_type_traverse(ty.as_type_ptr()).unwrap();

        // confirm that traversing errors
        let obj = Py::new(py, PanickyTraverse::new(py)).unwrap();
        assert_eq!(traverse(obj.as_ptr(), novisit, std::ptr::null_mut()), -1);
    })
}

#[pyclass]
struct TriesGILInTraverse {}

#[pymethods]
impl TriesGILInTraverse {
    fn __traverse__(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        Python::with_gil(|_py| Ok(()))
    }
}

#[test]
fn tries_gil_in_traverse() {
    Python::with_gil(|py| unsafe {
        // get the traverse function
        let ty = py.get_type_bound::<TriesGILInTraverse>();
        let traverse = get_type_traverse(ty.as_type_ptr()).unwrap();

        // confirm that traversing panicks
        let obj = Py::new(py, TriesGILInTraverse {}).unwrap();
        assert_eq!(traverse(obj.as_ptr(), novisit, std::ptr::null_mut()), -1);
    })
}

#[pyclass]
struct HijackedTraverse {
    traversed: Cell<bool>,
    hijacked: Cell<bool>,
}

impl HijackedTraverse {
    fn new() -> Self {
        Self {
            traversed: Cell::new(false),
            hijacked: Cell::new(false),
        }
    }

    fn traversed_and_hijacked(&self) -> (bool, bool) {
        (self.traversed.get(), self.hijacked.get())
    }
}

#[pymethods]
impl HijackedTraverse {
    #[allow(clippy::unnecessary_wraps)]
    fn __traverse__(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        self.traversed.set(true);
        Ok(())
    }
}

trait Traversable {
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError>;
}

impl<'a> Traversable for PyRef<'a, HijackedTraverse> {
    fn __traverse__(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        self.hijacked.set(true);
        Ok(())
    }
}

#[test]
fn traverse_cannot_be_hijacked() {
    Python::with_gil(|py| unsafe {
        // get the traverse function
        let ty = py.get_type_bound::<HijackedTraverse>();
        let traverse = get_type_traverse(ty.as_type_ptr()).unwrap();

        let cell = PyCell::new(py, HijackedTraverse::new()).unwrap();
        let obj = cell.to_object(py);
        assert_eq!(cell.borrow().traversed_and_hijacked(), (false, false));
        traverse(obj.as_ptr(), novisit, std::ptr::null_mut());
        assert_eq!(cell.borrow().traversed_and_hijacked(), (true, false));
    })
}

#[allow(dead_code)]
#[pyclass]
struct DropDuringTraversal {
    cycle: Cell<Option<Py<Self>>>,
    dropped: TestDropCall,
}

#[pymethods]
impl DropDuringTraversal {
    #[allow(clippy::unnecessary_wraps)]
    fn __traverse__(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        self.cycle.take();
        Ok(())
    }

    fn __clear__(&mut self) {
        self.cycle.take();
    }
}

#[test]
fn drop_during_traversal_with_gil() {
    let drop_called = Arc::new(AtomicBool::new(false));

    Python::with_gil(|py| {
        let inst = Py::new(
            py,
            DropDuringTraversal {
                cycle: Cell::new(None),
                dropped: TestDropCall {
                    drop_called: Arc::clone(&drop_called),
                },
            },
        )
        .unwrap();

        inst.borrow_mut(py).cycle.set(Some(inst.clone_ref(py)));

        drop(inst);
    });

    // due to the internal GC mechanism, we may need multiple
    // (but not too many) collections to get `inst` actually dropped.
    for _ in 0..10 {
        Python::with_gil(|py| {
            py.run_bound("import gc; gc.collect()", None, None).unwrap();
        });
    }
    assert!(drop_called.load(Ordering::Relaxed));
}

#[test]
fn drop_during_traversal_without_gil() {
    let drop_called = Arc::new(AtomicBool::new(false));

    let inst = Python::with_gil(|py| {
        let inst = Py::new(
            py,
            DropDuringTraversal {
                cycle: Cell::new(None),
                dropped: TestDropCall {
                    drop_called: Arc::clone(&drop_called),
                },
            },
        )
        .unwrap();

        inst.borrow_mut(py).cycle.set(Some(inst.clone_ref(py)));

        inst
    });

    drop(inst);

    // due to the internal GC mechanism, we may need multiple
    // (but not too many) collections to get `inst` actually dropped.
    for _ in 0..10 {
        Python::with_gil(|py| {
            py.run_bound("import gc; gc.collect()", None, None).unwrap();
        });
    }
    assert!(drop_called.load(Ordering::Relaxed));
}

#[pyclass(unsendable)]
struct UnsendableTraversal {
    traversed: Cell<bool>,
}

#[pymethods]
impl UnsendableTraversal {
    fn __clear__(&mut self) {}

    #[allow(clippy::unnecessary_wraps)]
    fn __traverse__(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        self.traversed.set(true);
        Ok(())
    }
}

#[test]
#[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
fn unsendable_are_not_traversed_on_foreign_thread() {
    Python::with_gil(|py| unsafe {
        let ty = py.get_type_bound::<UnsendableTraversal>();
        let traverse = get_type_traverse(ty.as_type_ptr()).unwrap();

        let obj = Py::new(
            py,
            UnsendableTraversal {
                traversed: Cell::new(false),
            },
        )
        .unwrap();

        let ptr = SendablePtr(obj.as_ptr());

        std::thread::spawn(move || {
            // traversal on foreign thread is a no-op
            assert_eq!(traverse({ ptr }.0, novisit, std::ptr::null_mut()), 0);
        })
        .join()
        .unwrap();

        assert!(!obj.borrow(py).traversed.get());

        // traversal on home thread still works
        assert_eq!(traverse({ ptr }.0, novisit, std::ptr::null_mut()), 0);

        assert!(obj.borrow(py).traversed.get());
    });
}

// Manual traversal utilities

unsafe fn get_type_traverse(tp: *mut pyo3::ffi::PyTypeObject) -> Option<pyo3::ffi::traverseproc> {
    std::mem::transmute(pyo3::ffi::PyType_GetSlot(tp, pyo3::ffi::Py_tp_traverse))
}

// a dummy visitor function
extern "C" fn novisit(
    _object: *mut pyo3::ffi::PyObject,
    _arg: *mut core::ffi::c_void,
) -> std::os::raw::c_int {
    0
}

// a visitor function which errors (returns nonzero code)
extern "C" fn visit_error(
    _object: *mut pyo3::ffi::PyObject,
    _arg: *mut core::ffi::c_void,
) -> std::os::raw::c_int {
    -1
}

#[derive(Clone, Copy)]
struct SendablePtr(*mut pyo3::ffi::PyObject);

unsafe impl Send for SendablePtr {}
