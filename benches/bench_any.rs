use criterion::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::{
    types::{
        PyBool, PyByteArray, PyBytes, PyDict, PyFloat, PyFrozenSet, PyInt, PyList, PyMapping,
        PySequence, PySet, PyString, PyTuple,
    },
    PyAny, PyResult, Python,
};

#[derive(PartialEq, Eq, Debug)]
enum ObjectType {
    None,
    Bool,
    ByteArray,
    Bytes,
    Dict,
    Float,
    FrozenSet,
    Int,
    List,
    Set,
    Str,
    Tuple,
    Sequence,
    Mapping,
    Unknown,
}

fn find_object_type(obj: &PyAny) -> ObjectType {
    if obj.is_none() {
        ObjectType::None
    } else if obj.is_instance_of::<PyBool>() {
        ObjectType::Bool
    } else if obj.is_instance_of::<PyByteArray>() {
        ObjectType::ByteArray
    } else if obj.is_instance_of::<PyBytes>() {
        ObjectType::Bytes
    } else if obj.is_instance_of::<PyDict>() {
        ObjectType::Dict
    } else if obj.is_instance_of::<PyFloat>() {
        ObjectType::Float
    } else if obj.is_instance_of::<PyFrozenSet>() {
        ObjectType::FrozenSet
    } else if obj.is_instance_of::<PyInt>() {
        ObjectType::Int
    } else if obj.is_instance_of::<PyList>() {
        ObjectType::List
    } else if obj.is_instance_of::<PySet>() {
        ObjectType::Set
    } else if obj.is_instance_of::<PyString>() {
        ObjectType::Str
    } else if obj.is_instance_of::<PyTuple>() {
        ObjectType::Tuple
    } else if obj.downcast::<PySequence>().is_ok() {
        ObjectType::Sequence
    } else if obj.downcast::<PyMapping>().is_ok() {
        ObjectType::Mapping
    } else {
        ObjectType::Unknown
    }
}

fn bench_identify_object_type(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let obj = py.eval("object()", None, None).unwrap();

        b.iter(|| find_object_type(obj));

        assert_eq!(find_object_type(obj), ObjectType::Unknown);
    });
}

fn bench_collect_generic_iterator(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let collection = py.eval("list(range(1 << 20))", None, None).unwrap();

        b.iter(|| {
            collection
                .iter()
                .unwrap()
                .collect::<PyResult<Vec<_>>>()
                .unwrap()
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("identify_object_type", bench_identify_object_type);
    c.bench_function("collect_generic_iterator", bench_collect_generic_iterator);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
