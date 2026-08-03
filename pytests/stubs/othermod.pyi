from typing import Final, Self, final

USIZE_MAX: Final[int]
USIZE_MIN: Final[int]

@final
class ModClass:
    def __new__(cls, /) -> Self: ...
    def noop(self, /, x: int) -> int: ...

def double(x: int) -> int: ...
