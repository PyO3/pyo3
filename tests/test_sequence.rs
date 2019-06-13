use pyo3::class::PySequenceProtocol;
use pyo3::exceptions::IndexError;
use pyo3::exceptions::ValueError;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyAny;
use pyo3::types::PyList;

#[pyclass]
struct ByteSequence {
    elements: Vec<u8>,
}

#[pymethods]
impl ByteSequence {
    #[new]
    fn new(obj: &PyRawObject, elements: Option<&PyList>) -> PyResult<()> {
        if let Some(pylist) = elements {
            let mut elems = Vec::with_capacity(pylist.len());
            for pyelem in pylist.into_iter() {
                let elem = u8::extract(pyelem)?;
                elems.push(elem);
            }
            obj.init(Self { elements: elems });
        } else {
            obj.init(Self {
                elements: Vec::new(),
            });
        }
        Ok(())
    }
}

#[pyproto]
impl PySequenceProtocol for ByteSequence {
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.elements.len())
    }

    fn __getitem__(&self, idx: isize) -> PyResult<u8> {
        self.elements
            .get(idx as usize)
            .map(|&byte| byte)
            .ok_or(IndexError::py_err("list index out of range"))
    }

    fn __setitem__(&mut self, idx: isize, value: u8) -> PyResult<()> {
        self.elements[idx as usize] = value;
        Ok(())
    }

    fn __delitem__(&mut self, idx: isize) -> PyResult<()> {
        if (idx < self.elements.len() as isize) && (idx >= 0) {
            self.elements.remove(idx as usize);
            Ok(())
        } else {
            Err(IndexError::py_err("list index out of range"))
        }
    }

    fn __contains__(&self, other: &PyAny) -> PyResult<bool> {
        match u8::extract(other) {
            Ok(ref x) => Ok(self.elements.contains(x)),
            Err(_) => Ok(false),
        }
    }

    fn __concat__(&self, other: &Self) -> PyResult<Self> {
        let mut elements = self.elements.clone();
        elements.extend_from_slice(&other.elements);
        Ok(Self { elements })
    }

    fn __repeat__(&self, count: isize) -> PyResult<Self> {
        if count >= 0 {
            let mut elements = Vec::with_capacity(self.elements.len() * count as usize);
            for _ in 0..count {
                elements.extend(&self.elements);
            }
            Ok(Self { elements })
        } else {
            Err(ValueError::py_err("invalid repeat count"))
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
