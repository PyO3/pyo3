from _typeshed import Incomplete
from typing import final

class AssertingBaseClass:
    """
    Demonstrates a base class which can operate on the relevant subclass in its constructor.
    """
    def __new__(cls, /, expected_type: type) -> AssertingBaseClass: ...

@final
class ClassWithDecorators:
    def __new__(cls, /) -> ClassWithDecorators: ...
    @property
    def attr(self, /) -> int:
        """
        A getter
        """
    @attr.deleter
    def attr(self, /) -> None:
        """
        A deleter
        """
    @attr.setter
    def attr(self, /, value: int) -> None:
        """
        A setter
        """
    @classmethod
    @property
    def cls_attribute(cls, /) -> int:
        """
        A class attribute
        """
    @classmethod
    def cls_method(cls, /) -> int:
        """
        A class method
        """
    @staticmethod
    def static_method() -> int:
        """
        A static method
        """

@final
class ClassWithDict:
    def __new__(cls, /) -> ClassWithDict: ...

@final
class ClassWithoutConstructor: ...

@final
class EmptyClass:
    def __len__(self, /) -> int: ...
    def __new__(cls, /) -> EmptyClass: ...
    def method(self, /) -> None: ...

@final
class PlainObject:
    @property
    def bar(self, /) -> int:
        """
        Bar
        """
    @bar.setter
    def bar(self, /, value: int) -> None:
        """
        Bar
        """
    @property
    def foo(self, /) -> str:
        """
        Foo
        """
    @foo.setter
    def foo(self, /, value: str) -> None:
        """
        Foo
        """

@final
class PyClassIter:
    """
    This is for demonstrating how to return a value from __next__
    """
    def __new__(cls, /) -> PyClassIter:
        """
        A constructor
        """
    def __next__(self, /) -> int: ...

@final
class PyClassThreadIter:
    def __new__(cls, /) -> PyClassThreadIter: ...
    def __next__(self, /) -> int: ...

@final
class SubClassWithInit(dict):
    def __init__(self, /, *args, **kwargs) -> None: ...
    def __new__(cls, /, *args, **kwargs) -> SubClassWithInit: ...

def map_a_class(
    cls: EmptyClass | tuple[EmptyClass, EmptyClass] | Incomplete,
) -> EmptyClass | tuple[EmptyClass, EmptyClass] | Incomplete: ...
