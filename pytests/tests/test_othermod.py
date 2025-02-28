from hypothesis import given, assume
from hypothesis import strategies as st

from pyo3_pytests import othermod

INTEGER31_ST = st.integers(min_value=(-(2**30)), max_value=(2**30 - 1))
USIZE_ST = st.integers(min_value=othermod.USIZE_MIN, max_value=othermod.USIZE_MAX)


# If the full 32 bits are used here, then you can get failures that look like this:
# hypothesis.errors.FailedHealthCheck: It looks like your strategy is filtering out a lot of data.
# Health check found 50 filtered examples but only 7 good ones.
#
# Limit the range to 31 bits to avoid this problem.
@given(x=INTEGER31_ST)
def test_double(x):
    expected = x * 2
    assume(-(2**31) <= expected <= (2**31 - 1))
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
