import platform

import pytest
from hypothesis import given, assume
from hypothesis import strategies as st
from rustapi_module import othermod

INTEGER32_ST = st.integers(min_value=(-(2 ** 31)), max_value=(2 ** 31 - 1))
USIZE_ST = st.integers(min_value=othermod.USIZE_MIN, max_value=othermod.USIZE_MAX)

PYPY = platform.python_implementation() == "PyPy"


@given(x=INTEGER32_ST)
def test_double(x):
    expected = x * 2
    assume(-(2 ** 31) <= expected <= (2 ** 31 - 1))
    assert othermod.double(x) == expected


@pytest.mark.xfail(PYPY, reason="classes not properly working yet")
def test_modclass():
    # Test that the repr of the class itself doesn't crash anything
    repr(othermod.ModClass)

    assert isinstance(othermod.ModClass, type)


@pytest.mark.xfail(PYPY, reason="classes not properly working yet")
def test_modclass_instance():
    mi = othermod.ModClass()

    repr(mi)
    repr(mi.__class__)

    assert isinstance(mi, othermod.ModClass)
    assert isinstance(mi, object)


@pytest.mark.xfail(PYPY, reason="classes not properly working yet")
@given(x=USIZE_ST)
def test_modclas_noop(x):
    mi = othermod.ModClass()

    assert mi.noop(x) == x
