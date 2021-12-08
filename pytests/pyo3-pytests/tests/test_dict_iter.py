import pytest
from pyo3_pytests.dict_iter import DictSize


@pytest.mark.parametrize("size", [64, 128, 256])
def test_size(size):
    d = {}
    for i in range(size):
        d[i] = str(i)
    assert DictSize(len(d)).iter_dict(d) == size
