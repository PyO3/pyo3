"""
Some module
"""

from _typeshed import Incomplete
from collections.abc import Sequence
from os import PathLike
from typing import Final, final

CONST: Final = 0
"""
Some const
"""

@final
class MyClass(dict):
    """
    Some class
    """

    class_attr: Final[float]
    def __eq__(self, /, other: object) -> bool: ...
    def __ge__(self, /, other: object) -> bool: ...
    def __gt__(self, /, other: object) -> bool: ...
    def __le__(self, /, other: object) -> bool: ...
    def __lt__(self, /, other: object) -> bool: ...
    def __ne__(self, /, other: object) -> bool: ...
    def __new__(cls, /, value: int) -> MyClass: ...
    @classmethod
    def class_method(cls, /) -> str: ...
    @staticmethod
    def static_method() -> bool: ...
    @property
    def value(self, /) -> int: ...
    @value.deleter
    def value(self, /) -> None: ...
    @value.setter
    def value(self, /, value: int) -> None: ...

def some_fn(
    _arg1: tuple[int, Sequence[str | PathLike[str]], dict[str, int]],
    /,
    _arg2: "int",
    *_args,
    _foo: str | None = None,
    **_kwargs,
) -> None:
    """
    Some function
    """

def __getattr__(name: str) -> Incomplete: ...
