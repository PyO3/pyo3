import gc
import os
import platform

import psutil
import pytest
from rustapi_module.buf_and_str import BytesExtractor

PYPY = platform.python_implementation() == "PyPy"


@pytest.mark.skipif(
    PYPY,
    reason="PyPy has a segfault bug around this area."
    "See https://github.com/PyO3/pyo3/issues/589 for detail.",
)
def test_pybuffer_doesnot_leak_memory():
    N = 10000
    extractor = BytesExtractor()
    process = psutil.Process(os.getpid())

    def memory_diff(f):
        before = process.memory_info().rss
        gc.collect()  # Trigger Garbage collection
        for _ in range(N):
            f()
        gc.collect()  # Trigger Garbage collection
        after = process.memory_info().rss
        return after - before

    message_b = b'\\(-"-;) Praying that memory leak would not happen..'
    message_s = '\\(-"-;) Praying that memory leak would not happen..'
    message_surrogate = '\\(-"-;) Praying that memory leak would not happen.. \ud800'

    def from_bytes():
        extractor.from_bytes(message_b)

    def from_str():
        extractor.from_str(message_s)

    def from_str_lossy():
        extractor.from_str_lossy(message_surrogate)

    # Running the memory_diff to warm-up the garbage collector
    memory_diff(from_bytes)
    memory_diff(from_str)
    memory_diff(from_str_lossy)

    assert memory_diff(from_bytes) == 0
    assert memory_diff(from_str) == 0
    assert memory_diff(from_str_lossy) == 0
