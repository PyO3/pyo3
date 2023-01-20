from pyo3_pytests import deprecated_pyfunctions as pyfunctions


def none_py():
    return None


def test_none_rs():
    rust = pyfunctions.none()
    py = none_py()
    assert rust == py


def simple_py(a, b="bar", *, c=None):
    return a, b, c


def test_simple_rs():
    rust = pyfunctions.simple(1, "foo", c={1: 2})
    py = simple_py(1, "foo", c={1: 2})
    assert rust == py


def simple_args_py(a, b="bar", *args, c=None):
    return a, b, args, c


def test_simple_args_rs():
    rust = pyfunctions.simple_args(1, "foo", 4, 5, 6, c={1: 2})
    py = simple_args_py(1, "foo", 4, 5, 6, c={1: 2})
    assert rust == py


def simple_kwargs_py(a, b="bar", c=None, **kwargs):
    return a, b, c, kwargs


def test_simple_kwargs_rs():
    rust = pyfunctions.simple_kwargs(1, "foo", c={1: 2}, bar=4, foo=10)
    py = simple_kwargs_py(1, "foo", c={1: 2}, bar=4, foo=10)
    assert rust == py


def simple_args_kwargs_py(a, b="bar", *args, c=None, **kwargs):
    return (a, b, args, c, kwargs)


def test_simple_args_kwargs_rs():
    rust = pyfunctions.simple_args_kwargs(1, "foo", "baz", bar=4, foo=10)
    py = simple_args_kwargs_py(1, "foo", "baz", bar=4, foo=10)
    assert rust == py


def args_kwargs_py(*args, **kwargs):
    return (args, kwargs)


def test_args_kwargs_rs():
    rust = pyfunctions.args_kwargs(1, "foo", {1: 2}, bar=4, foo=10)
    py = args_kwargs_py(1, "foo", {1: 2}, bar=4, foo=10)
    assert rust == py
