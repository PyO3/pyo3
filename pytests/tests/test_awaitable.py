import pytest
import sys

from pyo3_pytests.awaitable import IterAwaitable, FutureAwaitable, AsyncRange


@pytest.mark.skipif(
    sys.implementation.name == "graalpy",
    reason="GraalPy's asyncio module has a bug with native classes, see oracle/graalpython#365",
)
@pytest.mark.asyncio
async def test_iter_awaitable():
    assert await IterAwaitable(5) == 5


@pytest.mark.skipif(
    sys.implementation.name == "graalpy",
    reason="GraalPy's asyncio module has a bug with native classes, see oracle/graalpython#365",
)
@pytest.mark.asyncio
async def test_future_awaitable():
    assert await FutureAwaitable(5) == 5

class PyAsyncRange:
    def __init__(self, n):
        self.n = n

    def __aiter__(self):
        self.i = 0
        return self

    async def __anext__(self):
        if self.i < self.n:
            i = self.i
            self.i += 1
            return i
        raise StopAsyncIteration


@pytest.mark.parametrize(
    "ty",
    (
            AsyncRange,
            PyAsyncRange,
    ),
    ids=(
            "rust",
            "python",
    ),
)
@pytest.mark.asyncio
async def test_async_iterator(ty: int):
    x = 0
    async for i in ty(5):
        assert i == x
        x += 1
    assert x == 5
