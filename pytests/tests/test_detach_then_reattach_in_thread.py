import sysconfig
import threading

import pytest

from pyo3_pytests import misc


@pytest.mark.skipif(
    sysconfig.get_config_var("Py_DEBUG"),
    reason="causes a crash on debug builds",
)
def test_detach_then_reattach_in_thread():
    # A Python daemon thread calls block_in_detach_until_finalizing(), which releases
    # the GIL via py.detach() and polls Py_IsFinalizing() (safe without GIL — reads
    # an atomic). When the interpreter begins finalizing, the thread exits the closure
    # and SuspendAttach::drop() calls PyEval_RestoreThread() on a finalizing interpreter.
    t = threading.Thread(target=misc.block_in_detach_until_finalizing, daemon=True)
    t.start()
    # No join — thread polls until finalization
