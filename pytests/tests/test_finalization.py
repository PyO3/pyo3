import gc
import sysconfig
from sys import implementation

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
def test_hammer_attaching_in_thread():
    loopy.append(misc.hammer_attaching_in_thread())


@pytest.mark.skipif(
    implementation.name == "graalpy",
    reason="graalpy aborts instead of unwinding the thread",
)
def test_detach_during_finalization():
    loopy.append(misc.detach_during_finalization())


@pytest.mark.skipif(
    implementation.name == "graalpy",
    reason="GraalPy drops pyclass instances without invoking tp_finalize on this path",
)
def test_pyclass_del_runs_during_finalization():
    misc.reset_del_drop_counts()
    obj = misc.DelDropProbe()
    del obj

    for _ in range(10):
        gc.collect()
        finalized, dropped = misc.del_drop_counts()
        if finalized == dropped == 1:
            break

    assert misc.del_drop_counts() == (1, 1)
