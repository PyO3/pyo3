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
    N = 1000
    extractor = BytesExtractor()
    process = psutil.Process(os.getpid())

    def memory_diff(f):
        before = process.memory_info().rss
        f()
        after = process.memory_info().rss
        return after - before

    message_b = b'\\(-"-;) Praying that memory leak would not happen..'
    message_s = '\\(-"-;) Praying that memory leak would not happen..'

    def to_vec():
        for i in range(N):
            extractor.to_vec(message_b)

    def to_str():
        for i in range(N):
            extractor.to_str(message_s)

    assert memory_diff(to_vec) == 0
    assert memory_diff(to_str) == 0
