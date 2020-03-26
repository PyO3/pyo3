import gc
import platform
import sys

import pytest

from rustapi_module.objstore import ObjStore

PYPY = platform.python_implementation() == "PyPy"


@pytest.mark.skipif(PYPY, reason="PyPy does not have sys.getrefcount")
def test_objstore_doesnot_leak_memory():
    N = 10000
    message = b'\\(-"-;) Praying that memory leak would not happen..'
    before = sys.getrefcount(message)
    store = ObjStore()
    for _ in range(N):
        store.push(message)
    del store
    gc.collect()
    after = sys.getrefcount(message)

    assert after - before == 0
