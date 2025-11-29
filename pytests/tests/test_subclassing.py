from pyo3_pytests.subclassing import Subclassable, Subclass


class SomeSubClass(Subclassable):
    def __str__(self):
        return "SomeSubclass"


def test_python_subclassing():
    a = SomeSubClass()
    assert str(a) == "SomeSubclass"
    assert type(a) is SomeSubClass


def test_rust_subclassing():
    a = Subclass()
    assert str(a) == "Subclass"
    assert type(a) is Subclass
