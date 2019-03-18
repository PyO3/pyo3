import platform

import pytest

PYPY = platform.python_implementation() == "PyPy"

if not PYPY:
    from rustapi_module.subclassing import Subclassable


# should not raise
@pytest.mark.xfail(PYPY, reason="classes not properly working yet")
def test_subclassing_works():
    class SomeSubClass(Subclassable):
        pass

    a = SomeSubClass()
    _b = str(a) + repr(a)
