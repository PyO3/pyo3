from typing import final

__all__ = ["DictSize"]

@final
class DictSize:
    def __new__(cls, /, expected: int) -> DictSize: ...
    def iter_dict(self, /, dict: dict) -> int: ...
