import pytest
import sys

from pyo3_pytests.awaitable import IterAwaitable, FutureAwaitable


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
