#![cfg(feature = "macros")]

use pyo3::exceptions::{PyIndexError, PyValueError};
use pyo3::types::{IntoPyDict, PyList, PyMapping, PySequence};
use pyo3::{ffi, prelude::*};

use pyo3::py_run;

mod test_utils;

#[pyclass]
struct ByteSequence {
    elements: Vec<u8>,
}

#[pymethods]
impl ByteSequence {
    #[new]
    #[pyo3(signature=(elements = None))]
    fn new(elements: Option<&Bound<'_, PyList>>) -> PyResult<Self> {
        if let Some(pylist) = elements {
            let mut elems = Vec::with_capacity(pylist.len());
            for pyelem in pylist {
                let elem = pyelem.extract()?;
                elems.push(elem);
            }
            Ok(Self { elements: elems })
        } else {
            Ok(Self {
                elements: Vec::new(),
            })
        }
    }

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

    fn __delitem__(&mut self, mut idx: isize) -> PyResult<()> {
        let self_len = self.elements.len() as isize;
        if idx < 0 {
            idx += self_len;
        }
        if (idx < self_len) && (idx >= 0) {
            self.elements.remove(idx as usize);
            Ok(())
        } else {
            Err(PyIndexError::new_err("list index out of range"))
        }
    }

    fn __contains__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.extract::<u8>() {
            Ok(x) => self.elements.contains(&x),
            Err(_) => false,
        }
    }

    fn __concat__(&self, other: &Self) -> Self {
        let mut elements = self.elements.clone();
        elements.extend_from_slice(&other.elements);
        Self { elements }
    }

    fn __inplace_concat__(mut slf: PyRefMut<'_, Self>, other: &Self) -> Py<Self> {
        slf.elements.extend_from_slice(&other.elements);
        slf.into()
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

    fn __inplace_repeat__(mut slf: PyRefMut<'_, Self>, count: isize) -> PyResult<Py<Self>> {
        if count >= 0 {
            let mut elements = Vec::with_capacity(slf.elements.len() * count as usize);
            for _ in 0..count {
                elements.extend(&slf.elements);
            }
            slf.elements = elements;
            Ok(slf.into())
        } else {
            Err(PyValueError::new_err("invalid repeat count"))
        }
    }
}

/// Return a dict with `s = ByteSequence([1, 2, 3])`.
fn seq_dict(py: Python<'_>) -> Bound<'_, pyo3::types::PyDict> {
    let d = [("ByteSequence", py.get_type::<ByteSequence>())]
        .into_py_dict(py)
        .unwrap();
    // Though we can construct `s` in Rust, let's test `__new__` works.
    py_run!(py, *d, "s = ByteSequence([1, 2, 3])");
    d
}

#[test]
fn test_getitem() {
    Python::attach(|py| {
        let d = seq_dict(py);

        py_assert!(py, *d, "s[0] == 1");
        py_assert!(py, *d, "s[1] == 2");
        py_assert!(py, *d, "s[2] == 3");
        py_expect_exception!(py, *d, "print(s[-4])", PyIndexError);
        py_expect_exception!(py, *d, "print(s[4])", PyIndexError);
    });
}

#[test]
fn test_setitem() {
    Python::attach(|py| {
        let d = seq_dict(py);

        py_run!(py, *d, "s[0] = 4; assert list(s) == [4, 2, 3]");
        py_expect_exception!(py, *d, "s[0] = 'hello'", PyTypeError);
    });
}

#[test]
fn test_delitem() {
    Python::attach(|py| {
        let d = [("ByteSequence", py.get_type::<ByteSequence>())]
            .into_py_dict(py)
            .unwrap();

        py_run!(
            py,
            *d,
            "s = ByteSequence([1, 2, 3]); del s[0]; assert list(s) == [2, 3]"
        );
        py_run!(
            py,
            *d,
            "s = ByteSequence([1, 2, 3]); del s[1]; assert list(s) == [1, 3]"
        );
        py_run!(
            py,
            *d,
            "s = ByteSequence([1, 2, 3]); del s[-1]; assert list(s) == [1, 2]"
        );
        py_run!(
            py,
            *d,
            "s = ByteSequence([1, 2, 3]); del s[-2]; assert list(s) == [1, 3]"
        );
        py_expect_exception!(
            py,
            *d,
            "s = ByteSequence([1, 2, 3]); del s[-4]; print(list(s))",
            PyIndexError
        );
        py_expect_exception!(
            py,
            *d,
            "s = ByteSequence([1, 2, 3]); del s[4]",
            PyIndexError
        );
    });
}

#[test]
fn test_contains() {
    Python::attach(|py| {
        let d = seq_dict(py);

        py_assert!(py, *d, "1 in s");
        py_assert!(py, *d, "2 in s");
        py_assert!(py, *d, "3 in s");
        py_assert!(py, *d, "4 not in s");
        py_assert!(py, *d, "'hello' not in s");
    });
}

#[test]
fn test_concat() {
    Python::attach(|py| {
        let d = seq_dict(py);

        py_run!(
        py,
        *d,
        "s1 = ByteSequence([1, 2]); s2 = ByteSequence([3, 4]); assert list(s1 + s2) == [1, 2, 3, 4]"
    );
        py_expect_exception!(
            py,
            *d,
            "s1 = ByteSequence([1, 2]); s2 = 'hello'; s1 + s2",
            PyTypeError
        );
    });
}

#[test]
fn test_inplace_concat() {
    Python::attach(|py| {
        let d = seq_dict(py);

        py_run!(
            py,
            *d,
            "s += ByteSequence([4, 5]); assert list(s) == [1, 2, 3, 4, 5]"
        );
        py_expect_exception!(py, *d, "s += 'hello'", PyTypeError);
    });
}

#[test]
fn test_repeat() {
    Python::attach(|py| {
        let d = seq_dict(py);

        py_run!(py, *d, "s2 = s * 2; assert list(s2) == [1, 2, 3, 1, 2, 3]");
        py_expect_exception!(py, *d, "s2 = s * -1", PyValueError);
    });
}

#[test]
fn test_inplace_repeat() {
    Python::attach(|py| {
        let d = [("ByteSequence", py.get_type::<ByteSequence>())]
            .into_py_dict(py)
            .unwrap();

        py_run!(
            py,
            *d,
            "s = ByteSequence([1, 2]); s *= 3; assert list(s) == [1, 2, 1, 2, 1, 2]"
        );
        py_expect_exception!(py, *d, "s = ByteSequence([1, 2]); s *= -1", PyValueError);
    });
}

// Check that #[pyo3(get, set)] works correctly for Vec<PyObject>

#[pyclass]
struct AnyObjectList {
    #[pyo3(get, set)]
    items: Vec<Py<PyAny>>,
}

#[test]
fn test_any_object_list_get() {
    Python::attach(|py| {
        let list = AnyObjectList {
            items: [1i32, 2, 3]
                .iter()
                .map(|i| i.into_pyobject(py).unwrap().into_any().unbind())
                .collect(),
        }
        .into_pyobject(py)
        .unwrap();

        py_assert!(py, list, "list.items == [1, 2, 3]");
    });
}

#[test]
fn test_any_object_list_set() {
    Python::attach(|py| {
        let list = Bound::new(py, AnyObjectList { items: vec![] }).unwrap();

        py_run!(py, list, "list.items = [1, 2, 3]");
        assert!(list
            .borrow()
            .items
            .iter()
            .zip(&[1u32, 2, 3])
            .all(|(a, b)| a.bind(py).eq(b.into_pyobject(py).unwrap()).unwrap()));
    });
}

#[pyclass(sequence)]
struct OptionList {
    #[pyo3(get, set)]
    items: Vec<Option<i64>>,
}

#[pymethods]
impl OptionList {
    fn __len__(&self) -> usize {
        self.items.len()
    }

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
    Python::attach(|py| {
        let list = Py::new(
            py,
            OptionList {
                items: vec![Some(1), None],
            },
        )
        .unwrap();

        py_assert!(py, list, "list[0] == 1");
        py_assert!(py, list, "list[1] == None");
        py_expect_exception!(py, list, "list[2]", PyIndexError);
    });
}

#[test]
fn sequence_is_not_mapping() {
    Python::attach(|py| {
        let list = Bound::new(
            py,
            OptionList {
                items: vec![Some(1), None],
            },
        )
        .unwrap()
        .into_any();

        PySequence::register::<OptionList>(py).unwrap();

        assert!(list.cast::<PyMapping>().is_err());
        assert!(list.cast::<PySequence>().is_ok());
    })
}

#[test]
fn sequence_length() {
    Python::attach(|py| {
        let list = Bound::new(
            py,
            OptionList {
                items: vec![Some(1), None],
            },
        )
        .unwrap()
        .into_any();

        assert_eq!(list.len().unwrap(), 2);
        assert_eq!(unsafe { ffi::PySequence_Length(list.as_ptr()) }, 2);

        assert_eq!(unsafe { ffi::PyMapping_Length(list.as_ptr()) }, -1);
        unsafe { ffi::PyErr_Clear() };
    })
}

#[cfg(Py_3_10)]
#[pyclass(generic, sequence)]
struct GenericList {
    #[pyo3(get, set)]
    items: Vec<Py<PyAny>>,
}

#[cfg(Py_3_10)]
#[pymethods]
impl GenericList {
    fn __len__(&self) -> usize {
        self.items.len()
    }

    fn __getitem__(&self, idx: isize) -> PyResult<Py<PyAny>> {
        match self.items.get(idx as usize) {
            Some(x) => pyo3::Python::attach(|py| Ok(x.clone_ref(py))),
            None => Err(PyIndexError::new_err("Index out of bounds")),
        }
    }
}

#[cfg(Py_3_10)]
#[test]
fn test_generic_both_subscriptions_types() {
    use pyo3::types::PyInt;
    use std::convert::Infallible;

    Python::attach(|py| {
        let l = Bound::new(
            py,
            GenericList {
                items: [1, 2, 3]
                    .iter()
                    .map(|x| -> Py<PyAny> {
                        let x: Result<Bound<'_, PyInt>, Infallible> = x.into_pyobject(py);
                        x.unwrap().into_any().unbind()
                    })
                    .chain([py.None()])
                    .collect(),
            },
        )
        .unwrap();
        let ty = py.get_type::<GenericList>();
        py_assert!(py, l, "l[0] == 1");
        py_run!(
            py,
            ty,
            "import types;
            import typing;
            IntOrNone: typing.TypeAlias = typing.Union[int, None];
            assert ty[IntOrNone] == types.GenericAlias(ty, (IntOrNone,))"
        );
        py_assert!(py, l, "list(reversed(l)) == [None, 3, 2, 1]");
    });
}
