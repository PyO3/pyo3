use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::{prelude::*, pyclass::CompareOp, Python};

#[pyclass]
struct OrderedDunderMethods(i64);

#[pymethods]
impl OrderedDunderMethods {
    fn __lt__(&self, other: &Self) -> bool {
        self.0 < other.0
    }

    fn __le__(&self, other: &Self) -> bool {
        self.0 <= other.0
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    fn __ne__(&self, other: &Self) -> bool {
        self.0 != other.0
    }

    fn __gt__(&self, other: &Self) -> bool {
        self.0 > other.0
    }

    fn __ge__(&self, other: &Self) -> bool {
        self.0 >= other.0
    }
}

#[pyclass]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct OrderedRichcmp(i64);

#[pymethods]
impl OrderedRichcmp {
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        op.matches(self.cmp(other))
    }
}

fn bench_ordered_dunder_methods(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let obj1 = PyDetached::new(py, OrderedDunderMethods(0))
            .unwrap()
            .into_ref(py);
        let obj2 = PyDetached::new(py, OrderedDunderMethods(1))
            .unwrap()
            .into_ref(py);

        b.iter(|| obj2.gt(obj1).unwrap());
    });
}

fn bench_ordered_richcmp(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let obj1 = PyDetached::new(py, OrderedRichcmp(0)).unwrap().into_ref(py);
        let obj2 = PyDetached::new(py, OrderedRichcmp(1)).unwrap().into_ref(py);

        b.iter(|| obj2.gt(obj1).unwrap());
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("ordered_dunder_methods", bench_ordered_dunder_methods);
    c.bench_function("ordered_richcmp", bench_ordered_richcmp);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
