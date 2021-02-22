import pyo3_benchmarks


def test_args_and_kwargs(benchmark):
    benchmark(pyo3_benchmarks.args_and_kwargs, 1, 2, 3, a=4, foo=10)


def args_and_kwargs_py(*args, **kwargs):
    return (args, kwargs)


def test_args_and_kwargs_py(benchmark):
    rust = pyo3_benchmarks.args_and_kwargs(1, 2, 3, bar=4, foo=10)
    py = args_and_kwargs_py(1, 2, 3, bar=4, foo=10)
    assert rust == py
    benchmark(args_and_kwargs_py, 1, 2, 3, bar=4, foo=10)


def test_mixed_args(benchmark):
    benchmark(pyo3_benchmarks.mixed_args, 1, 2, 3, bar=4, foo=10)


def mixed_args_py(a, b=2, *args, c=4, **kwargs):
    return (a, b, args, c, kwargs)


def test_mixed_args_py(benchmark):
    rust = pyo3_benchmarks.mixed_args(1, 2, 3, bar=4, foo=10)
    py = mixed_args_py(1, 2, 3, bar=4, foo=10)
    assert rust == py
    benchmark(mixed_args_py, 1, 2, 3, bar=4, foo=10)


def test_no_args(benchmark):
    benchmark(pyo3_benchmarks.no_args)


def no_args_py():
    return None


def test_no_args_py(benchmark):
    rust = pyo3_benchmarks.no_args()
    py = no_args_py()
    assert rust == py
    benchmark(no_args_py)
