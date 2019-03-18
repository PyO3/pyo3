import platform

PYPY = platform.python_implementation() == "PyPy"

if not PYPY:
    from rustapi_module.test_dict import DictSize

import pytest


@pytest.mark.xfail(PYPY, reason="classes not properly working yet")
@pytest.mark.parametrize(
    "size",
    [64, 128, 256],
)
def test_size(size):
    d = {}
    for i in range(size):
        d[i] = str(i)
    assert DictSize(len(d)).iter_dict(d) == size
