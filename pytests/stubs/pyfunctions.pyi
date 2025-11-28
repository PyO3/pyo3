from typing import Any

def args_kwargs(*args, **kwargs) -> tuple[tuple, dict | None]: ...
def many_keyword_arguments(
    *,
    ant: Any | None = None,
    bear: Any | None = None,
    cat: Any | None = None,
    dog: Any | None = None,
    elephant: Any | None = None,
    fox: Any | None = None,
    goat: Any | None = None,
    horse: Any | None = None,
    iguana: Any | None = None,
    jaguar: Any | None = None,
    koala: Any | None = None,
    lion: Any | None = None,
    monkey: Any | None = None,
    newt: Any | None = None,
    owl: Any | None = None,
    penguin: Any | None = None,
) -> None: ...
def none() -> None: ...
def positional_only(a: Any, /, b: Any) -> tuple[Any, Any]: ...
def simple(
    a: Any, b: Any | None = None, *, c: Any | None = None
) -> tuple[Any, Any | None, Any | None]: ...
def simple_args(
    a: Any, b: Any | None = None, *args, c: Any | None = None
) -> tuple[Any, Any | None, tuple, Any | None]: ...
def simple_args_kwargs(
    a: Any, b: Any | None = None, *args, c: Any | None = None, **kwargs
) -> tuple[Any, Any | None, tuple, Any | None, dict | None]: ...
def simple_kwargs(
    a: Any, b: Any | None = None, c: Any | None = None, **kwargs
) -> tuple[Any, Any | None, Any | None, dict | None]: ...
def with_custom_type_annotations(
    a: int, *_args: str, _b: int | None = None, **_kwargs: bool
) -> int: ...
def with_typed_args(
    a: bool = False, b: int = 0, c: float = 0.0, d: str = ""
) -> tuple[bool, int, float, str]: ...
