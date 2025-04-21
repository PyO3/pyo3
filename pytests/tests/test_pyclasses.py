import platform
from typing import Type

import pytest
from pyo3_pytests import pyclasses


def test_empty_class_init(benchmark):
    benchmark(pyclasses.EmptyClass)


def test_method_call(benchmark):
    obj = pyclasses.EmptyClass()
    assert benchmark(obj.method) is None


def test_proto_call(benchmark):
    obj = pyclasses.EmptyClass()
    assert benchmark(len, obj) == 0


class EmptyClassPy:
    def method(self):
        pass

    def __len__(self) -> int:
        return 0


def test_empty_class_init_py(benchmark):
    benchmark(EmptyClassPy)


def test_method_call_py(benchmark):
    obj = EmptyClassPy()
    assert benchmark(obj.method) == pyclasses.EmptyClass().method()


def test_proto_call_py(benchmark):
    obj = EmptyClassPy()
    assert benchmark(len, obj) == len(pyclasses.EmptyClass())


def test_iter():
    i = pyclasses.PyClassIter()
    assert next(i) == 1
    assert next(i) == 2
    assert next(i) == 3
    assert next(i) == 4
    assert next(i) == 5

    with pytest.raises(StopIteration) as excinfo:
        next(i)
    assert excinfo.value.value == "Ended"


@pytest.mark.skipif(
    platform.machine() in ["wasm32", "wasm64"],
    reason="not supporting threads in CI for WASM yet",
)
def test_parallel_iter():
    import concurrent.futures

    i = pyclasses.PyClassThreadIter()

    def func():
        next(i)

    # the second thread attempts to borrow a reference to the instance's
    # state while the first thread is still sleeping, so we trigger a
    # runtime borrow-check error
    with pytest.raises(RuntimeError, match="Already borrowed"):
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as tpe:
            futures = [tpe.submit(func), tpe.submit(func)]
            [f.result() for f in futures]


class AssertingSubClass(pyclasses.AssertingBaseClass):
    pass


def test_new_classmethod():
    # The `AssertingBaseClass` constructor errors if it is not passed the
    # relevant subclass.
    _ = AssertingSubClass(expected_type=AssertingSubClass)
    with pytest.raises(ValueError):
        _ = AssertingSubClass(expected_type=str)


class ClassWithoutConstructor:
    def __new__(cls):
        raise TypeError("No constructor defined for ClassWithoutConstructor")


@pytest.mark.parametrize(
    "cls", [pyclasses.ClassWithoutConstructor, ClassWithoutConstructor]
)
def test_no_constructor_defined_propagates_cause(cls: Type):
    original_error = ValueError("Original message")
    with pytest.raises(Exception) as exc_info:
        try:
            raise original_error
        except Exception:
            cls()  # should raise TypeError("No constructor defined for ...")

    assert exc_info.type is TypeError
    assert exc_info.value.args == (
        "No constructor defined for ClassWithoutConstructor",
    )
    assert exc_info.value.__context__ is original_error


def test_dict():
    try:
        ClassWithDict = pyclasses.ClassWithDict
    except AttributeError:
        pytest.skip("not defined using abi3 < 3.9")

    d = ClassWithDict()
    assert d.__dict__ == {}

    d.foo = 42
    assert d.__dict__ == {"foo": 42}
