from typing import Final, SupportsIndex, final

USIZE_MAX: Final[int]
USIZE_MIN: Final[int]

@final
class ModClass:
    def __new__(cls, /) -> ModClass: ...
    def noop(self, /, x: SupportsIndex) -> int: ...

def double(x: SupportsIndex) -> int: ...
