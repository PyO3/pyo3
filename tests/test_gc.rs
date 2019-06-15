use pyo3::class::PyGCProtocol;
use pyo3::class::PyTraverseError;
use pyo3::class::PyVisit;
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::py_run;
use pyo3::types::PyAny;
use pyo3::types::PyTuple;
use pyo3::AsPyPointer;
use pyo3::PyRawObject;
use std::cell::RefCell;
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

#[pyclass]
struct ClassWithDrop {}

impl Drop for ClassWithDrop {
    fn drop(&mut self) {
        unsafe {
            let py = Python::assume_gil_acquired();

            let _empty1: Py<PyTuple> = FromPy::from_py(PyTuple::empty(py), py);
            let _empty2 = PyTuple::empty(py).into_object(py);
            let _empty3: &PyAny = py.from_owned_ptr(ffi::PyTuple_New(0));
        }
    }
}

// Test behavior of pythonrun::register_pointers + type_object::dealloc
#[test]
fn create_pointers_in_drop() {
    let _gil = Python::acquire_gil();

    let ptr;
    let cnt;
    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let empty = PyTuple::empty(py).into_object(py);
        ptr = empty.as_ptr();
        // substract 2, because `PyTuple::empty(py).into_object(py)` increases the refcnt by 2
        cnt = empty.get_refcnt() - 2;
        let inst = Py::new(py, ClassWithDrop {}).unwrap();
        drop(inst);
    }

    // empty1 and empty2 are still alive (stored in pointers list)
    {
        let _gil = Python::acquire_gil();
        assert_eq!(cnt + 2, unsafe { ffi::Py_REFCNT(ptr) });
    }

    // empty1 and empty2 should be released
    {
        let _gil = Python::acquire_gil();
        assert_eq!(cnt, unsafe { ffi::Py_REFCNT(ptr) });
    }
}

#[allow(dead_code)]
#[pyclass]
struct GCIntegration {
    self_ref: RefCell<PyObject>,
    dropped: TestDropCall,
}

#[pyproto]
impl PyGCProtocol for GCIntegration {
    fn __traverse__(&self, visit: PyVisit) -> Result<(), PyTraverseError> {
        visit.call(&*self.self_ref.borrow())
    }

    fn __clear__(&mut self) {
        let gil = GILGuard::acquire();
        *self.self_ref.borrow_mut() = gil.python().None();
    }
}

#[test]
fn gc_integration() {
    let drop_called = Arc::new(AtomicBool::new(false));

    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let inst = PyRef::new(
            py,
            GCIntegration {
                self_ref: RefCell::new(py.None()),
                dropped: TestDropCall {
                    drop_called: Arc::clone(&drop_called),
                },
            },
        )
        .unwrap();

        *inst.self_ref.borrow_mut() = inst.to_object(py);
    }

    let gil = Python::acquire_gil();
    let py = gil.python();
    py.run("import gc; gc.collect()", None, None).unwrap();
    assert!(drop_called.load(Ordering::Relaxed));
}

#[pyclass(gc)]
struct GCIntegration2 {}

#[test]
fn gc_integration2() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    // Temporarily disable pythons garbage collector to avoid a race condition
    py.run("import gc; gc.disable()", None, None).unwrap();
    let inst = PyRef::new(py, GCIntegration2 {}).unwrap();
    py_run!(py, inst, "assert inst in gc.get_objects()");
    py.run("gc.enable()", None, None).unwrap();
}

#[pyclass(weakref)]
struct WeakRefSupport {}

#[test]
fn weakref_support() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = PyRef::new(py, WeakRefSupport {}).unwrap();
    py_run!(
        py,
        inst,
        "import weakref; assert weakref.ref(inst)() is inst"
    );
}

#[pyclass]
struct BaseClassWithDrop {
    data: Option<Arc<AtomicBool>>,
}

#[pymethods]
impl BaseClassWithDrop {
    #[new]
    fn new(obj: &PyRawObject) {
        obj.init(BaseClassWithDrop { data: None })
    }
}

impl Drop for BaseClassWithDrop {
    fn drop(&mut self) {
        if let Some(ref mut data) = self.data {
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
    fn new(obj: &PyRawObject) {
        obj.init(SubClassWithDrop { data: None });
        BaseClassWithDrop::new(obj);
    }
}

impl Drop for SubClassWithDrop {
    fn drop(&mut self) {
        if let Some(ref mut data) = self.data {
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

        let obj = SubClassWithDrop::try_from_mut(inst).unwrap();
        obj.data = Some(Arc::clone(&drop_called1));

        let base: &mut <SubClassWithDrop as pyo3::PyTypeInfo>::BaseType =
            unsafe { py.mut_from_borrowed_ptr(inst.as_ptr()) };
        base.data = Some(Arc::clone(&drop_called2));
    }

    assert!(drop_called1.load(Ordering::Relaxed));
    assert!(drop_called2.load(Ordering::Relaxed));
}
