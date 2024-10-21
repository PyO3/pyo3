import pytest
import sys
import sysconfig

from pyo3_pytests import free_threaded_mod  # NOQA

GIL_DISABLED_BUILD = bool(sysconfig.get_config_var("Py_GIL_DISABLED"))


@pytest.mark.skipif(
    not GIL_DISABLED_BUILD, reason="test is not meaningful on GIL-enabled build"
)
def test_gil_disabled():
    assert not sys._is_gil_enabled()
