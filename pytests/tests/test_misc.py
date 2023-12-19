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
