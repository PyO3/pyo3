import pytest
from sequential import Id


def test_make_some():
    for x in range(12):
        i = Id()
        assert x == int(i)


def test_args():
    with pytest.raises(TypeError, match="Id\\(\\) takes no arguments"):
        Id(3, 4)


def test_cmp():
    a = Id()
    b = Id()
    assert a <= b
    assert a < b
    assert a == a
