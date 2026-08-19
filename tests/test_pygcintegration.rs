#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::{PyTraverseError, PyVisit};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};
use std::marker::PhantomData;
use std::sync::OnceLock;

struct NotTraversable {
    value: i32,
}

#[derive(PyGcTraversable, PartialEq, Eq, PartialOrd, Ord)]
struct Leaf {
    value: i32,
}

impl std::hash::Hash for Leaf {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[derive(PyGcTraversable)]
struct Branch {
    left: Option<Leaf>,
    right: Vec<Leaf>,
}

#[derive(PyGcTraversable)]
struct BranchWithIgnoredField {
    tracked: Option<Leaf>,
    #[pyo3(gc = false)]
    ignored: NotTraversable,
}

#[derive(PyGcTraversable)]
struct Wrappers {
    map: BTreeMap<Leaf, Leaf>,
    set: BTreeSet<Leaf>,
    lock: OnceLock<Leaf>,
    marker: PhantomData<Leaf>,
    result: Result<Leaf, Leaf>,
    array: [Leaf; 2],
    hash_map: HashMap<Leaf, Leaf>,
    hash_set: HashSet<Leaf>,
    vec_deque: VecDeque<Leaf>,
    linked_list: LinkedList<Leaf>,
}

#[derive(PyGcTraversable)]
struct OpaqueWrapper {
    hidden: PyGcOpaque<Py<PyAny>>,
}

#[pyclass]
#[derive(PyGcTraversable)]
struct TraversedByPyClass {
    field: Py<PyAny>,
}

#[pymethods]
impl TraversedByPyClass {
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        PyGcTraversable::traverse(self, visit)
    }

    fn __clear__(&mut self) {
        PyGcTraversable::clear(self);
    }
}

#[derive(PyGcTraversable)]
enum Node {
    #[allow(dead_code)]
    Unit,
    #[allow(dead_code)]
    Tuple(Option<Leaf>, i32),
    #[allow(dead_code)]
    Struct {
        left: Option<Leaf>,
        right: Vec<Leaf>,
    },
}

#[test]
fn may_contain_cycles_structs_and_enums() {
    assert!(!Leaf::MAY_CONTAIN_CYCLES);
    assert!(!Branch::MAY_CONTAIN_CYCLES);
    assert!(!BranchWithIgnoredField::MAY_CONTAIN_CYCLES);
    assert!(!Node::MAY_CONTAIN_CYCLES);
}

#[test]
fn clear_recursively_clears_supported_types() {
    let mut branch = Branch {
        left: Some(Leaf { value: 1 }),
        right: vec![Leaf { value: 2 }],
    };

    PyGcTraversable::clear(&mut branch);
    assert!(branch.left.is_some());
    assert_eq!(branch.right.len(), 1);

    let mut maybe_branch = Some(branch);
    PyGcTraversable::clear(&mut maybe_branch);
    assert!(maybe_branch.is_some());
}

#[test]
fn clear_ignored_field_is_noop() {
    let mut value = BranchWithIgnoredField {
        tracked: Some(Leaf { value: 5 }),
        ignored: NotTraversable { value: 10 },
    };

    PyGcTraversable::clear(&mut value);
    assert_eq!(value.ignored.value, 10);
    assert!(value.tracked.is_some());
}

#[test]
fn wrappers_compile_and_clear() {
    let mut map = BTreeMap::new();
    map.insert(Leaf { value: 1 }, Leaf { value: 2 });

    let mut set = BTreeSet::new();
    let _ = set.insert(Leaf { value: 3 });

    let lock = OnceLock::new();
    let _ = lock.set(Leaf { value: 4 });

    let mut hash_map = HashMap::new();
    hash_map.insert(Leaf { value: 10 }, Leaf { value: 11 });

    let mut hash_set = HashSet::new();
    let _ = hash_set.insert(Leaf { value: 12 });

    let mut vec_deque = VecDeque::new();
    vec_deque.push_back(Leaf { value: 13 });

    let mut linked_list = LinkedList::new();
    linked_list.push_back(Leaf { value: 14 });

    let mut wrappers = Wrappers {
        map,
        set,
        lock,
        marker: PhantomData,
        result: Ok(Leaf { value: 7 }),
        array: [Leaf { value: 8 }, Leaf { value: 9 }],
        hash_map,
        hash_set,
        vec_deque,
        linked_list,
    };

    PyGcTraversable::clear(&mut wrappers);
    assert_eq!(wrappers.map.len(), 1);
    assert_eq!(wrappers.set.len(), 1);
    assert!(wrappers.lock.get().is_some());
    assert_eq!(wrappers.hash_map.len(), 1);
    assert_eq!(wrappers.hash_set.len(), 1);
    assert_eq!(wrappers.vec_deque.len(), 1);
    assert_eq!(wrappers.linked_list.len(), 1);
}

#[test]
fn opaque_wrapper_breaks_traversal_chain() {
    assert!(!OpaqueWrapper::MAY_CONTAIN_CYCLES);

    Python::attach(|py| {
        let obj = py.None();
        let mut value = OpaqueWrapper {
            hidden: PyGcOpaque::new(obj),
        };

        let ptr_before = value.hidden.as_ptr();
        PyGcTraversable::clear(&mut value);
        assert_eq!(ptr_before, value.hidden.as_ptr());
    });
}

#[test]
fn direct_py_field_is_traversed() {
    Python::attach(|py| {
        let inner = py.None();
        let value = Py::new(py, TraversedByPyClass { field: inner.clone_ref(py) }).unwrap();

        let locals = PyDict::new(py);
        locals.set_item("gc", py.import("gc").unwrap()).unwrap();
        locals.set_item("obj", value.bind(py)).unwrap();
        locals.set_item("inner", inner.bind(py)).unwrap();

        py.run(
            c"found = False
for r in gc.get_referents(obj):
    if r is inner:
        found = True
        break
assert found",
            None,
            Some(&locals),
        )
        .unwrap();
    });
}
