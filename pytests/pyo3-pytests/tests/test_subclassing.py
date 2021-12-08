import platform

from pyo3_pytests.subclassing import Subclassable


class SomeSubClass(Subclassable):
    def __str__(self):
        return "SomeSubclass"


def test_subclassing():
    a = SomeSubClass()
    assert str(a) == "SomeSubclass"
    assert type(a) is SomeSubClass
