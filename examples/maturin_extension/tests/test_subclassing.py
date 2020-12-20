import platform

from maturin_extension import subclassing

PYPY = platform.python_implementation() == "PyPy"


class SomeSubClass(subclassing.Subclassable):
    def __str__(self):
        return "SomeSubclass"


def test_subclassing():
    if not PYPY:
        a = SomeSubClass()
        assert str(a) == "SomeSubclass"
        assert type(a) is SomeSubClass
