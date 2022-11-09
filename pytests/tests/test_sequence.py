import pytest
import platform

from pyo3_pytests import sequence


def test_vec_from_list_i32():
    assert sequence.vec_to_vec_i32([1, 2, 3]) == [1, 2, 3]


def test_vec_from_list_pystring():
    assert sequence.vec_to_vec_pystring(["1", "2", "3"]) == ["1", "2", "3"]


def test_vec_from_bytes():
    assert sequence.vec_to_vec_i32(b"123") == [49, 50, 51]


def test_vec_from_str():
    with pytest.raises(TypeError):
        sequence.vec_to_vec_pystring("123")


@pytest.mark.skipif(
    platform.system() != "Linux" or platform.python_implementation() != "CPython",
    reason="Binary NumPy wheels are not available for all platforms and Python implementations",
)
def test_vec_from_array():
    import numpy

    assert sequence.vec_to_vec_i32(numpy.array([1, 2, 3])) == [1, 2, 3]


@pytest.mark.skipif(
    platform.system() != "Linux" or platform.python_implementation() != "CPython",
    reason="Binary NumPy wheels are not available for all platforms and Python implementations",
)
def test_rust_array_from_array():
    import numpy

    assert sequence.array_to_array_i32(numpy.array([1, 2, 3])) == [1, 2, 3]
