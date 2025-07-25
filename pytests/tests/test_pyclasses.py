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
        raise TypeError(
            f"cannot create '{cls.__module__}.{cls.__qualname__}' instances"
        )


@pytest.mark.parametrize(
    "cls, exc_message",
    [
        (
            pyclasses.ClassWithoutConstructor,
            "cannot create 'builtins.ClassWithoutConstructor' instances",
        ),
        (
            ClassWithoutConstructor,
            "cannot create 'test_pyclasses.ClassWithoutConstructor' instances",
        ),
    ],
)
def test_no_constructor_defined_propagates_cause(cls: Type, exc_message: str):
    original_error = ValueError("Original message")
    with pytest.raises(Exception) as exc_info:
        try:
            raise original_error
        except Exception:
            cls()  # should raise TypeError("No constructor defined for ...")

    assert exc_info.type is TypeError
    assert exc_info.value.args == (exc_message,)
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
