import platform

from rustapi_module.subclassing import Subclassable

PYPY = platform.python_implementation() == "PyPy"


class SomeSubClass(Subclassable):
    def __str__(self):
        return "Subclass"


def test_subclassing():
    if not PYPY:
        a = SomeSubClass()
        assert str(a) == "Subclass"
