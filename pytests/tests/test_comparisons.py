from typing import Type, Union

import pytest
from pyo3_pytests.comparisons import (
    Eq,
    EqDefaultNe,
    EqDerived,
    Ordered,
    OrderedDefaultNe,
)
from typing_extensions import Self


class PyEq:
    def __init__(self, x: int) -> None:
        self.x = x

    def __eq__(self, other: object) -> bool:
        if isinstance(other, self.__class__):
            return self.x == other.x
        else:
            return NotImplemented

    def __ne__(self, other: Self) -> bool:
        if isinstance(other, self.__class__):
            return self.x != other.x
        else:
            return NotImplemented


@pytest.mark.parametrize(
    "ty", (Eq, EqDerived, PyEq), ids=("rust", "rust-derived", "python")
)
def test_eq(ty: Type[Union[Eq, EqDerived, PyEq]]):
    a = ty(0)
    b = ty(0)
    c = ty(1)

    assert a == b
    assert not (a != b)
    assert a != c
    assert not (a == c)

    assert b == a
    assert not (a != b)
    assert b != c
    assert not (b == c)

    assert not a == 0
    assert a != 0
    assert not b == 0
    assert b != 1
    assert not c == 1
    assert c != 1

    with pytest.raises(TypeError):
        assert a <= b

    with pytest.raises(TypeError):
        assert a >= b

    with pytest.raises(TypeError):
        assert a < c

    with pytest.raises(TypeError):
        assert c > a


class PyEqDefaultNe:
    def __init__(self, x: int) -> None:
        self.x = x

    def __eq__(self, other: Self) -> bool:
        return self.x == other.x


@pytest.mark.parametrize("ty", (EqDefaultNe, PyEqDefaultNe), ids=("rust", "python"))
def test_eq_default_ne(ty: Type[Union[EqDefaultNe, PyEqDefaultNe]]):
    a = ty(0)
    b = ty(0)
    c = ty(1)

    assert a == b
    assert not (a != b)
    assert a != c
    assert not (a == c)

    assert b == a
    assert not (a != b)
    assert b != c
    assert not (b == c)

    with pytest.raises(TypeError):
        assert a <= b

    with pytest.raises(TypeError):
        assert a >= b

    with pytest.raises(TypeError):
        assert a < c

    with pytest.raises(TypeError):
        assert c > a


class PyOrdered:
    def __init__(self, x: int) -> None:
        self.x = x

    def __lt__(self, other: Self) -> bool:
        return self.x < other.x

    def __le__(self, other: Self) -> bool:
        return self.x <= other.x

    def __eq__(self, other: Self) -> bool:
        return self.x == other.x

    def __ne__(self, other: Self) -> bool:
        return self.x != other.x

    def __gt__(self, other: Self) -> bool:
        return self.x >= other.x

    def __ge__(self, other: Self) -> bool:
        return self.x >= other.x


@pytest.mark.parametrize("ty", (Ordered, PyOrdered), ids=("rust", "python"))
def test_ordered(ty: Type[Union[Ordered, PyOrdered]]):
    a = ty(0)
    b = ty(0)
    c = ty(1)

    assert a == b
    assert a <= b
    assert a >= b
    assert a != c
    assert a <= c

    assert b == a
    assert b <= a
    assert b >= a
    assert b != c
    assert b <= c

    assert c != a
    assert c != b
    assert c > a
    assert c >= a
    assert c > b
    assert c >= b


class PyOrderedDefaultNe:
    def __init__(self, x: int) -> None:
        self.x = x

    def __lt__(self, other: Self) -> bool:
        return self.x < other.x

    def __le__(self, other: Self) -> bool:
        return self.x <= other.x

    def __eq__(self, other: Self) -> bool:
        return self.x == other.x

    def __gt__(self, other: Self) -> bool:
        return self.x >= other.x

    def __ge__(self, other: Self) -> bool:
        return self.x >= other.x


@pytest.mark.parametrize(
    "ty", (OrderedDefaultNe, PyOrderedDefaultNe), ids=("rust", "python")
)
def test_ordered_default_ne(ty: Type[Union[OrderedDefaultNe, PyOrderedDefaultNe]]):
    a = ty(0)
    b = ty(0)
    c = ty(1)

    assert a == b
    assert not (a != b)
    assert a <= b
    assert a >= b
    assert a != c
    assert not (a == c)
    assert a <= c

    assert b == a
    assert not (b != a)
    assert b <= a
    assert b >= a
    assert b != c
    assert not (b == c)
    assert b <= c

    assert c != a
    assert not (c == a)
    assert c != b
    assert not (c == b)
    assert c > a
    assert c >= a
    assert c > b
    assert c >= b
