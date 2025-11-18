from typing import Any

def args_kwargs(*args, **kwargs) -> Any: ...
def many_keyword_arguments(
    *,
    ant: object | None = None,
    bear: object | None = None,
    cat: object | None = None,
    dog: object | None = None,
    elephant: object | None = None,
    fox: object | None = None,
    goat: object | None = None,
    horse: object | None = None,
    iguana: object | None = None,
    jaguar: object | None = None,
    koala: object | None = None,
    lion: object | None = None,
    monkey: object | None = None,
    newt: object | None = None,
    owl: object | None = None,
    penguin: object | None = None,
) -> None: ...
def none() -> None: ...
def positional_only(a: object, /, b: object) -> Any: ...
def simple(a: object, b: object | None = None, *, c: object | None = None) -> Any: ...
def simple_args(
    a: object, b: object | None = None, *args, c: object | None = None
) -> Any: ...
def simple_args_kwargs(
    a: object, b: object | None = None, *args, c: object | None = None, **kwargs
) -> Any: ...
def simple_kwargs(
    a: object, b: object | None = None, c: object | None = None, **kwargs
) -> Any: ...
def with_custom_type_annotations(
    a: int, *_args: str, _b: int | None = None, **_kwargs: bool
) -> int: ...
def with_typed_args(
    a: bool = False, b: int = 0, c: float = 0.0, d: str = ""
) -> Any: ...
