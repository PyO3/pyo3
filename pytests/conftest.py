import sysconfig
import sys
import pytest

FREE_THREADED_BUILD = bool(sysconfig.get_config_var("Py_GIL_DISABLED"))

gil_enabled_at_start = True
if FREE_THREADED_BUILD:
    gil_enabled_at_start = sys._is_gil_enabled()


def pytest_terminal_summary(terminalreporter, exitstatus, config):
    if FREE_THREADED_BUILD and not gil_enabled_at_start and sys._is_gil_enabled():
        tr = terminalreporter
        tr.ensure_newline()
        tr.section("GIL re-enabled", sep="=", red=True, bold=True)
        tr.line("The GIL was re-enabled at runtime during the tests.")
        tr.line("")
        tr.line("Please ensure all new modules declare support for running")
        tr.line("without the GIL. Any new tests that intentionally imports ")
        tr.line("code that re-enables the GIL should do so in a subprocess.")
        pytest.exit("GIL re-enabled during tests", returncode=1)
