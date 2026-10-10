"""
This is documentation for the main module of PyO3 integration tests

It provides multiple modules to do tests
"""

from . import (
    annotations as annotations,
    awaitable as awaitable,
    buf_and_str as buf_and_str,
    comparisons as comparisons,
    consts as consts,
    datetime as datetime,
    dict_iter as dict_iter,
    enums as enums,
    exception as exception,
    misc as misc,
    objstore as objstore,
    othermod as othermod,
    path as path,
    pyclasses as pyclasses,
    pyfunctions as pyfunctions,
    sequence as sequence,
    subclassing as subclassing,
)
from _typeshed import Incomplete, SupportsRead, SupportsWrite
from typing import Final, Protocol, SupportsIndex, type_check_only

NO_STD: Final[bool]

@type_check_only
class SupportsClose(Protocol):
    """
    Only referenced as a base of `SupportsTypeshedReadWriteClose`
    """
    def close(self, /) -> None: ...

@type_check_only
class SupportsReadWrite(Protocol):
    """
    A binary stream that can be read from and written to
    """
    def read(self, n: int = ..., /) -> bytes: ...
    def write(self, data: bytes, /) -> SupportsIndex: ...

@type_check_only
class SupportsTypeshedReadWriteClose(
    SupportsRead[bytes], SupportsWrite[bytes], SupportsClose, Protocol
):
    """
    Built from the protocols of typeshed and `SupportsClose`
    """

def __getattr__(name: str) -> Incomplete: ...
