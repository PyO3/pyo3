from typing import SupportsIndex, final

__all__ = ["DictSize"]

@final
class DictSize:
    def __new__(cls, /, expected: SupportsIndex) -> DictSize: ...
    def iter_dict(self, /, dict: dict) -> int: ...
