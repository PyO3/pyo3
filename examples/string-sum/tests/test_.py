import pytest
from string_sum import sum_as_string


def test_sum():
    a, b = 12, 42

    added = sum_as_string(a, b)
    assert added == "54"


def test_err1():
    a, b = "abc", 42

    with pytest.raises(
        TypeError, match="sum_as_string expected an int for positional argument 1"
    ) as e:
        sum_as_string(a, b)


def test_err2():
    a, b = 0, {}

    with pytest.raises(
        TypeError, match="sum_as_string expected an int for positional argument 2"
    ) as e:
        sum_as_string(a, b)


def test_overflow1():
    a, b = 0, 1 << 43

    with pytest.raises(OverflowError, match="cannot fit 8796093022208 in 32 bits") as e:
        sum_as_string(a, b)


def test_overflow2():
    a, b = 1 << 30, 1 << 30

    with pytest.raises(OverflowError, match="arguments too large to add") as e:
        sum_as_string(a, b)
