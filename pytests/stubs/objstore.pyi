from typing import Any, final

__all__ = ["ObjStore"]

@final
class ObjStore:
    def __new__(cls, /) -> ObjStore: ...
    def push(self, /, obj: Any) -> None: ...
