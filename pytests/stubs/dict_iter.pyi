from typing import Self, SupportsIndex, final

__all__ = ["DictSize"]

@final
class DictSize:
    def __new__(cls, /, expected: SupportsIndex) -> Self: ...
    def iter_dict(self, /, dict: dict) -> int: ...
