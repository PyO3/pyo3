#![feature(test)]

extern crate test;
use pyo3::prelude::*;
use pyo3::PyIterProtocol;
use test::Bencher;

#[pyclass]
pub struct MyClass {
    #[pyo3(get, set)]
    obj: PyObject,

    #[pyo3(get, set)]
    x: i32,

    values: Vec<usize>,
}

#[pyclass]
pub struct Iter {
    inner: std::vec::IntoIter<usize>,
}

#[pyproto]
impl PyIterProtocol for Iter {
    fn __next__(mut slf: PyRefMut<'p, Self>) -> PyResult<Option<usize>> {
        Ok(slf.inner.next())
    }
}

#[pyproto]
impl PyIterProtocol for MyClass {
    fn __iter__(slf: PyRef<'p, Self>) -> PyResult<Py<Iter>> {
        let iter = Iter {
            inner: slf.values.clone().into_iter(),
        };
        PyCell::new(slf.py(), iter).map(Into::into)
    }
}

#[bench]
fn create_many_pyclass(b: &mut Bencher) {
    b.iter(|| {
        for i in 0..20 {
            let gil = Python::acquire_gil();
            let py = gil.python();
            for j in 0..200 {
                let _ = PyCell::new(
                    py,
                    MyClass {
                        obj: py.None(),
                        x: i * j,
                        values: vec![],
                    },
                );
            }
        }
    });
}

#[bench]
fn iter_pyclass(b: &mut Bencher) {
    let mut sum = 0;

    b.iter(|| {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = PyCell::new(
            py,
            MyClass {
                obj: py.None(),
                x: 0,
                values: (0..10_000).into_iter().collect(),
            },
        )
        .unwrap()
        .to_object(py);

        for x in obj.as_ref(py).iter().unwrap() {
            sum += x.unwrap().extract::<usize>().unwrap();
        }
    });
}

#[bench]
fn get_property(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = PyCell::new(
        py,
        MyClass {
            obj: py.None(),
            x: 1,
            values: vec![],
        },
    )
    .unwrap()
    .to_object(py);
    let any = obj.as_ref(py);

    let mut sum = 0;

    b.iter(|| {
        for _ in 0..4_000 {
            sum += any.getattr("x").unwrap().extract::<usize>().unwrap();
        }
    });
}

#[bench]
fn set_property(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = PyCell::new(
        py,
        MyClass {
            obj: py.None(),
            x: 1,
            values: vec![],
        },
    )
    .unwrap()
    .to_object(py);
    let any = obj.as_ref(py);

    b.iter(|| {
        for _ in 0..4_000 {
            any.setattr("obj", py.None()).unwrap();
        }
    });
}
