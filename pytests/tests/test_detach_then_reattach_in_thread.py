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


# Keep the LockHolder alive until interpreter finalization. A plain module-level
# variable would be dropped (via refcount) during module cleanup, before the
# interpreter is marked as finalizing. The reference cycle here is ineligible for
# refcount collection and is instead collected by the cyclic GC during finalization,
# at which point _Py_IsInterpreterFinalizing() is true and PyEval_RestoreThread()
# triggers pthread_exit through our Rust frames.
loopy = [make_loop()]


@pytest.mark.skipif(
    sysconfig.get_config_var("Py_DEBUG"),
    reason="causes a crash on debug builds",
)
def test_detach_then_reattach_in_thread():
    # A thread releases the GIL via py.detach() and blocks. When the interpreter
    # begins finalizing and drops the LockHolder, the thread unblocks and
    # SuspendAttach::drop() calls PyEval_RestoreThread() on a finalizing interpreter.
    # In buggy versions of PyO3 this crashes because PyEval_RestoreThread is not
    # guarded the same way PyGILState_Ensure is (see https://github.com/PyO3/pyo3/pull/4874).
    loopy.append(misc.detach_then_reattach_in_thread())
