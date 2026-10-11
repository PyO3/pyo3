from pyo3_pytests import exception


def test_pypy_exception_dealloc():
    # See https://github.com/pypy/pypy/issues/5555
    # - we had an issue caused by https://github.com/PyO3/pyo3/pull/6224
    for _ in range(10_000):
        instance = exception.MyValueErrorClass()
        del instance
