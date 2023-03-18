import pytest
from pyo3_pytests import pyfunctions


def none_py():
    return None


def test_none_py(benchmark):
    benchmark(none_py)


def test_none_rs(benchmark):
    rust = benchmark(pyfunctions.none)
    py = none_py()
    assert rust == py


def simple_py(a, b=None, *, c=None):
    return a, b, c


def test_simple_py(benchmark):
    benchmark(simple_py, 1, "foo", c={1: 2})


def test_simple_rs(benchmark):
    rust = benchmark(pyfunctions.simple, 1, "foo", c={1: 2})
    py = simple_py(1, "foo", c={1: 2})
    assert rust == py


def simple_args_py(a, b=None, *args, c=None):
    return a, b, args, c


def test_simple_args_py(benchmark):
    benchmark(simple_args_py, 1, "foo", 4, 5, 6, c={1: 2})


def test_simple_args_rs(benchmark):
    rust = benchmark(pyfunctions.simple_args, 1, "foo", 4, 5, 6, c={1: 2})
    py = simple_args_py(1, "foo", 4, 5, 6, c={1: 2})
    assert rust == py


def simple_kwargs_py(a, b=None, c=None, **kwargs):
    return a, b, c, kwargs


def test_simple_kwargs_py(benchmark):
    benchmark(simple_kwargs_py, 1, "foo", c={1: 2}, bar=4, foo=10)


def test_simple_kwargs_rs(benchmark):
    rust = benchmark(pyfunctions.simple_kwargs, 1, "foo", c={1: 2}, bar=4, foo=10)
    py = simple_kwargs_py(1, "foo", c={1: 2}, bar=4, foo=10)
    assert rust == py


def simple_args_kwargs_py(a, b=None, *args, c=None, **kwargs):
    return (a, b, args, c, kwargs)


def test_simple_args_kwargs_py(benchmark):
    benchmark(simple_args_kwargs_py, 1, "foo", "baz", bar=4, foo=10)


def test_simple_args_kwargs_rs(benchmark):
    rust = benchmark(pyfunctions.simple_args_kwargs, 1, "foo", "baz", bar=4, foo=10)
    py = simple_args_kwargs_py(1, "foo", "baz", bar=4, foo=10)
    assert rust == py


def args_kwargs_py(*args, **kwargs):
    return (args, kwargs)


def test_args_kwargs_py(benchmark):
    benchmark(args_kwargs_py, 1, "foo", {1: 2}, bar=4, foo=10)


def test_args_kwargs_rs(benchmark):
    rust = benchmark(pyfunctions.args_kwargs, 1, "foo", {1: 2}, bar=4, foo=10)
    py = args_kwargs_py(1, "foo", {1: 2}, bar=4, foo=10)
    assert rust == py

def test_panic():
    with pytest.raises(BaseException) as exc_info:
        pyfunctions.panic()
    assert exc_info.type.__name__ == "PanicException"
    assert exc_info.type.__bases__ == (Exception, )
    assert str(exc_info.value) == "Dodge this"
