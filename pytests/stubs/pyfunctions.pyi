import typing

def args_kwargs(*args, **kwargs) -> typing.Any: ...
def many_keyword_arguments(
    *,
    ant: typing.Any | None = None,
    bear: typing.Any | None = None,
    cat: typing.Any | None = None,
    dog: typing.Any | None = None,
    elephant: typing.Any | None = None,
    fox: typing.Any | None = None,
    goat: typing.Any | None = None,
    horse: typing.Any | None = None,
    iguana: typing.Any | None = None,
    jaguar: typing.Any | None = None,
    koala: typing.Any | None = None,
    lion: typing.Any | None = None,
    monkey: typing.Any | None = None,
    newt: typing.Any | None = None,
    owl: typing.Any | None = None,
    penguin: typing.Any | None = None,
) -> typing.Any: ...
def none() -> None: ...
def positional_only(a: typing.Any, /, b: typing.Any) -> typing.Any: ...
def simple(
    a: typing.Any, b: typing.Any | None = None, *, c: typing.Any | None = None
) -> typing.Any: ...
def simple_args(
    a: typing.Any, b: typing.Any | None = None, *args, c: typing.Any | None = None
) -> typing.Any: ...
def simple_args_kwargs(
    a: typing.Any,
    b: typing.Any | None = None,
    *args,
    c: typing.Any | None = None,
    **kwargs,
) -> typing.Any: ...
def simple_kwargs(
    a: typing.Any, b: typing.Any | None = None, c: typing.Any | None = None, **kwargs
) -> typing.Any: ...
def with_custom_type_annotations(
    a: int, *_args: str, _b: int | None = None, **_kwargs: bool
) -> int: ...
def with_typed_args(
    a: bool = False, b: int = 0, c: float = 0.0, d: str = ""
) -> typing.Any: ...
