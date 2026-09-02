from typing import Final, final

__all__ = ["USIZE_MAX", "USIZE_MIN", "ModClass", "double"]

USIZE_MAX: Final[int]
USIZE_MIN: Final[int]

@final
class ModClass:
    def __new__(cls, /) -> ModClass: ...
    def noop(self, /, x: int) -> int: ...

def double(x: int) -> int: ...
