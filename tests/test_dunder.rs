#![feature(specialization)]

extern crate pyo3;

use std::{isize, iter};

use pyo3::class::{
    PyContextProtocol, PyIterProtocol, PyMappingProtocol, PyObjectProtocol, PySequenceProtocol,
};
use pyo3::exceptions::{IndexError, ValueError};
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::python::ToPyPointer;
use pyo3::types::{PyBytes, PyDict, PyObjectRef, PySlice, PyString, PyType};

#[macro_use]
mod common;

#[pyclass]
pub struct Len {
    l: usize,
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

    let inst = Py::new(py, |_| Len { l: 10 }).unwrap();
    py_assert!(py, inst, "len(inst) == 10");
    unsafe {
        assert_eq!(ffi::PyObject_Size(inst.as_ptr()), 10);
        assert_eq!(ffi::PyMapping_Size(inst.as_ptr()), 10);
    }

    let inst = Py::new(py, |_| Len {
        l: (isize::MAX as usize) + 1,
    })
    .unwrap();
    py_expect_exception!(py, inst, "len(inst)", OverflowError);
}

#[pyclass]
struct Iterator {
    iter: Box<iter::Iterator<Item = i32> + Send>,
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

    let inst = Py::new(py, |_| Iterator {
        iter: Box::new(5..8),
    })
    .unwrap();
    py_assert!(py, inst, "iter(inst) is inst");
    py_assert!(py, inst, "list(inst) == [5, 6, 7]");
}

#[pyclass]
struct StringMethods {}

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

    fn __bytes__(&self) -> PyResult<PyObject> {
        let gil = GILGuard::acquire();
        Ok(PyBytes::new(gil.python(), b"bytes").into())
    }

    fn __unicode__(&self) -> PyResult<PyObject> {
        let gil = GILGuard::acquire();
        Ok(PyString::new(gil.python(), "unicode").into())
    }
}

#[cfg(Py_3)]
#[test]
fn string_methods() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = Py::new(py, |_| StringMethods {}).unwrap();
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

    let obj = Py::new(py, |_| StringMethods {}).unwrap();
    py_assert!(py, obj, "str(obj) == 'str'");
    py_assert!(py, obj, "repr(obj) == 'repr'");
    py_assert!(py, obj, "unicode(obj) == 'unicode'");
    py_assert!(py, obj, "'{0:x}'.format(obj) == 'format(x)'");
}

#[pyclass]
struct Comparisons {
    val: i32,
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

    let zero = Py::new(py, |_| Comparisons { val: 0 }).unwrap();
    let one = Py::new(py, |_| Comparisons { val: 1 }).unwrap();
    let ten = Py::new(py, |_| Comparisons { val: 10 }).unwrap();
    let minus_one = Py::new(py, |_| Comparisons { val: -1 }).unwrap();
    py_assert!(py, one, "hash(one) == 1");
    py_assert!(py, ten, "hash(ten) == 10");
    py_assert!(py, minus_one, "hash(minus_one) == -2");

    py_assert!(py, one, "bool(one) is True");
    py_assert!(py, zero, "not zero");
}

#[pyclass]
struct Sequence {}

#[pyproto]
impl PySequenceProtocol for Sequence {
    fn __len__(&self) -> PyResult<usize> {
        Ok(5)
    }

    fn __getitem__(&self, key: isize) -> PyResult<isize> {
        if key == 5 {
            return Err(PyErr::new::<IndexError, NoArgs>(NoArgs));
        }
        Ok(key)
    }
}

#[test]
fn sequence() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init(|_| Sequence {}).unwrap();
    py_assert!(py, c, "list(c) == [0, 1, 2, 3, 4]");
    py_expect_exception!(py, c, "c['abc']", TypeError);
}

#[pyclass]
struct Callable {}

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

    let c = py.init(|_| Callable {}).unwrap();
    py_assert!(py, c, "callable(c)");
    py_assert!(py, c, "c(7) == 42");

    let nc = py.init(|_| Comparisons { val: 0 }).unwrap();
    py_assert!(py, nc, "not callable(nc)");
}

#[pyclass]
struct SetItem {
    key: i32,
    val: i32,
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

    let c = py.init_ref(|_| SetItem { key: 0, val: 0 }).unwrap();
    py_run!(py, c, "c[1] = 2");
    assert_eq!(c.key, 1);
    assert_eq!(c.val, 2);
    py_expect_exception!(py, c, "del c[1]", NotImplementedError);
}

#[pyclass]
struct DelItem {
    key: i32,
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

    let c = py.init_ref(|_| DelItem { key: 0 }).unwrap();
    py_run!(py, c, "del c[1]");
    assert_eq!(c.key, 1);
    py_expect_exception!(py, c, "c[1] = 2", NotImplementedError);
}

#[pyclass]
struct SetDelItem {
    val: Option<i32>,
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

    let c = py.init_ref(|_| SetDelItem { val: None }).unwrap();
    py_run!(py, c, "c[1] = 2");
    assert_eq!(c.val, Some(2));
    py_run!(py, c, "del c[1]");
    assert_eq!(c.val, None);
}

#[pyclass]
struct Reversed {}

#[pyproto]
impl PyMappingProtocol for Reversed {
    fn __reversed__(&self) -> PyResult<&'static str> {
        Ok("I am reversed")
    }
}

#[test]
fn reversed() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init(|_| Reversed {}).unwrap();
    py_run!(py, c, "assert reversed(c) == 'I am reversed'");
}

#[pyclass]
struct Contains {}

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

    let c = py.init(|_| Contains {}).unwrap();
    py_run!(py, c, "assert 1 in c");
    py_run!(py, c, "assert -1 not in c");
    py_expect_exception!(py, c, "assert 'wrong type' not in c", TypeError);
}

#[pyclass]
struct ContextManager {
    exit_called: bool,
}

#[pyproto]
impl<'p> PyContextProtocol<'p> for ContextManager {
    fn __enter__(&mut self) -> PyResult<i32> {
        Ok(42)
    }

    fn __exit__(
        &mut self,
        ty: Option<&'p PyType>,
        _value: Option<&'p PyObjectRef>,
        _traceback: Option<&'p PyObjectRef>,
    ) -> PyResult<bool> {
        let gil = GILGuard::acquire();
        self.exit_called = true;
        if ty == Some(gil.python().get_type::<ValueError>()) {
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

    let c = py
        .init_mut(|_| ContextManager { exit_called: false })
        .unwrap();
    py_run!(py, c, "with c as x: assert x == 42");
    assert!(c.exit_called);

    c.exit_called = false;
    py_run!(py, c, "with c as x: raise ValueError");
    assert!(c.exit_called);

    c.exit_called = false;
    py_expect_exception!(
        py,
        c,
        "with c as x: raise NotImplementedError",
        NotImplementedError
    );
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
struct Test {}

#[pyproto]
impl<'p> PyMappingProtocol<'p> for Test {
    fn __getitem__(&self, idx: &PyObjectRef) -> PyResult<PyObject> {
        let gil = GILGuard::acquire();
        if let Ok(slice) = idx.cast_as::<PySlice>() {
            let indices = slice.indices(1000)?;
            if indices.start == 100 && indices.stop == 200 && indices.step == 1 {
                return Ok("slice".into_object(gil.python()));
            }
        } else if let Ok(idx) = idx.extract::<isize>() {
            if idx == 1 {
                return Ok("int".into_object(gil.python()));
            }
        }
        Err(PyErr::new::<ValueError, _>("error"))
    }
}

#[test]
fn test_cls_impl() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let ob = py.init(|_| Test {}).unwrap();
    let d = PyDict::new(py);
    d.set_item("ob", ob).unwrap();

    py.run("assert ob[1] == 'int'", None, Some(d)).unwrap();
    py.run("assert ob[100:200:1] == 'slice'", None, Some(d))
        .unwrap();
}

#[pyclass(dict)]
struct DunderDictSupport {}

#[test]
fn dunder_dict_support() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = Py::new_ref(py, |_| DunderDictSupport {}).unwrap();
    py_run!(
        py,
        inst,
        r#"
        inst.a = 1
        assert inst.a == 1
    "#
    );
}

#[pyclass(weakref, dict)]
struct WeakRefDunderDictSupport {}

#[test]
fn weakref_dunder_dict_support() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = Py::new_ref(py, |_| WeakRefDunderDictSupport {}).unwrap();
    py_run!(
        py,
        inst,
        "import weakref; assert weakref.ref(inst)() is inst; inst.a = 1; assert inst.a == 1"
    );
}
