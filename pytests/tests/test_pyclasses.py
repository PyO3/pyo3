from typing import Type
import pytest
from pyo3_pytests import pyclasses


def test_empty_class_init(benchmark):
    benchmark(pyclasses.EmptyClass)


class EmptyClassPy:
    pass


def test_empty_class_init_py(benchmark):
    benchmark(EmptyClassPy)


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


class ClassWithoutConstructorPy:
    def __new__(cls):
        raise TypeError("No constructor defined")


@pytest.mark.parametrize(
    "cls", [pyclasses.ClassWithoutConstructor, ClassWithoutConstructorPy]
)
def test_no_constructor_defined_propagates_cause(cls: Type):
    original_error = ValueError("Original message")
    with pytest.raises(Exception) as exc_info:
        try:
            raise original_error
        except Exception:
            cls()  # should raise TypeError("No constructor defined")

    assert exc_info.type is TypeError
    assert exc_info.value.args == ("No constructor defined",)
    assert exc_info.value.__context__ is original_error
