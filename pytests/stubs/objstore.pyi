from typing import Any, Self, final

__all__ = ["ObjStore"]

@final
class ObjStore:
    def __new__(cls, /) -> Self: ...
    def push(self, /, obj: Any) -> None: ...
