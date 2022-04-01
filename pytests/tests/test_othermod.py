from hypothesis import given, assume
from hypothesis import strategies as st

from pyo3_pytests import othermod

INTEGER32_ST = st.integers(min_value=(-(2 ** 31)), max_value=(2 ** 31 - 1))
USIZE_ST = st.integers(min_value=othermod.USIZE_MIN, max_value=othermod.USIZE_MAX)


@given(x=INTEGER32_ST)
def test_double(x):
    expected = x * 2
    assume(-(2 ** 31) <= expected <= (2 ** 31 - 1))
    assert othermod.double(x) == expected


def test_modclass():
    # Test that the repr of the class itself doesn't crash anything
    repr(othermod.ModClass)

    assert isinstance(othermod.ModClass, type)


def test_modclass_instance():
    mi = othermod.ModClass()

    repr(mi)
    repr(mi.__class__)

    assert isinstance(mi, othermod.ModClass)
    assert isinstance(mi, object)


@given(x=USIZE_ST)
def test_modclas_noop(x):
    mi = othermod.ModClass()

    assert mi.noop(x) == x
