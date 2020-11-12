use pyo3::class::PySequenceProtocol;
use pyo3::exceptions::{PyIndexError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyList};

use pyo3::py_run;

mod common;

#[pyclass]
struct ByteSequence {
    elements: Vec<u8>,
}

#[pymethods]
impl ByteSequence {
    #[new]
    fn new(elements: Option<&PyList>) -> PyResult<Self> {
        if let Some(pylist) = elements {
            let mut elems = Vec::with_capacity(pylist.len());
            for pyelem in pylist.into_iter() {
                let elem = u8::extract(pyelem)?;
                elems.push(elem);
            }
            Ok(Self { elements: elems })
        } else {
            Ok(Self {
                elements: Vec::new(),
            })
        }
    }
}

#[pyproto]
impl PySequenceProtocol for ByteSequence {
    fn __len__(&self) -> usize {
        self.elements.len()
    }

    fn __getitem__(&self, idx: isize) -> PyResult<u8> {
        self.elements
            .get(idx as usize)
            .copied()
            .ok_or_else(|| PyIndexError::new_err("list index out of range"))
    }

    fn __setitem__(&mut self, idx: isize, value: u8) {
        self.elements[idx as usize] = value;
    }

    fn __delitem__(&mut self, idx: isize) -> PyResult<()> {
        if (idx < self.elements.len() as isize) && (idx >= 0) {
            self.elements.remove(idx as usize);
            Ok(())
        } else {
            Err(PyIndexError::new_err("list index out of range"))
        }
    }

    fn __contains__(&self, other: &PyAny) -> bool {
        match u8::extract(other) {
            Ok(x) => self.elements.contains(&x),
            Err(_) => false,
        }
    }

    fn __concat__(&self, other: PyRef<'p, Self>) -> Self {
        let mut elements = self.elements.clone();
        elements.extend_from_slice(&other.elements);
        Self { elements }
    }

    fn __repeat__(&self, count: isize) -> PyResult<Self> {
        if count >= 0 {
            let mut elements = Vec::with_capacity(self.elements.len() * count as usize);
            for _ in 0..count {
                elements.extend(&self.elements);
            }
            Ok(Self { elements })
        } else {
            Err(PyValueError::new_err("invalid repeat count"))
        }
    }
}

#[test]
fn test_getitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let d = [("ByteSequence", py.get_type::<ByteSequence>())].into_py_dict(py);

    let run = |code| py.run(code, None, Some(d)).unwrap();
    let err = |code| py.run(code, None, Some(d)).unwrap_err();

    run("s = ByteSequence([1, 2, 3]); assert s[0] == 1");
    run("s = ByteSequence([1, 2, 3]); assert s[1] == 2");
    run("s = ByteSequence([1, 2, 3]); assert s[2] == 3");
    err("s = ByteSequence([1, 2, 3]); print(s[-4])");
    err("s = ByteSequence([1, 2, 3]); print(s[4])");
}

#[test]
fn test_setitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let d = [("ByteSequence", py.get_type::<ByteSequence>())].into_py_dict(py);

    let run = |code| py.run(code, None, Some(d)).unwrap();
    let err = |code| py.run(code, None, Some(d)).unwrap_err();

    run("s = ByteSequence([1, 2, 3]); s[0] = 4; assert list(s) == [4, 2, 3]");
    err("s = ByteSequence([1, 2, 3]); s[0] = 'hello'");
}

#[test]
fn test_delitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = [("ByteSequence", py.get_type::<ByteSequence>())].into_py_dict(py);
    let run = |code| py.run(code, None, Some(d)).unwrap();
    let err = |code| py.run(code, None, Some(d)).unwrap_err();

    run("s = ByteSequence([1, 2, 3]); del s[0]; assert list(s) == [2, 3]");
    run("s = ByteSequence([1, 2, 3]); del s[1]; assert list(s) == [1, 3]");
    run("s = ByteSequence([1, 2, 3]); del s[-1]; assert list(s) == [1, 2]");
    run("s = ByteSequence([1, 2, 3]); del s[-2]; assert list(s) == [1, 3]");
    err("s = ByteSequence([1, 2, 3]); del s[-4]; print(list(s))");
    err("s = ByteSequence([1, 2, 3]); del s[4]");
}

#[test]
fn test_contains() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = [("ByteSequence", py.get_type::<ByteSequence>())].into_py_dict(py);
    let run = |code| py.run(code, None, Some(d)).unwrap();

    run("s = ByteSequence([1, 2, 3]); assert 1 in s");
    run("s = ByteSequence([1, 2, 3]); assert 2 in s");
    run("s = ByteSequence([1, 2, 3]); assert 3 in s");
    run("s = ByteSequence([1, 2, 3]); assert 4 not in s");
    run("s = ByteSequence([1, 2, 3]); assert 'hello' not in s");
}

#[test]
fn test_concat() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = [("ByteSequence", py.get_type::<ByteSequence>())].into_py_dict(py);
    let run = |code| py.run(code, None, Some(d)).unwrap();
    let err = |code| py.run(code, None, Some(d)).unwrap_err();

    run("s1 = ByteSequence([1, 2]); s2 = ByteSequence([3, 4]); assert list(s1+s2) == [1, 2, 3, 4]");
    err("s1 = ByteSequence([1, 2]); s2 = 'hello'; s1 + s2");
}

#[test]
fn test_inplace_concat() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = [("ByteSequence", py.get_type::<ByteSequence>())].into_py_dict(py);
    let run = |code| py.run(code, None, Some(d)).unwrap();
    let err = |code| py.run(code, None, Some(d)).unwrap_err();

    run("s = ByteSequence([1, 2]); s += ByteSequence([3, 4]); assert list(s) == [1, 2, 3, 4]");
    err("s = ByteSequence([1, 2]); s += 'hello'");
}

#[test]
fn test_repeat() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = [("ByteSequence", py.get_type::<ByteSequence>())].into_py_dict(py);
    let run = |code| py.run(code, None, Some(d)).unwrap();
    let err = |code| py.run(code, None, Some(d)).unwrap_err();

    run("s1 = ByteSequence([1, 2, 3]); s2 = s1*2; assert list(s2) == [1, 2, 3, 1, 2, 3]");
    err("s1 = ByteSequence([1, 2, 3]); s2 = s1*-1; assert list(s2) == [1, 2, 3, 1, 2, 3]");
}

#[test]
fn test_inplace_repeat() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = [("ByteSequence", py.get_type::<ByteSequence>())].into_py_dict(py);
    let run = |code| py.run(code, None, Some(d)).unwrap();
    let err = |code| py.run(code, None, Some(d)).unwrap_err();

    run("s = ByteSequence([1, 2]); s *= 3; assert list(s) == [1, 2, 1, 2, 1, 2]");
    err("s = ByteSequence([1, 2); s *= -1");
}

// Check that #[pyo3(get, set)] works correctly for Vec<PyObject>

#[pyclass]
struct GenericList {
    #[pyo3(get, set)]
    items: Vec<PyObject>,
}

#[test]
fn test_generic_list_get() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let list: PyObject = GenericList {
        items: [1, 2, 3].iter().map(|i| i.to_object(py)).collect(),
    }
    .into_py(py);

    py_assert!(py, list, "list.items == [1, 2, 3]");
}

#[test]
fn test_generic_list_set() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let list = PyCell::new(py, GenericList { items: vec![] }).unwrap();

    py_run!(py, list, "list.items = [1, 2, 3]");
    assert_eq!(
        list.borrow().items,
        vec![1.to_object(py), 2.to_object(py), 3.to_object(py)]
    );
}

#[pyclass]
struct OptionList {
    #[pyo3(get, set)]
    items: Vec<Option<i64>>,
}

#[pyproto]
impl PySequenceProtocol for OptionList {
    fn __getitem__(&self, idx: isize) -> PyResult<Option<i64>> {
        match self.items.get(idx as usize) {
            Some(x) => Ok(*x),
            None => Err(PyIndexError::new_err("Index out of bounds")),
        }
    }
}

#[test]
fn test_option_list_get() {
    // Regression test for #798
    let gil = Python::acquire_gil();
    let py = gil.python();

    let list = PyCell::new(
        py,
        OptionList {
            items: vec![Some(1), None],
        },
    )
    .unwrap();

    py_assert!(py, list, "list[0] == 1");
    py_assert!(py, list, "list[1] == None");
    py_expect_exception!(py, list, "list[2]", PyIndexError);
}
