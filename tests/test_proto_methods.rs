#![cfg(feature = "macros")]

use pyo3::exceptions::{PyAttributeError, PyIndexError, PyValueError};
use pyo3::types::{PyDict, PyList, PyMapping, PySequence, PySlice, PyType};
use pyo3::{prelude::*, py_run};
use std::iter;
use std::sync::Mutex;

mod test_utils;

#[pyclass]
struct EmptyClass;

#[pyclass]
struct ExampleClass {
    #[pyo3(get, set)]
    value: i32,
    custom_attr: Option<i32>,
}

#[pymethods]
impl ExampleClass {
    fn __getattr__(&self, py: Python<'_>, attr: &str) -> PyResult<Py<PyAny>> {
        if attr == "special_custom_attr" {
            Ok(self.custom_attr.into_pyobject(py)?.into_any().unbind())
        } else {
            Err(PyAttributeError::new_err(attr.to_string()))
        }
    }

    fn __setattr__(&mut self, attr: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        if attr == "special_custom_attr" {
            self.custom_attr = Some(value.extract()?);
            Ok(())
        } else {
            Err(PyAttributeError::new_err(attr.to_string()))
        }
    }

    fn __delattr__(&mut self, attr: &str) -> PyResult<()> {
        if attr == "special_custom_attr" {
            self.custom_attr = None;
            Ok(())
        } else {
            Err(PyAttributeError::new_err(attr.to_string()))
        }
    }

    fn __str__(&self) -> String {
        self.value.to_string()
    }

    fn __repr__(&self) -> String {
        format!("ExampleClass(value={})", self.value)
    }

    fn __hash__(&self) -> u64 {
        let i64_value: i64 = self.value.into();
        i64_value as u64
    }

    fn __bool__(&self) -> bool {
        self.value != 0
    }
}

fn make_example(py: Python<'_>) -> Bound<'_, ExampleClass> {
    Bound::new(
        py,
        ExampleClass {
            value: 5,
            custom_attr: Some(20),
        },
    )
    .unwrap()
}

#[test]
fn test_getattr() {
    Python::attach(|py| {
        let example_py = make_example(py);
        assert_eq!(
            example_py
                .getattr("value")
                .unwrap()
                .extract::<i32>()
                .unwrap(),
            5,
        );
        assert_eq!(
            example_py
                .getattr("special_custom_attr")
                .unwrap()
                .extract::<i32>()
                .unwrap(),
            20,
        );
        assert!(example_py
            .getattr("other_attr")
            .unwrap_err()
            .is_instance_of::<PyAttributeError>(py));
    })
}

#[test]
fn test_setattr() {
    Python::attach(|py| {
        let example_py = make_example(py);
        example_py.setattr("special_custom_attr", 15).unwrap();
        assert_eq!(
            example_py
                .getattr("special_custom_attr")
                .unwrap()
                .extract::<i32>()
                .unwrap(),
            15,
        );
    })
}

#[test]
fn test_delattr() {
    Python::attach(|py| {
        let example_py = make_example(py);
        example_py.delattr("special_custom_attr").unwrap();
        assert!(example_py.getattr("special_custom_attr").unwrap().is_none());
    })
}

#[test]
fn test_str() {
    Python::attach(|py| {
        let example_py = make_example(py);
        assert_eq!(example_py.str().unwrap(), "5");
    })
}

#[test]
fn test_repr() {
    Python::attach(|py| {
        let example_py = make_example(py);
        assert_eq!(example_py.repr().unwrap(), "ExampleClass(value=5)");
    })
}

#[test]
fn test_hash() {
    Python::attach(|py| {
        let example_py = make_example(py);
        assert_eq!(example_py.hash().unwrap(), 5);
    })
}

#[test]
fn test_bool() {
    Python::attach(|py| {
        let example_py = make_example(py);
        assert!(example_py.is_truthy().unwrap());
        example_py.borrow_mut().value = 0;
        assert!(!example_py.is_truthy().unwrap());
    })
}

#[pyclass]
pub struct LenOverflow;

#[pymethods]
impl LenOverflow {
    fn __len__(&self) -> usize {
        (isize::MAX as usize) + 1
    }
}

#[test]
fn len_overflow() {
    Python::attach(|py| {
        let inst = Py::new(py, LenOverflow).unwrap();
        py_expect_exception!(py, inst, "len(inst)", PyOverflowError);
    });
}

#[pyclass]
pub struct Mapping {
    values: Py<PyDict>,
}

#[pymethods]
impl Mapping {
    fn __len__(&self, py: Python<'_>) -> usize {
        self.values.bind(py).len()
    }

    fn __getitem__<'py>(&self, key: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let any: &Bound<'py, PyAny> = self.values.bind(key.py());
        any.get_item(key)
    }

    fn __setitem__<'py>(&self, key: &Bound<'py, PyAny>, value: &Bound<'py, PyAny>) -> PyResult<()> {
        self.values.bind(key.py()).set_item(key, value)
    }

    fn __delitem__(&self, key: &Bound<'_, PyAny>) -> PyResult<()> {
        self.values.bind(key.py()).del_item(key)
    }
}

#[test]
fn mapping() {
    Python::attach(|py| {
        PyMapping::register::<Mapping>(py).unwrap();

        let inst = Py::new(
            py,
            Mapping {
                values: PyDict::new(py).into(),
            },
        )
        .unwrap();

        let mapping: &Bound<'_, PyMapping> = inst.bind(py).cast().unwrap();

        py_assert!(py, inst, "len(inst) == 0");

        py_run!(py, inst, "inst['foo'] = 'foo'");
        py_assert!(py, inst, "inst['foo'] == 'foo'");
        py_run!(py, inst, "del inst['foo']");
        py_expect_exception!(py, inst, "inst['foo']", PyKeyError);

        // Default iteration will call __getitem__ with integer indices
        // which fails with a KeyError
        py_expect_exception!(py, inst, "[*inst] == []", PyKeyError, "0");

        // check mapping protocol
        assert_eq!(mapping.len().unwrap(), 0);

        mapping.set_item(0, 5).unwrap();
        assert_eq!(mapping.len().unwrap(), 1);

        assert_eq!(mapping.get_item(0).unwrap().extract::<u8>().unwrap(), 5);

        mapping.del_item(0).unwrap();
        assert_eq!(mapping.len().unwrap(), 0);
    });
}

#[derive(FromPyObject)]
enum SequenceIndex<'py> {
    Integer(isize),
    Slice(Bound<'py, PySlice>),
}

#[pyclass]
pub struct Sequence {
    values: Vec<Py<PyAny>>,
}

#[pymethods]
impl Sequence {
    fn __len__(&self) -> usize {
        self.values.len()
    }

    fn __getitem__(&self, index: SequenceIndex<'_>, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match index {
            SequenceIndex::Integer(index) => {
                let uindex = self.usize_index(index)?;
                self.values
                    .get(uindex)
                    .map(|o| o.clone_ref(py))
                    .ok_or_else(|| PyIndexError::new_err(index))
            }
            // Just to prove that slicing can be implemented
            SequenceIndex::Slice(s) => Ok(s.into()),
        }
    }

    fn __setitem__(&mut self, index: isize, value: Py<PyAny>) -> PyResult<()> {
        let uindex = self.usize_index(index)?;
        self.values
            .get_mut(uindex)
            .map(|place| *place = value)
            .ok_or_else(|| PyIndexError::new_err(index))
    }

    fn __delitem__(&mut self, index: isize) -> PyResult<()> {
        let uindex = self.usize_index(index)?;
        if uindex >= self.values.len() {
            Err(PyIndexError::new_err(index))
        } else {
            self.values.remove(uindex);
            Ok(())
        }
    }

    fn append(&mut self, value: Py<PyAny>) {
        self.values.push(value);
    }
}

impl Sequence {
    fn usize_index(&self, index: isize) -> PyResult<usize> {
        if index < 0 {
            let corrected_index = index + self.values.len() as isize;
            if corrected_index < 0 {
                Err(PyIndexError::new_err(index))
            } else {
                Ok(corrected_index as usize)
            }
        } else {
            Ok(index as usize)
        }
    }
}

#[test]
fn sequence() {
    Python::attach(|py| {
        PySequence::register::<Sequence>(py).unwrap();

        let inst = Py::new(py, Sequence { values: vec![] }).unwrap();

        let sequence: &Bound<'_, PySequence> = inst.bind(py).cast().unwrap();

        py_assert!(py, inst, "len(inst) == 0");

        py_expect_exception!(py, inst, "inst[0]", PyIndexError);
        py_run!(py, inst, "inst.append('foo')");

        py_assert!(py, inst, "inst[0] == 'foo'");
        py_assert!(py, inst, "inst[-1] == 'foo'");

        py_expect_exception!(py, inst, "inst[1]", PyIndexError);
        py_expect_exception!(py, inst, "inst[-2]", PyIndexError);

        py_assert!(py, inst, "[*inst] == ['foo']");

        py_run!(py, inst, "del inst[0]");

        py_expect_exception!(py, inst, "inst['foo']", PyTypeError);

        py_assert!(py, inst, "inst[0:2] == slice(0, 2)");

        // check sequence protocol

        // we don't implement sequence length so that CPython doesn't attempt to correct negative
        // indices.
        assert!(sequence.len().is_err());
        // however regular python len() works thanks to mp_len slot
        assert_eq!(inst.bind(py).len().unwrap(), 0);

        py_run!(py, inst, "inst.append(0)");
        sequence.set_item(0, 5).unwrap();
        assert_eq!(inst.bind(py).len().unwrap(), 1);

        assert_eq!(sequence.get_item(0).unwrap().extract::<u8>().unwrap(), 5);
        sequence.del_item(0).unwrap();

        assert_eq!(inst.bind(py).len().unwrap(), 0);
    });
}

#[pyclass]
struct Iterator {
    iter: Mutex<Box<dyn iter::Iterator<Item = i32> + Send>>,
}

#[pymethods]
impl Iterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(slf: PyRefMut<'_, Self>) -> Option<i32> {
        slf.iter.lock().unwrap().next()
    }
}

#[test]
fn iterator() {
    Python::attach(|py| {
        let inst = Py::new(
            py,
            Iterator {
                iter: Mutex::new(Box::new(5..8)),
            },
        )
        .unwrap();
        py_assert!(py, inst, "iter(inst) is inst");
        py_assert!(py, inst, "list(inst) == [5, 6, 7]");
    });
}

#[pyclass]
struct Callable;

#[pymethods]
impl Callable {
    fn __call__(&self, arg: i32) -> i32 {
        arg * 6
    }
}

#[pyclass]
struct NotCallable;

#[test]
fn callable() {
    Python::attach(|py| {
        let c = Py::new(py, Callable).unwrap();
        py_assert!(py, c, "callable(c)");
        py_assert!(py, c, "c(7) == 42");

        let nc = Py::new(py, NotCallable).unwrap();
        py_assert!(py, nc, "not callable(nc)");
    });
}

#[pyclass]
#[derive(Debug)]
struct SetItem {
    key: i32,
    val: i32,
}

#[pymethods]
impl SetItem {
    fn __setitem__(&mut self, key: i32, val: i32) {
        self.key = key;
        self.val = val;
    }
}

#[test]
fn setitem() {
    Python::attach(|py| {
        let c = Bound::new(py, SetItem { key: 0, val: 0 }).unwrap();
        py_run!(py, c, "c[1] = 2");
        {
            let c = c.borrow();
            assert_eq!(c.key, 1);
            assert_eq!(c.val, 2);
        }
        py_expect_exception!(py, c, "del c[1]", PyNotImplementedError);
    });
}

#[pyclass]
struct DelItem {
    key: i32,
}

#[pymethods]
impl DelItem {
    fn __delitem__(&mut self, key: i32) {
        self.key = key;
    }
}

#[test]
fn delitem() {
    Python::attach(|py| {
        let c = Bound::new(py, DelItem { key: 0 }).unwrap();
        py_run!(py, c, "del c[1]");
        {
            let c = c.borrow();
            assert_eq!(c.key, 1);
        }
        py_expect_exception!(py, c, "c[1] = 2", PyNotImplementedError);
    });
}

#[pyclass]
struct SetDelItem {
    val: Option<i32>,
}

#[pymethods]
impl SetDelItem {
    fn __setitem__(&mut self, _key: i32, val: i32) {
        self.val = Some(val);
    }

    fn __delitem__(&mut self, _key: i32) {
        self.val = None;
    }
}

#[test]
fn setdelitem() {
    Python::attach(|py| {
        let c = Bound::new(py, SetDelItem { val: None }).unwrap();
        py_run!(py, c, "c[1] = 2");
        {
            let c = c.borrow();
            assert_eq!(c.val, Some(2));
        }
        py_run!(py, c, "del c[1]");
        let c = c.borrow();
        assert_eq!(c.val, None);
    });
}

#[pyclass]
struct Contains {}

#[pymethods]
impl Contains {
    fn __contains__(&self, item: i32) -> bool {
        item >= 0
    }
}

#[test]
fn contains() {
    Python::attach(|py| {
        let c = Py::new(py, Contains {}).unwrap();
        py_run!(py, c, "assert 1 in c");
        py_run!(py, c, "assert -1 not in c");
        py_expect_exception!(py, c, "assert 'wrong type' not in c", PyTypeError);
    });
}

#[pyclass]
struct GetItem {}

#[pymethods]
impl GetItem {
    fn __getitem__(&self, idx: &Bound<'_, PyAny>) -> PyResult<&'static str> {
        if let Ok(slice) = idx.cast::<PySlice>() {
            let indices = slice.indices(1000)?;
            if indices.start == 100 && indices.stop == 200 && indices.step == 1 {
                return Ok("slice");
            }
        } else if let Ok(idx) = idx.extract::<isize>() {
            if idx == 1 {
                return Ok("int");
            }
        }
        Err(PyValueError::new_err("error"))
    }
}

#[test]
fn test_getitem() {
    Python::attach(|py| {
        let ob = Py::new(py, GetItem {}).unwrap();

        py_assert!(py, ob, "ob[1] == 'int'");
        py_assert!(py, ob, "ob[100:200:1] == 'slice'");
    });
}

#[pyclass]
struct ClassWithGetAttr {
    #[pyo3(get, set)]
    data: u32,
}

#[pymethods]
impl ClassWithGetAttr {
    fn __getattr__(&self, _name: &str) -> u32 {
        self.data * 2
    }
}

#[test]
fn getattr_doesnt_override_member() {
    Python::attach(|py| {
        let inst = Py::new(py, ClassWithGetAttr { data: 4 }).unwrap();
        py_assert!(py, inst, "inst.data == 4");
        py_assert!(py, inst, "inst.a == 8");
    });
}

#[pyclass]
struct ClassWithGetAttribute {
    #[pyo3(get, set)]
    data: u32,
}

#[pymethods]
impl ClassWithGetAttribute {
    fn __getattribute__(&self, _name: &str) -> u32 {
        self.data * 2
    }
}

#[test]
fn getattribute_overrides_member() {
    Python::attach(|py| {
        let inst = Py::new(py, ClassWithGetAttribute { data: 4 }).unwrap();
        py_assert!(py, inst, "inst.data == 8");
        py_assert!(py, inst, "inst.y == 8");
    });
}

#[pyclass]
struct ClassWithGetAttrAndGetAttribute;

#[pymethods]
impl ClassWithGetAttrAndGetAttribute {
    fn __getattribute__(&self, name: &str) -> PyResult<u32> {
        if name == "exists" {
            Ok(42)
        } else if name == "error" {
            Err(PyValueError::new_err("bad"))
        } else {
            Err(PyAttributeError::new_err("fallback"))
        }
    }

    fn __getattr__(&self, name: &str) -> PyResult<u32> {
        if name == "lucky" {
            Ok(57)
        } else {
            Err(PyAttributeError::new_err("no chance"))
        }
    }
}

#[test]
fn getattr_and_getattribute() {
    Python::attach(|py| {
        let inst = Py::new(py, ClassWithGetAttrAndGetAttribute).unwrap();
        py_assert!(py, inst, "inst.exists == 42");
        py_assert!(py, inst, "inst.lucky == 57");
        py_expect_exception!(py, inst, "inst.error", PyValueError);
        py_expect_exception!(py, inst, "inst.unlucky", PyAttributeError);
    });
}

/// Wraps a Python future and yield it once.
#[pyclass]
#[derive(Debug)]
struct OnceFuture {
    future: Py<PyAny>,
    polled: bool,
}

#[pymethods]
impl OnceFuture {
    #[new]
    fn new(future: Py<PyAny>) -> Self {
        OnceFuture {
            future,
            polled: false,
        }
    }

    fn __await__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__<'py>(&mut self, py: Python<'py>) -> Option<&Bound<'py, PyAny>> {
        if !self.polled {
            self.polled = true;
            Some(self.future.bind(py))
        } else {
            None
        }
    }
}

#[test]
#[cfg(not(target_arch = "wasm32"))] // Won't work without wasm32 event loop (e.g., Pyodide has WebLoop)
fn test_await() {
    Python::attach(|py| {
        let once = py.get_type::<OnceFuture>();
        let source = pyo3_ffi::c_str!(
            r#"
import asyncio
import sys

async def main():
    res = await Once(await asyncio.sleep(0.1))
    assert res is None

# For an odd error similar to https://bugs.python.org/issue38563
if sys.platform == "win32" and sys.version_info >= (3, 8, 0):
    asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())

asyncio.run(main())
"#
        );
        let globals = PyModule::import(py, "__main__").unwrap().dict();
        globals.set_item("Once", once).unwrap();
        py.run(source, Some(&globals), None)
            .map_err(|e| e.display(py))
            .unwrap();
    });
}

#[pyclass]
struct AsyncIterator {
    future: Option<Py<OnceFuture>>,
}

#[pymethods]
impl AsyncIterator {
    #[new]
    fn new(future: Py<OnceFuture>) -> Self {
        Self {
            future: Some(future),
        }
    }

    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__(&mut self) -> Option<Py<OnceFuture>> {
        self.future.take()
    }
}

#[test]
#[cfg(not(target_arch = "wasm32"))] // Won't work without wasm32 event loop (e.g., Pyodide has WebLoop)
fn test_anext_aiter() {
    Python::attach(|py| {
        let once = py.get_type::<OnceFuture>();
        let source = pyo3_ffi::c_str!(
            r#"
import asyncio
import sys

async def main():
    count = 0
    async for result in AsyncIterator(Once(await asyncio.sleep(0.1))):
        # The Once is awaited as part of the `async for` and produces None
        assert result is None
        count +=1
    assert count == 1

# For an odd error similar to https://bugs.python.org/issue38563
if sys.platform == "win32" and sys.version_info >= (3, 8, 0):
    asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())

asyncio.run(main())
"#
        );
        let globals = PyModule::import(py, "__main__").unwrap().dict();
        globals.set_item("Once", once).unwrap();
        globals
            .set_item("AsyncIterator", py.get_type::<AsyncIterator>())
            .unwrap();
        py.run(source, Some(&globals), None)
            .map_err(|e| e.display(py))
            .unwrap();
    });
}

/// Increment the count when `__get__` is called.
#[pyclass]
struct DescrCounter {
    #[pyo3(get)]
    count: usize,
}

#[pymethods]
impl DescrCounter {
    #[new]
    fn new() -> Self {
        DescrCounter { count: 0 }
    }
    /// Each access will increase the count
    fn __get__<'a>(
        mut slf: PyRefMut<'a, Self>,
        _instance: &Bound<'_, PyAny>,
        _owner: Option<&Bound<'_, PyType>>,
    ) -> PyRefMut<'a, Self> {
        slf.count += 1;
        slf
    }
    /// Allow assigning a new counter to the descriptor, copying the count across
    fn __set__(&self, _instance: &Bound<'_, PyAny>, new_value: &mut Self) {
        new_value.count = self.count;
    }
    /// Delete to reset the counter
    fn __delete__(&mut self, _instance: &Bound<'_, PyAny>) {
        self.count = 0;
    }
}

#[test]
fn descr_getset() {
    Python::attach(|py| {
        let counter = py.get_type::<DescrCounter>();
        let source = pyo3_ffi::c_str!(pyo3::indoc::indoc!(
            r#"
class Class:
    counter = Counter()

# access via type
counter = Class.counter
assert counter.count == 1

# access with instance directly
assert Counter.__get__(counter, Class()).count == 2

# access via instance
c = Class()
assert c.counter.count == 3

# __set__
c.counter = Counter()
assert c.counter.count == 4

# __delete__
del c.counter
assert c.counter.count == 1
"#
        ));
        let globals = PyModule::import(py, "__main__").unwrap().dict();
        globals.set_item("Counter", counter).unwrap();
        py.run(source, Some(&globals), None)
            .map_err(|e| e.display(py))
            .unwrap();
    });
}

#[pyclass]
struct NotHashable;

#[pymethods]
impl NotHashable {
    #[classattr]
    const __hash__: Option<Py<PyAny>> = None;
}

#[test]
fn test_hash_opt_out() {
    // By default Python provides a hash implementation, which can be disabled by setting __hash__
    // to None.
    Python::attach(|py| {
        let empty = Py::new(py, EmptyClass).unwrap();
        py_assert!(py, empty, "hash(empty) is not None");

        let not_hashable = Py::new(py, NotHashable).unwrap();
        py_expect_exception!(py, not_hashable, "hash(not_hashable)", PyTypeError);
    })
}

/// Class with __iter__ gets default contains from CPython.
#[pyclass]
struct DefaultedContains;

#[pymethods]
impl DefaultedContains {
    fn __iter__(&self, py: Python<'_>) -> Py<PyAny> {
        PyList::new(py, ["a", "b", "c"])
            .unwrap()
            .as_any()
            .try_iter()
            .unwrap()
            .into()
    }
}

#[pyclass]
struct NoContains;

#[pymethods]
impl NoContains {
    fn __iter__(&self, py: Python<'_>) -> Py<PyAny> {
        PyList::new(py, ["a", "b", "c"])
            .unwrap()
            .as_any()
            .try_iter()
            .unwrap()
            .into()
    }

    // Equivalent to the opt-out const form in NotHashable above, just more verbose, to confirm this
    // also works.
    #[classattr]
    fn __contains__() -> Option<Py<PyAny>> {
        None
    }
}

#[test]
fn test_contains_opt_out() {
    Python::attach(|py| {
        let defaulted_contains = Py::new(py, DefaultedContains).unwrap();
        py_assert!(py, defaulted_contains, "'a' in defaulted_contains");

        let no_contains = Py::new(py, NoContains).unwrap();
        py_expect_exception!(py, no_contains, "'a' in no_contains", PyTypeError);
    })
}
