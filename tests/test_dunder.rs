#![feature(proc_macro, specialization)]

extern crate pyo3;

use pyo3::prelude::*;
use std::{isize, iter};
use pyo3::ffi;

use pyo3::py::class as pyclass;
use pyo3::py::methods as pymethods;
use pyo3::py::proto as pyproto;


#[macro_use]
mod common;


#[pyclass]
pub struct Len {
    l: usize,
    token: PyToken,
}

#[pyproto]
impl PyMappingProtocol for Len {
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.l)
    }
}

#[test]
fn len() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = Py::new(py, |t| Len{l: 10, token: t}).unwrap();
    py_assert!(py, inst, "len(inst) == 10");
    unsafe {
        assert_eq!(ffi::PyObject_Size(inst.as_ptr()), 10);
        assert_eq!(ffi::PyMapping_Size(inst.as_ptr()), 10);
    }

    let inst = Py::new(py, |t| Len{l: (isize::MAX as usize) + 1, token: t}).unwrap();
    py_expect_exception!(py, inst, "len(inst)", OverflowError);
}

#[pyclass]
struct Iterator{
    iter: Box<iter::Iterator<Item=i32> + Send>,
    token: PyToken,
}

#[pyproto]
impl PyIterProtocol for Iterator {
    fn __iter__(&mut self) -> PyResult<Py<Iterator>> {
        Ok(self.into())
    }

    fn __next__(&mut self) -> PyResult<Option<i32>> {
        Ok(self.iter.next())
    }
}

#[test]
fn iterator() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = Py::new(py, |t| Iterator{iter: Box::new(5..8), token: t}).unwrap();
    py_assert!(py, inst, "iter(inst) is inst");
    py_assert!(py, inst, "list(inst) == [5, 6, 7]");
}

#[pyclass]
struct StringMethods {token: PyToken}

#[pyproto]
impl<'p> PyObjectProtocol<'p> for StringMethods {
    fn __str__(&self) -> PyResult<&'static str> {
        Ok("str")
    }

    fn __repr__(&self) -> PyResult<&'static str> {
        Ok("repr")
    }

    fn __format__(&self, format_spec: String) -> PyResult<String> {
        Ok(format!("format({})", format_spec))
    }

    fn __unicode__(&self) -> PyResult<PyObject> {
        Ok(PyString::new(self.py(), "unicode").into())
    }

    fn __bytes__(&self) -> PyResult<PyObject> {
        Ok(PyBytes::new(self.py(), b"bytes").into())
    }
}

#[cfg(Py_3)]
#[test]
fn string_methods() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = Py::new(py, |t| StringMethods{token: t}).unwrap();
    py_assert!(py, obj, "str(obj) == 'str'");
    py_assert!(py, obj, "repr(obj) == 'repr'");
    py_assert!(py, obj, "'{0:x}'.format(obj) == 'format(x)'");
    py_assert!(py, obj, "bytes(obj) == b'bytes'");
}

#[cfg(not(Py_3))]
#[test]
fn string_methods() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = Py::new(py, |t| StringMethods{token: t}).unwrap();
    py_assert!(py, obj, "str(obj) == 'str'");
    py_assert!(py, obj, "repr(obj) == 'repr'");
    py_assert!(py, obj, "unicode(obj) == 'unicode'");
    py_assert!(py, obj, "'{0:x}'.format(obj) == 'format(x)'");
}


#[pyclass]
struct Comparisons {
    val: i32,
    token: PyToken,
}

#[pyproto]
impl PyObjectProtocol for Comparisons {
    fn __hash__(&self) -> PyResult<isize> {
        Ok(self.val as isize)
    }
    fn __bool__(&self) -> PyResult<bool> {
        Ok(self.val != 0)
    }
}


#[test]
fn comparisons() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let zero = Py::new(py, |t| Comparisons{val: 0, token: t}).unwrap();
    let one = Py::new(py, |t| Comparisons{val: 1, token: t}).unwrap();
    let ten = Py::new(py, |t| Comparisons{val: 10, token: t}).unwrap();
    let minus_one = Py::new(py, |t| Comparisons{val: -1, token: t}).unwrap();
    py_assert!(py, one, "hash(one) == 1");
    py_assert!(py, ten, "hash(ten) == 10");
    py_assert!(py, minus_one, "hash(minus_one) == -2");

    py_assert!(py, one, "bool(one) is True");
    py_assert!(py, zero, "not zero");
}


#[pyclass]
struct Sequence {
    token: PyToken
}

#[pyproto]
impl PySequenceProtocol for Sequence {
    fn __len__(&self) -> PyResult<usize> {
        Ok(5)
    }

    fn __getitem__(&self, key: isize) -> PyResult<isize> {
        if key == 5 {
            return Err(PyErr::new::<exc::IndexError, NoArgs>(NoArgs));
        }
        Ok(key)
    }
}

#[test]
fn sequence() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init(|t| Sequence{token: t}).unwrap();
    py_assert!(py, c, "list(c) == [0, 1, 2, 3, 4]");
    py_expect_exception!(py, c, "c['abc']", TypeError);
}


#[pyclass]
struct Callable {token: PyToken}

#[pymethods]
impl Callable {

    #[__call__]
    fn __call__(&self, arg: i32) -> PyResult<i32> {
        Ok(arg * 6)
    }
}

#[test]
fn callable() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init(|t| Callable{token: t}).unwrap();
    py_assert!(py, c, "callable(c)");
    py_assert!(py, c, "c(7) == 42");

    let nc = py.init(|t| Comparisons{val: 0, token: t}).unwrap();
    py_assert!(py, nc, "not callable(nc)");
}

#[pyclass]
struct SetItem {
    key: i32,
    val: i32,
    token: PyToken,
}

#[pyproto]
impl PyMappingProtocol<'a> for SetItem {
    fn __setitem__(&mut self, key: i32, val: i32) -> PyResult<()> {
        self.key = key;
        self.val = val;
        Ok(())
    }
}

#[test]
fn setitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init_ref(|t| SetItem{key: 0, val: 0, token: t}).unwrap();
    py_run!(py, c, "c[1] = 2");
    assert_eq!(c.key, 1);
    assert_eq!(c.val, 2);
    py_expect_exception!(py, c, "del c[1]", NotImplementedError);
}

#[pyclass]
struct DelItem {
    key: i32,
    token: PyToken,
}

#[pyproto]
impl PyMappingProtocol<'a> for DelItem {
    fn __delitem__(&mut self, key: i32) -> PyResult<()> {
        self.key = key;
        Ok(())
    }
}

#[test]
fn delitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init_ref(|t| DelItem{key:0, token:t}).unwrap();
    py_run!(py, c, "del c[1]");
    assert_eq!(c.key, 1);
    py_expect_exception!(py, c, "c[1] = 2", NotImplementedError);
}

#[pyclass]
struct SetDelItem {
    val: Option<i32>,
    token: PyToken,
}

#[pyproto]
impl PyMappingProtocol for SetDelItem {
    fn __setitem__(&mut self, _key: i32, val: i32) -> PyResult<()> {
        self.val = Some(val);
        Ok(())
    }

    fn __delitem__(&mut self, _key: i32) -> PyResult<()> {
        self.val = None;
        Ok(())
    }
}

#[test]
fn setdelitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init_ref(|t| SetDelItem{val: None, token: t}).unwrap();
    py_run!(py, c, "c[1] = 2");
    assert_eq!(c.val, Some(2));
    py_run!(py, c, "del c[1]");
    assert_eq!(c.val, None);
}

#[pyclass]
struct Reversed {token: PyToken}

#[pyproto]
impl PyMappingProtocol for Reversed{
    fn __reversed__(&self) -> PyResult<&'static str> {
        Ok("I am reversed")
    }
}

#[test]
fn reversed() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init(|t| Reversed{token: t}).unwrap();
    py_run!(py, c, "assert reversed(c) == 'I am reversed'");
}

#[pyclass]
struct Contains {token: PyToken}

#[pyproto]
impl PySequenceProtocol for Contains {
    fn __contains__(&self, item: i32) -> PyResult<bool> {
        Ok(item >= 0)
    }
}

#[test]
fn contains() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init(|t| Contains{token: t}).unwrap();
    py_run!(py, c, "assert 1 in c");
    py_run!(py, c, "assert -1 not in c");
    py_expect_exception!(py, c, "assert 'wrong type' not in c", TypeError);
}


#[pyclass]
struct ContextManager {
    exit_called: bool,
    token: PyToken,
}

#[pyproto]
impl<'p> PyContextProtocol<'p> for ContextManager {

    fn __enter__(&mut self) -> PyResult<i32> {
        Ok(42)
    }

    fn __exit__(&mut self,
                ty: Option<&'p PyType>,
                _value: Option<&'p PyObjectRef>,
                _traceback: Option<&'p PyObjectRef>) -> PyResult<bool> {
        self.exit_called = true;
        if ty == Some(self.py().get_type::<exc::ValueError>()) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[test]
fn context_manager() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init_mut(|t| ContextManager{exit_called: false, token: t}).unwrap();
    py_run!(py, c, "with c as x: assert x == 42");
    assert!(c.exit_called);

    c.exit_called = false;
    py_run!(py, c, "with c as x: raise ValueError");
    assert!(c.exit_called);

    c.exit_called = false;
    py_expect_exception!(
        py, c, "with c as x: raise NotImplementedError", NotImplementedError);
    assert!(c.exit_called);
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
struct Test {
    token: PyToken
}

#[pyproto]
impl<'p> PyMappingProtocol<'p> for Test
{
    fn __getitem__(&self, idx: &PyObjectRef) -> PyResult<PyObject> {
        if let Ok(slice) = idx.cast_as::<PySlice>() {
            let indices = slice.indices(1000)?;
            if indices.start == 100 && indices.stop == 200 && indices.step == 1 {
                return Ok("slice".into_object(self.py()))
            }
        }
        else if let Ok(idx) = idx.extract::<isize>() {
            if idx == 1 {
                return Ok("int".into_object(self.py()))
            }
        }
        Err(PyErr::new::<exc::ValueError, _>("error"))
    }
}

#[test]
fn test_cls_impl() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let ob = py.init(|t| Test{token: t}).unwrap();
    let d = PyDict::new(py);
    d.set_item("ob", ob).unwrap();

    py.run("assert ob[1] == 'int'", None, Some(d)).unwrap();
    py.run("assert ob[100:200:1] == 'slice'", None, Some(d)).unwrap();
}

#[pyclass(dict)]
struct DunderDictSupport {
    token: PyToken,
}

#[test]
fn dunder_dict_support() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = Py::new_ref(py, |t| DunderDictSupport{token: t}).unwrap();
    py_run!(py, inst, r#"
        inst.a = 1
        assert inst.a == 1
    "#);
}

#[pyclass(weakref, dict)]
struct WeakRefDunderDictSupport {
    token: PyToken,
}

#[test]
fn weakref_dunder_dict_support() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = Py::new_ref(py, |t| WeakRefDunderDictSupport{token: t}).unwrap();
    py_run!(py, inst, "import weakref; assert weakref.ref(inst)() is inst; inst.a = 1; assert inst.a == 1");
}
