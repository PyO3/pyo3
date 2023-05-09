import importlib
import platform

import pyo3_pytests.misc
import pytest


def test_issue_219():
    # Should not deadlock
    pyo3_pytests.misc.issue_219()


@pytest.mark.skipif(
    platform.python_implementation() == "PyPy",
    reason="PyPy does not reinitialize the module (appears to be some internal caching)",
)
def test_second_module_import_fails():
    spec = importlib.util.find_spec("pyo3_pytests.pyo3_pytests")

    with pytest.raises(
        ImportError,
        match="PyO3 modules may only be initialized once per interpreter process",
    ):
        importlib.util.module_from_spec(spec)
