import sysconfig

import pytest

from pyo3_pytests import misc


def make_loop():
    # create a reference loop that will only be destroyed when the GC is called at the end
    # of execution
    start = []
    cur = [start]
    for _ in range(1000 * 1000 * 10):
        cur = [cur]
    start.append(cur)
    return start


# set a bomb that will explode when modules are cleaned up
loopy = [make_loop()]


@pytest.mark.skipif(
    sysconfig.get_config_var("Py_DEBUG"),
    reason="causes a crash on debug builds, see discussion in https://github.com/PyO3/pyo3/pull/4874",
)
def test_hammer_gil():
    loopy.append(misc.hammer_gil_in_thread())
