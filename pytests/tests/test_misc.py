import importlib
import platform

import pyo3_pytests.misc
import pytest


def test_issue_219():
    # Should not deadlock
    pyo3_pytests.misc.issue_219()


def test_multiple_imports_same_interpreter_ok():
    spec = importlib.util.find_spec("pyo3_pytests.pyo3_pytests")

    module = importlib.util.module_from_spec(spec)
    assert dir(module) == dir(pyo3_pytests.pyo3_pytests)


@pytest.mark.skipif(
    platform.python_implementation() == "PyPy",
    reason="PyPy does not support subinterpreters",
)
def test_import_in_subinterpreter_forbidden():
    import _xxsubinterpreters

    sub_interpreter = _xxsubinterpreters.create()
    with pytest.raises(
        _xxsubinterpreters.RunFailedError,
        match="PyO3 modules do not yet support subinterpreters, see https://github.com/PyO3/pyo3/issues/576",
    ):
        _xxsubinterpreters.run_string(
            sub_interpreter, "import pyo3_pytests.pyo3_pytests"
        )

    _xxsubinterpreters.destroy(sub_interpreter)
