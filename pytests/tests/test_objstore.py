import gc
import sys

from pyo3_pytests.objstore import ObjStore


def test_objstore_doesnot_leak_memory():
    N = 10000
    message = b'\\(-"-;) Praying that memory leak would not happen..'

    # PyPy does not have sys.getrefcount, provide a no-op lambda and don't
    # check refcount on PyPy
    getrefcount = getattr(sys, "getrefcount", lambda obj: 0)

    if sys.implementation.name == "graalpy":
        # GraalPy has an incomplete sys.getrefcount implementation
        def getrefcount(obj):
            return 0

    before = getrefcount(message)
    store = ObjStore()
    for _ in range(N):
        store.push(message)
    del store
    gc.collect()
    after = getrefcount(message)

    assert after - before == 0
