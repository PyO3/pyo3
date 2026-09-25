"""
Objects related to PyBuffer and PyStr
"""

from _typeshed import SupportsGetItem
from collections.abc import Buffer
from typing import SupportsIndex, final

__all__ = [
    "BytesExtractor",
    "map_byte_cow",
    "map_byte_slice",
    "map_byte_vec",
    "return_memoryview",
]

@final
class BytesExtractor:
    """
    This is for confirming that PyBuffer does not cause memory leak
    """
    def __new__(cls, /) -> BytesExtractor: ...
    @staticmethod
    def from_buffer(buf: Buffer) -> int: ...
    @staticmethod
    def from_bytes(bytes: bytes) -> int: ...
    @staticmethod
    def from_str(string: str) -> int: ...
    @staticmethod
    def from_str_lossy(string: str) -> int: ...

def map_byte_cow(bytes: SupportsGetItem[int, SupportsIndex]) -> bytes: ...
def map_byte_slice(bytes: bytes) -> bytes: ...
def map_byte_vec(bytes: SupportsGetItem[int, SupportsIndex]) -> bytes: ...
def return_memoryview() -> memoryview: ...
