from .pyclasses import EmptyClass
from typing import Protocol, SupportsIndex, type_check_only

__all__ = ["cross_module_imports", "with_custom_type_annotations", "write_to"]

@type_check_only
class SupportsReadWrite(Protocol):
    """
    A binary stream that can be read from and written to
    """
    def read(self, n: int = ..., /) -> bytes: ...
    def write(self, data: bytes, /) -> SupportsIndex: ...

def cross_module_imports(_a: EmptyClass) -> None: ...
def with_custom_type_annotations(
    a: "list[int]", *_args: "str", _b: "int | None" = None, **_kwargs: "bool"
) -> "int": ...
def write_to(f: SupportsReadWrite, data: bytes) -> int: ...
