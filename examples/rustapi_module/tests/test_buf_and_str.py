import os
import psutil
from rustapi_module.buf_and_str import BytesExtractor


def test_pybuffer_doesnot_leak_memory():
    N = int(1e5)
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

    mv = memory_diff(to_vec)
    ms = memory_diff(to_str)
    assert abs(mv - ms) < 1000
