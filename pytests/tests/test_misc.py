import importlib
import platform
import sys

import pyo3_pytests.misc
import pytest


def test_issue_219():
    # Should not deadlock
    pyo3_pytests.misc.issue_219()


@pytest.mark.xfail(
    platform.python_implementation() == "CPython" and sys.version_info < (3, 9),
    reason="Cannot identify subinterpreters on Python older than 3.9",
)
def test_multiple_imports_same_interpreter_ok():
    spec = importlib.util.find_spec("pyo3_pytests.pyo3_pytests")

    module = importlib.util.module_from_spec(spec)
    assert dir(module) == dir(pyo3_pytests.pyo3_pytests)


@pytest.mark.xfail(
    platform.python_implementation() == "CPython" and sys.version_info < (3, 9),
    reason="Cannot identify subinterpreters on Python older than 3.9",
)
@pytest.mark.skipif(
    platform.python_implementation() == "PyPy",
    reason="PyPy does not support subinterpreters",
)
def test_import_in_subinterpreter_forbidden():
    import _xxsubinterpreters

    if sys.version_info < (3, 12):
        expected_error = "PyO3 modules do not yet support subinterpreters, see https://github.com/PyO3/pyo3/issues/576"
    else:
        expected_error = "module pyo3_pytests.pyo3_pytests does not support loading in subinterpreters"

    sub_interpreter = _xxsubinterpreters.create()
    with pytest.raises(
        _xxsubinterpreters.RunFailedError,
        match=expected_error,
    ):
        _xxsubinterpreters.run_string(
            sub_interpreter, "import pyo3_pytests.pyo3_pytests"
        )

    _xxsubinterpreters.destroy(sub_interpreter)


def test_type_full_name_includes_module():
    numpy = pytest.importorskip("numpy")

    assert pyo3_pytests.misc.get_type_full_name(numpy.bool_(True)) == "numpy.bool_"


def test_accepts_numpy_bool():
    # binary numpy wheel not available on all platforms
    numpy = pytest.importorskip("numpy")

    assert pyo3_pytests.misc.accepts_bool(True) is True
    assert pyo3_pytests.misc.accepts_bool(False) is False
    assert pyo3_pytests.misc.accepts_bool(numpy.bool_(True)) is True
    assert pyo3_pytests.misc.accepts_bool(numpy.bool_(False)) is False


class ArbitraryClass:
    worker_id: int
    iteration: int

    def __init__(self, worker_id: int, iteration: int):
        self.worker_id = worker_id
        self.iteration = iteration

    def __repr__(self):
        return f"ArbitraryClass({self.worker_id}, {self.iteration})"

    def __del__(self):
        print("del", self.worker_id, self.iteration)


def test_gevent():
    gevent = pytest.importorskip("gevent")

    def worker(worker_id: int) -> None:
        for iteration in range(2):
            d = {"key": ArbitraryClass(worker_id, iteration)}

            def arbitrary_python_code():
                # remove the dictionary entry so that the class value can be
                # garbage collected
                del d["key"]
                print("gevent sleep", worker_id, iteration)
                gevent.sleep(0)
                print("after gevent sleep", worker_id, iteration)

            print("start", worker_id, iteration)
            pyo3_pytests.misc.get_item_and_run_callback(d, arbitrary_python_code)
            print("end", worker_id, iteration)

    workers = [gevent.spawn(worker, i) for i in range(2)]
    gevent.joinall(workers)
