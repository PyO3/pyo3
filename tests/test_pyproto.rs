#![cfg(feature = "macros")]
#![cfg(feature = "pyproto")]

use pyo3::class::{
    PyAsyncProtocol, PyDescrProtocol, PyIterProtocol, PyMappingProtocol, PyObjectProtocol,
    PySequenceProtocol,
};
use pyo3::exceptions::{PyIndexError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PySlice, PyType};
use pyo3::{ffi, py_run, AsPyPointer, PyCell};
use std::convert::TryFrom;
use std::{isize, iter};

mod common;

#[pyclass]
pub struct Len {
    l: usize,
}

#[pyproto]
impl PyMappingProtocol for Len {
    fn __len__(&self) -> usize {
        self.l
    }
}

#[test]
fn len() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = Py::new(py, Len { l: 10 }).unwrap();
    py_assert!(py, inst, "len(inst) == 10");
    unsafe {
        assert_eq!(ffi::PyObject_Size(inst.as_ptr()), 10);
        assert_eq!(ffi::PyMapping_Size(inst.as_ptr()), 10);
    }

    let inst = Py::new(
        py,
        Len {
            l: (isize::MAX as usize) + 1,
        },
    )
    .unwrap();
    py_expect_exception!(py, inst, "len(inst)", PyOverflowError);
}

#[pyclass]
struct Iterator {
    iter: Box<dyn iter::Iterator<Item = i32> + Send>,
}

#[pyproto]
impl PyIterProtocol for Iterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<i32> {
        slf.iter.next()
    }
}

#[test]
fn iterator() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = Py::new(
        py,
        Iterator {
            iter: Box::new(5..8),
        },
    )
    .unwrap();
    py_assert!(py, inst, "iter(inst) is inst");
    py_assert!(py, inst, "list(inst) == [5, 6, 7]");
}

#[pyclass]
struct StringMethods {}

#[pyproto]
impl PyObjectProtocol for StringMethods {
    fn __str__(&self) -> &'static str {
        "str"
    }

    fn __repr__(&self) -> &'static str {
        "repr"
    }
}

#[test]
fn string_methods() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = Py::new(py, StringMethods {}).unwrap();
    py_assert!(py, obj, "str(obj) == 'str'");
    py_assert!(py, obj, "repr(obj) == 'repr'");
}

#[pyclass]
struct Comparisons {
    val: i32,
}

#[pyproto]
impl PyObjectProtocol for Comparisons {
    fn __hash__(&self) -> isize {
        self.val as isize
    }
    fn __bool__(&self) -> bool {
        self.val != 0
    }
}

#[test]
fn comparisons() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let zero = Py::new(py, Comparisons { val: 0 }).unwrap();
    let one = Py::new(py, Comparisons { val: 1 }).unwrap();
    let ten = Py::new(py, Comparisons { val: 10 }).unwrap();
    let minus_one = Py::new(py, Comparisons { val: -1 }).unwrap();
    py_assert!(py, one, "hash(one) == 1");
    py_assert!(py, ten, "hash(ten) == 10");
    py_assert!(py, minus_one, "hash(minus_one) == -2");

    py_assert!(py, one, "bool(one) is True");
    py_assert!(py, zero, "not zero");
}

#[pyclass]
#[derive(Debug)]
struct Sequence {
    fields: Vec<String>,
}

impl Default for Sequence {
    fn default() -> Sequence {
        let mut fields = vec![];
        for &s in &["A", "B", "C", "D", "E", "F", "G"] {
            fields.push(s.to_string());
        }
        Sequence { fields }
    }
}

#[pyproto]
impl PySequenceProtocol for Sequence {
    fn __len__(&self) -> usize {
        self.fields.len()
    }

    fn __getitem__(&self, key: isize) -> PyResult<String> {
        let idx = usize::try_from(key)?;
        if let Some(s) = self.fields.get(idx) {
            Ok(s.clone())
        } else {
            Err(PyIndexError::new_err(()))
        }
    }

    fn __setitem__(&mut self, idx: isize, value: String) -> PyResult<()> {
        let idx = usize::try_from(idx)?;
        if let Some(elem) = self.fields.get_mut(idx) {
            *elem = value;
            Ok(())
        } else {
            Err(PyIndexError::new_err(()))
        }
    }
}

#[test]
fn sequence() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = Py::new(py, Sequence::default()).unwrap();
    py_assert!(py, c, "list(c) == ['A', 'B', 'C', 'D', 'E', 'F', 'G']");
    py_assert!(py, c, "c[-1] == 'G'");
    py_run!(
        py,
        c,
        r#"
    c[0] = 'H'
    assert c[0] == 'H'
"#
    );
    py_expect_exception!(py, c, "c['abc']", PyTypeError);
}

#[pyclass]
#[derive(Debug)]
struct SetItem {
    key: i32,
    val: i32,
}

#[pyproto]
impl PyMappingProtocol for SetItem {
    fn __setitem__(&mut self, key: i32, val: i32) {
        self.key = key;
        self.val = val;
    }
}

#[test]
fn setitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = PyCell::new(py, SetItem { key: 0, val: 0 }).unwrap();
    py_run!(py, c, "c[1] = 2");
    {
        let c = c.borrow();
        assert_eq!(c.key, 1);
        assert_eq!(c.val, 2);
    }
    py_expect_exception!(py, c, "del c[1]", PyNotImplementedError);
}

#[pyclass]
struct DelItem {
    key: i32,
}

#[pyproto]
impl PyMappingProtocol<'a> for DelItem {
    fn __delitem__(&mut self, key: i32) {
        self.key = key;
    }
}

#[test]
fn delitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = PyCell::new(py, DelItem { key: 0 }).unwrap();
    py_run!(py, c, "del c[1]");
    {
        let c = c.borrow();
        assert_eq!(c.key, 1);
    }
    py_expect_exception!(py, c, "c[1] = 2", PyNotImplementedError);
}

#[pyclass]
struct SetDelItem {
    val: Option<i32>,
}

#[pyproto]
impl PyMappingProtocol for SetDelItem {
    fn __setitem__(&mut self, _key: i32, val: i32) {
        self.val = Some(val);
    }

    fn __delitem__(&mut self, _key: i32) {
        self.val = None;
    }
}

#[test]
fn setdelitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = PyCell::new(py, SetDelItem { val: None }).unwrap();
    py_run!(py, c, "c[1] = 2");
    {
        let c = c.borrow();
        assert_eq!(c.val, Some(2));
    }
    py_run!(py, c, "del c[1]");
    let c = c.borrow();
    assert_eq!(c.val, None);
}

#[pyclass]
struct Contains {}

#[pyproto]
impl PySequenceProtocol for Contains {
    fn __contains__(&self, item: i32) -> bool {
        item >= 0
    }
}

#[test]
fn contains() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = Py::new(py, Contains {}).unwrap();
    py_run!(py, c, "assert 1 in c");
    py_run!(py, c, "assert -1 not in c");
    py_expect_exception!(py, c, "assert 'wrong type' not in c", PyTypeError);
}

#[test]
fn test_basics() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let v = PySlice::new(py, 1, 10, 2);
    let indices = v.indices(100).unwrap();
    assert_eq!(1, indices.start);
    assert_eq!(10, indices.stop);
    assert_eq!(2, indices.step);
    assert_eq!(5, indices.slicelength);
}

#[pyclass]
struct Test {}

#[pyproto]
impl<'p> PyMappingProtocol<'p> for Test {
    fn __getitem__(&self, idx: &PyAny) -> PyResult<&'static str> {
        if let Ok(slice) = idx.cast_as::<PySlice>() {
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
fn test_cls_impl() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let ob = Py::new(py, Test {}).unwrap();

    py_assert!(py, ob, "ob[1] == 'int'");
    py_assert!(py, ob, "ob[100:200:1] == 'slice'");
}

#[pyclass]
struct ClassWithGetAttr {
    #[pyo3(get, set)]
    data: u32,
}

#[pyproto]
impl PyObjectProtocol for ClassWithGetAttr {
    fn __getattr__(&self, _name: &str) -> u32 {
        self.data * 2
    }
}

#[test]
fn getattr_doesnt_override_member() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = PyCell::new(py, ClassWithGetAttr { data: 4 }).unwrap();
    py_assert!(py, inst, "inst.data == 4");
    py_assert!(py, inst, "inst.a == 8");
}

/// Wraps a Python future and yield it once.
#[pyclass]
struct OnceFuture {
    future: PyObject,
    polled: bool,
}

#[pymethods]
impl OnceFuture {
    #[new]
    fn new(future: PyObject) -> Self {
        OnceFuture {
            future,
            polled: false,
        }
    }
}

#[pyproto]
impl PyAsyncProtocol for OnceFuture {
    fn __await__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }
}

#[pyproto]
impl PyIterProtocol for OnceFuture {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
        if !slf.polled {
            slf.polled = true;
            Some(slf.future.clone())
        } else {
            None
        }
    }
}

#[test]
fn test_await() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let once = py.get_type::<OnceFuture>();
    let source = pyo3::indoc::indoc!(
        r#"
import asyncio
import sys

async def main():
    res = await Once(await asyncio.sleep(0.1))
    return res
# For an odd error similar to https://bugs.python.org/issue38563
if sys.platform == "win32" and sys.version_info >= (3, 8, 0):
    asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
# get_event_loop can raise an error: https://github.com/PyO3/pyo3/pull/961#issuecomment-645238579
loop = asyncio.new_event_loop()
asyncio.set_event_loop(loop)
assert loop.run_until_complete(main()) is None
loop.close()
"#
    );
    let globals = PyModule::import(py, "__main__").unwrap().dict();
    globals.set_item("Once", once).unwrap();
    py.run(source, Some(globals), None)
        .map_err(|e| e.print(py))
        .unwrap();
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
}

#[pyproto]
impl PyDescrProtocol for DescrCounter {
    fn __get__(
        mut slf: PyRefMut<Self>,
        _instance: &PyAny,
        _owner: Option<&PyType>,
    ) -> PyRefMut<Self> {
        slf.count += 1;
        slf
    }
    fn __set__(_slf: PyRef<Self>, _instance: &PyAny, mut new_value: PyRefMut<Self>) {
        new_value.count = _slf.count;
    }
}

#[test]
fn descr_getset() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let counter = py.get_type::<DescrCounter>();
    let source = pyo3::indoc::indoc!(
        r#"
class Class:
    counter = Counter()
c = Class()
c.counter # count += 1
assert c.counter.count == 2
c.counter = Counter()
assert c.counter.count == 3
"#
    );
    let globals = PyModule::import(py, "__main__").unwrap().dict();
    globals.set_item("Counter", counter).unwrap();
    py.run(source, Some(globals), None)
        .map_err(|e| e.print(py))
        .unwrap();
}
