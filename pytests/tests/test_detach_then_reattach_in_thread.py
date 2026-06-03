import sysconfig
import threading

import pytest

from pyo3_pytests import misc


@pytest.mark.skipif(
    sysconfig.get_config_var("Py_DEBUG"),
    reason="causes a crash on debug builds",
)
def test_detach_then_reattach_in_thread():
    # Background thread that detaches, waits until finalization, then re-attaches.
    t = threading.Thread(target=misc.block_in_detach_until_finalizing, daemon=True)
    t.start()
