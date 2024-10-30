#![cfg(feature = "macros")]

use pyo3::class::PyTraverseError;
use pyo3::class::PyVisit;
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::py_run;
#[cfg(not(target_arch = "wasm32"))]
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;
use std::sync::{Arc, Mutex};

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

/// Helper function to create a pair of objects that can be used to test drops;
/// the first object is a guard that records when it has been dropped, the second
/// object is a check that can be used to assert that the guard has been dropped.
fn drop_check() -> (DropGuard, DropCheck) {
    let flag = Arc::new(Once::new());
    (DropGuard(flag.clone()), DropCheck(flag))
}

/// Helper structure that records when it has been dropped in the cor
struct DropGuard(Arc<Once>);
impl Drop for DropGuard {
    fn drop(&mut self) {
        self.0.call_once(|| ());
    }
}

struct DropCheck(Arc<Once>);
impl DropCheck {
    #[track_caller]
    fn assert_not_dropped(&self) {
        assert!(!self.0.is_completed());
    }

    #[track_caller]
    fn assert_dropped(&self) {
        assert!(self.0.is_completed());
    }

    #[track_caller]
    fn assert_drops_with_gc(&self, object: *mut pyo3::ffi::PyObject) {
        // running the GC might take a few cycles to collect an object
        for _ in 0..100 {
            if self.0.is_completed() {
                return;
            }

            Python::with_gil(|py| {
                py.run(ffi::c_str!("import gc; gc.collect()"), None, None)
                    .unwrap();
            });
            #[cfg(Py_GIL_DISABLED)]
            {
                // on the free-threaded build, the GC might be running in a separate thread, allow
                // some time for this
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }

        panic!(
            "Object was not dropped after 100 GC cycles, refcount is {}",
            // this could be garbage, but it's in a test and we're just printing the value
            unsafe { ffi::Py_REFCNT(object) }
        );
    }
}

#[test]
fn data_is_dropped() {
    #[pyclass]
    struct DataIsDropped {
        _guard1: DropGuard,
        _guard2: DropGuard,
    }

    let (guard1, check1) = drop_check();
    let (guard2, check2) = drop_check();

    Python::with_gil(|py| {
        let data_is_dropped = DataIsDropped {
            _guard1: guard1,
            _guard2: guard2,
        };
        let inst = Py::new(py, data_is_dropped).unwrap();
        check1.assert_not_dropped();
        check2.assert_not_dropped();
        drop(inst);
    });

    check1.assert_dropped();
    check2.assert_dropped();
}

#[pyclass(subclass)]
struct CycleWithClear {
    cycle: Option<PyObject>,
    _guard: DropGuard,
}

#[pymethods]
impl CycleWithClear {
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        visit.call(&self.cycle)
    }

    fn __clear__(slf: &Bound<'_, Self>) {
        println!("clear run, refcount before {}", slf.get_refcnt());
        slf.borrow_mut().cycle = None;
        println!("clear run, refcount after {}", slf.get_refcnt());
    }
}

#[test]
fn test_cycle_clear() {
    let (guard, check) = drop_check();

    let ptr = Python::with_gil(|py| {
        let inst = Bound::new(
            py,
            CycleWithClear {
                cycle: None,
                _guard: guard,
            },
        )
        .unwrap();

        inst.borrow_mut().cycle = Some(inst.clone().into_any().unbind());

        py_run!(py, inst, "import gc; assert inst in gc.get_objects()");
        check.assert_not_dropped();
        inst.as_ptr()
    });

    check.assert_drops_with_gc(ptr);
}

/// Test that traversing `None` of `Option<Py<T>>` does not cause a segfault
#[test]
fn gc_null_traversal() {
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
        py.run(ffi::c_str!("import gc; gc.collect()"), None, None)
            .unwrap();
    });
}

#[test]
fn inheritance_with_new_methods_with_drop() {
    #[pyclass(subclass)]
    struct BaseClassWithDrop {
        guard: Option<DropGuard>,
    }

    #[pymethods]
    impl BaseClassWithDrop {
        #[new]
        fn new() -> BaseClassWithDrop {
            BaseClassWithDrop { guard: None }
        }
    }

    #[pyclass(extends = BaseClassWithDrop)]
    struct SubClassWithDrop {
        guard: Option<DropGuard>,
    }

    #[pymethods]
    impl SubClassWithDrop {
        #[new]
        fn new() -> (Self, BaseClassWithDrop) {
            (
                SubClassWithDrop { guard: None },
                BaseClassWithDrop { guard: None },
            )
        }
    }

    let (guard_base, check_base) = drop_check();
    let (guard_sub, check_sub) = drop_check();

    Python::with_gil(|py| {
        let typeobj = py.get_type::<SubClassWithDrop>();
        let inst = typeobj
            .call((), None)
            .unwrap()
            .downcast_into::<SubClassWithDrop>()
            .unwrap();

        inst.as_super().borrow_mut().guard = Some(guard_base);
        inst.borrow_mut().guard = Some(guard_sub);

        check_base.assert_not_dropped();
        check_sub.assert_not_dropped();
    });

    check_base.assert_dropped();
    check_sub.assert_dropped();
}

#[test]
fn gc_during_borrow() {
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

    Python::with_gil(|py| {
        // get the traverse function
        let ty = py.get_type::<TraversableClass>();
        let traverse = unsafe { get_type_traverse(ty.as_type_ptr()).unwrap() };

        // create an object and check that traversing it works normally
        // when it's not borrowed
        let cell = Bound::new(py, TraversableClass::new()).unwrap();
        assert!(!cell.borrow().traversed.load(Ordering::Relaxed));
        unsafe { traverse(cell.as_ptr(), novisit, std::ptr::null_mut()) };
        assert!(cell.borrow().traversed.load(Ordering::Relaxed));

        // create an object and check that it is not traversed if the GC
        // is invoked while it is already borrowed mutably
        let cell2 = Bound::new(py, TraversableClass::new()).unwrap();
        let guard = cell2.borrow_mut();
        assert!(!guard.traversed.load(Ordering::Relaxed));
        unsafe { traverse(cell2.as_ptr(), novisit, std::ptr::null_mut()) };
        assert!(!guard.traversed.load(Ordering::Relaxed));
        drop(guard);
    });
}

#[test]
fn traverse_partial() {
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

    Python::with_gil(|py| {
        // get the traverse function
        let ty = py.get_type::<PartialTraverse>();
        let traverse = unsafe { get_type_traverse(ty.as_type_ptr()).unwrap() };

        // confirm that traversing errors
        let obj = Py::new(py, PartialTraverse::new(py)).unwrap();
        assert_eq!(
            unsafe { traverse(obj.as_ptr(), visit_error, std::ptr::null_mut()) },
            -1
        );
    })
}

#[test]
fn traverse_panic() {
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

    Python::with_gil(|py| {
        // get the traverse function
        let ty = py.get_type::<PanickyTraverse>();
        let traverse = unsafe { get_type_traverse(ty.as_type_ptr()).unwrap() };

        // confirm that traversing errors
        let obj = Py::new(py, PanickyTraverse::new(py)).unwrap();
        assert_eq!(
            unsafe { traverse(obj.as_ptr(), novisit, std::ptr::null_mut()) },
            -1
        );
    })
}

#[test]
fn tries_gil_in_traverse() {
    #[pyclass]
    struct TriesGILInTraverse {}

    #[pymethods]
    impl TriesGILInTraverse {
        fn __traverse__(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
            Python::with_gil(|_py| Ok(()))
        }
    }

    Python::with_gil(|py| {
        // get the traverse function
        let ty = py.get_type::<TriesGILInTraverse>();
        let traverse = unsafe { get_type_traverse(ty.as_type_ptr()).unwrap() };

        // confirm that traversing panicks
        let obj = Py::new(py, TriesGILInTraverse {}).unwrap();
        assert_eq!(
            unsafe { traverse(obj.as_ptr(), novisit, std::ptr::null_mut()) },
            -1
        );
    })
}

#[test]
fn traverse_cannot_be_hijacked() {
    #[pyclass]
    struct HijackedTraverse {
        traversed: AtomicBool,
        hijacked: AtomicBool,
    }

    impl HijackedTraverse {
        fn new() -> Self {
            Self {
                traversed: AtomicBool::new(false),
                hijacked: AtomicBool::new(false),
            }
        }

        fn traversed_and_hijacked(&self) -> (bool, bool) {
            (
                self.traversed.load(Ordering::Acquire),
                self.hijacked.load(Ordering::Acquire),
            )
        }
    }

    #[pymethods]
    impl HijackedTraverse {
        #[allow(clippy::unnecessary_wraps)]
        fn __traverse__(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
            self.traversed.store(true, Ordering::Release);
            Ok(())
        }
    }

    #[allow(dead_code)]
    trait Traversable {
        fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError>;
    }

    impl Traversable for PyRef<'_, HijackedTraverse> {
        fn __traverse__(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
            self.hijacked.store(true, Ordering::Release);
            Ok(())
        }
    }

    Python::with_gil(|py| {
        // get the traverse function
        let ty = py.get_type::<HijackedTraverse>();
        let traverse = unsafe { get_type_traverse(ty.as_type_ptr()).unwrap() };

        let cell = Bound::new(py, HijackedTraverse::new()).unwrap();
        assert_eq!(cell.borrow().traversed_and_hijacked(), (false, false));
        unsafe { traverse(cell.as_ptr(), novisit, std::ptr::null_mut()) };
        assert_eq!(cell.borrow().traversed_and_hijacked(), (true, false));
    })
}

#[pyclass]
struct DropDuringTraversal {
    cycle: Mutex<Option<Py<Self>>>,
    _guard: DropGuard,
}

#[pymethods]
impl DropDuringTraversal {
    #[allow(clippy::unnecessary_wraps)]
    fn __traverse__(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        let mut cycle_ref = self.cycle.lock().unwrap();
        *cycle_ref = None;
        Ok(())
    }
}

#[cfg(not(pyo3_disable_reference_pool))]
#[test]
fn drop_during_traversal_with_gil() {
    let (guard, check) = drop_check();

    let ptr = Python::with_gil(|py| {
        let cycle = Mutex::new(None);
        let inst = Py::new(
            py,
            DropDuringTraversal {
                cycle,
                _guard: guard,
            },
        )
        .unwrap();

        *inst.borrow_mut(py).cycle.lock().unwrap() = Some(inst.clone_ref(py));

        check.assert_not_dropped();
        let ptr = inst.as_ptr();
        drop(inst); // drop the object while holding the GIL

        #[cfg(not(Py_GIL_DISABLED))]
        {
            // other thread might have caused GC on free-threaded build
            check.assert_not_dropped();
        }

        ptr
    });

    check.assert_drops_with_gc(ptr);
}

#[cfg(not(pyo3_disable_reference_pool))]
#[test]
fn drop_during_traversal_without_gil() {
    let (guard, check) = drop_check();

    let inst = Python::with_gil(|py| {
        let cycle = Mutex::new(None);
        let inst = Py::new(
            py,
            DropDuringTraversal {
                cycle,
                _guard: guard,
            },
        )
        .unwrap();

        *inst.borrow_mut(py).cycle.lock().unwrap() = Some(inst.clone_ref(py));

        check.assert_not_dropped();
        inst
    });

    let ptr = inst.as_ptr();
    drop(inst); // drop the object without holding the GIL

    check.assert_drops_with_gc(ptr);
}

#[test]
#[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
fn unsendable_are_not_traversed_on_foreign_thread() {
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

    #[derive(Clone, Copy)]
    struct SendablePtr(*mut pyo3::ffi::PyObject);

    unsafe impl Send for SendablePtr {}

    Python::with_gil(|py| {
        let ty = py.get_type::<UnsendableTraversal>();
        let traverse = unsafe { get_type_traverse(ty.as_type_ptr()).unwrap() };

        let obj = Bound::new(
            py,
            UnsendableTraversal {
                traversed: Cell::new(false),
            },
        )
        .unwrap();

        let ptr = SendablePtr(obj.as_ptr());

        std::thread::spawn(move || {
            // traversal on foreign thread is a no-op
            assert_eq!(
                unsafe { traverse({ ptr }.0, novisit, std::ptr::null_mut()) },
                0
            );
        })
        .join()
        .unwrap();

        assert!(!obj.borrow().traversed.get());

        // traversal on home thread still works
        assert_eq!(
            unsafe { traverse({ ptr }.0, novisit, std::ptr::null_mut()) },
            0
        );

        assert!(obj.borrow().traversed.get());
    });
}

#[test]
fn test_traverse_subclass() {
    #[pyclass(extends = CycleWithClear)]
    struct SubOverrideTraverse {}

    #[pymethods]
    impl SubOverrideTraverse {
        #[allow(clippy::unnecessary_wraps)]
        fn __traverse__(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
            // subclass traverse overrides the base class traverse
            Ok(())
        }
    }

    let (guard, check) = drop_check();

    Python::with_gil(|py| {
        let base = CycleWithClear {
            cycle: None,
            _guard: guard,
        };
        let obj = Bound::new(
            py,
            PyClassInitializer::from(base).add_subclass(SubOverrideTraverse {}),
        )
        .unwrap();
        obj.borrow_mut().as_super().cycle = Some(obj.clone().into_any().unbind());

        check.assert_not_dropped();
        let ptr = obj.as_ptr();
        drop(obj);
        #[cfg(not(Py_GIL_DISABLED))]
        {
            // other thread might have caused GC on free-threaded build
            check.assert_not_dropped();
        }

        #[cfg(not(Py_GIL_DISABLED))]
        {
            // FIXME: seems like a bug that this is flaky on the free-threaded build
            // https://github.com/PyO3/pyo3/issues/4627
            check.assert_drops_with_gc(ptr);
        }

        #[cfg(Py_GIL_DISABLED)]
        {
            // silence unused ptr warning
            let _ = ptr;
        }
    });
}

#[test]
fn test_traverse_subclass_override_clear() {
    #[pyclass(extends = CycleWithClear)]
    struct SubOverrideClear {}

    #[pymethods]
    impl SubOverrideClear {
        #[allow(clippy::unnecessary_wraps)]
        fn __traverse__(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
            // subclass traverse overrides the base class traverse, necessary for
            // the sub clear to be called
            // FIXME: should this really need to be the case?
            Ok(())
        }

        fn __clear__(&self) {
            // subclass clear overrides the base class clear
        }
    }

    let (guard, check) = drop_check();

    Python::with_gil(|py| {
        let base = CycleWithClear {
            cycle: None,
            _guard: guard,
        };
        let obj = Bound::new(
            py,
            PyClassInitializer::from(base).add_subclass(SubOverrideClear {}),
        )
        .unwrap();
        obj.borrow_mut().as_super().cycle = Some(obj.clone().into_any().unbind());

        check.assert_not_dropped();
        let ptr = obj.as_ptr();
        drop(obj);
        #[cfg(not(Py_GIL_DISABLED))]
        {
            // other thread might have caused GC on free-threaded build
            check.assert_not_dropped();
        }

        #[cfg(not(Py_GIL_DISABLED))]
        {
            // FIXME: seems like a bug that this is flaky on the free-threaded build
            // https://github.com/PyO3/pyo3/issues/4627
            check.assert_drops_with_gc(ptr);
        }

        #[cfg(Py_GIL_DISABLED)]
        {
            // silence unused ptr warning
            let _ = ptr;
        }
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
