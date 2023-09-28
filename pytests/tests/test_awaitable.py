import pytest

from pyo3_pytests.awaitable import IterAwaitable, FutureAwaitable


@pytest.mark.asyncio
async def test_iter_awaitable():
    assert await IterAwaitable(5) == 5


@pytest.mark.asyncio
async def test_future_awaitable():
    assert await FutureAwaitable(5) == 5
