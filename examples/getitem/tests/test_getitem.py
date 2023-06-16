import getitem
import pytest


def test_simple():
    container = getitem.ExampleContainer()
    assert container[3] == 3
    assert container[4] == 4
    assert container[-1] == -1
    assert container[5:3] == 2
    assert container[3:5] == 2
    # test setitem, but this just displays, no return to check
    container[3:5] = 2
    container[2] = 2
    # and note we will get an error on this one since we didn't
    # add strings
    with pytest.raises(TypeError):
        container["foo"] = 2
