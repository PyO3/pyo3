from typing import Final, Self, SupportsIndex, final

USIZE_MAX: Final[int]
USIZE_MIN: Final[int]

@final
class ModClass:
    def __new__(cls, /) -> Self: ...
    def noop(self, /, x: SupportsIndex) -> int: ...

def double(x: SupportsIndex) -> int: ...
