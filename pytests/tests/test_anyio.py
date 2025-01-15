import asyncio

from pyo3_pytests.anyio import sleep
import trio


def test_asyncio():
    assert asyncio.run(sleep(0)) is None
    assert asyncio.run(sleep(0.1, 42)) == 42


def test_trio():
    assert trio.run(sleep, 0) is None
    assert trio.run(sleep, 0.1, 42) == 42
