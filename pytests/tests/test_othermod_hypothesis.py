import pytest
from pyo3_pytests import othermod

hypothesis = pytest.importorskip("hypothesis")
st = pytest.importorskip("hypothesis.strategies")

INTEGER31_ST = st.integers(min_value=(-(2**30)), max_value=(2**30 - 1))
USIZE_ST = st.integers(min_value=othermod.USIZE_MIN, max_value=othermod.USIZE_MAX)


# If the full 32 bits are used here, then you can get failures that look like this:
# hypothesis.errors.FailedHealthCheck: It looks like your strategy is filtering out a lot of data.
# Health check found 50 filtered examples but only 7 good ones.
#
# Limit the range to 31 bits to avoid this problem.
@hypothesis.given(x=INTEGER31_ST)
def test_double(x):
    expected = x * 2
    hypothesis.assume(-(2**31) <= expected <= (2**31 - 1))
    assert othermod.double(x) == expected


@hypothesis.given(x=USIZE_ST)
def test_modclas_noop(x):
    mi = othermod.ModClass()

    assert mi.noop(x) == x
