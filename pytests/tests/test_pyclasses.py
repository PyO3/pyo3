import asyncio
import platform
import sys
from collections.abc import Iterator

import pytest
from pyo3_pytests import pyclasses
from pyo3_pytests.awaitable import IterAwaitable
from typing_extensions import assert_type


def test_empty_class_init(benchmark):
    benchmark(pyclasses.EmptyClass)


def test_method_call(benchmark):
    obj = pyclasses.EmptyClass()
    assert benchmark(obj.method) is None


def test_proto_call(benchmark):
    obj = pyclasses.EmptyClass()
    assert benchmark(len, obj) == 0


class EmptyClassPy:
    def method(self):
        pass

    def __len__(self) -> int:
        return 0


def test_empty_class_init_py(benchmark):
    benchmark(EmptyClassPy)


def test_method_call_py(benchmark):
    obj = EmptyClassPy()
    assert benchmark(obj.method) == pyclasses.EmptyClass().method()


def test_proto_call_py(benchmark):
    obj = EmptyClassPy()
    assert benchmark(len, obj) == len(pyclasses.EmptyClass())


def test_iter():
    i = pyclasses.PyClassIter()
    assert next(i) == 1
    assert next(i) == 2
    assert next(i) == 3
    assert next(i) == 4
    assert next(i) == 5

    with pytest.raises(StopIteration) as excinfo:
        next(i)
    assert excinfo.value.value == "Ended"


@pytest.mark.parametrize(
    "cls", [pyclasses.PyClassOptionIter, pyclasses.PyClassResultOptionIter]
)
def test_option_iter(cls):
    assert list(cls()) == [1, 2, 3, 4, 5]

    i = cls()
    for _ in range(5):
        next(i)
    with pytest.raises(StopIteration):
        next(i)


@pytest.mark.skipif(
    sys.implementation.name == "graalpy" and sys.implementation.version < (25, 1),
    reason="`async for` on GraalPy < 25.1 lets a synchronously raised StopAsyncIteration escape",
)
def test_option_async_iter():
    async def collect():
        return [value async for value in pyclasses.PyClassOptionAsyncIter()]

    assert asyncio.run(collect()) == [1, 2, 3, 4, 5]


def test_option_iter_type_hints() -> None:
    # `None` stops the iteration rather than being yielded, so these classes are `Iterator[int]`
    # and not `Iterator[int | None]`
    plain: Iterator[int] = pyclasses.PyClassOptionIter()
    fallible: Iterator[int] = pyclasses.PyClassResultOptionIter()
    assert_type(next(plain), int)
    assert_type(next(fallible), int)

    # `__anext__` likewise hands back the awaitable itself, not `IterAwaitable | None`
    assert_type(pyclasses.PyClassOptionAsyncIter().__anext__(), IterAwaitable)


@pytest.mark.skipif(
    platform.machine() in ["wasm32", "wasm64"],
    reason="not supporting threads in CI for WASM yet",
)
def test_parallel_iter():
    import concurrent.futures
    import threading

    thread_iter = pyclasses.PyClassThreadIter()
    max_workers = 2
    b = threading.Barrier(max_workers)
    error_happened = threading.Event()

    # the second thread attempts to borrow a reference to the instance's
    # state while the first thread is still sleeping, so we trigger a
    # runtime borrow-check error
    def closure(i):
        b.wait()
        # should never reach 100 iterations, the borrow error should
        # happen relatively quickly because the loops are synchronized
        for j in range(100):
            if not error_happened.is_set():
                try:
                    next(thread_iter)
                except RuntimeError as e:
                    assert "Already borrowed" in str(e), str(e)
                    error_happened.set()
            else:
                break
        else:
            assert False, "Should not be able to complete loop"

    with concurrent.futures.ThreadPoolExecutor(max_workers=max_workers) as tpe:
        for _ in tpe.map(closure, range(max_workers)):
            pass


class AssertingSubClass(pyclasses.AssertingBaseClass):
    pass


def test_new_classmethod():
    # The `AssertingBaseClass` constructor errors if it is not passed the
    # relevant subclass.
    _ = AssertingSubClass(expected_type=AssertingSubClass)
    with pytest.raises(ValueError):
        _ = AssertingSubClass(expected_type=str)


class ClassWithoutConstructor:
    def __new__(cls):
        raise TypeError(
            f"cannot create '{cls.__module__}.{cls.__qualname__}' instances"
        )


@pytest.mark.xfail(
    platform.python_implementation() == "PyPy" and sys.version_info[:2] == (3, 11),
    reason="broken on PyPy 3.11 due to https://github.com/pypy/pypy/issues/5319, waiting for next release",
)
@pytest.mark.parametrize(
    "cls, exc_message",
    [
        (
            pyclasses.ClassWithoutConstructor,
            "cannot create 'builtins.ClassWithoutConstructor' instances",
        ),
        (
            ClassWithoutConstructor,
            "cannot create 'test_pyclasses.ClassWithoutConstructor' instances",
        ),
    ],
)
def test_no_constructor_defined_propagates_cause(cls: type, exc_message: str):
    original_error = ValueError("Original message")
    with pytest.raises(TypeError) as exc_info:
        try:
            raise original_error
        except ValueError:
            cls()  # should raise TypeError("No constructor defined for ...")

    assert exc_info.value.args == (exc_message,)
    assert exc_info.value.__context__ is original_error


def test_dict():
    try:
        ClassWithDict = pyclasses.ClassWithDict
    except AttributeError:
        pytest.skip("not defined using abi3 < 3.9")

    d = ClassWithDict()
    assert d.__dict__ == {}

    d.foo = 42  # type: ignore[missing-attribute]
    assert d.__dict__ == {"foo": 42}


def test_getter(benchmark):
    obj = pyclasses.ClassWithDecorators()
    benchmark(lambda: obj.attr)


def test_setter(benchmark):
    obj = pyclasses.ClassWithDecorators()

    def set_attr():
        obj.attr = 42

    benchmark(set_attr)


def test_deleter():
    obj = pyclasses.ClassWithDecorators()
    del obj.attr
    with pytest.raises(AttributeError):
        _ = obj.attr
    obj.attr = 42
    assert obj.attr == 42


def test_class_attribute(benchmark):
    cls = pyclasses.ClassWithDecorators
    benchmark(lambda: cls.cls_attribute)


def test_class_method(benchmark):
    cls = pyclasses.ClassWithDecorators
    benchmark(lambda: cls.cls_method())


def test_static_method(benchmark):
    cls = pyclasses.ClassWithDecorators
    benchmark(lambda: cls.static_method())


def test_class_init_method():
    try:
        SubClassWithInit = pyclasses.SubClassWithInit
    except AttributeError:
        pytest.skip("not defined using abi3")

    d = SubClassWithInit()
    assert d == {"__init__": True}

    d = SubClassWithInit({"a": 1}, b=2)
    assert d == {"__init__": True, "a": 1, "b": 2}
