# This file is only collected when Python >= 3.10, because it tests match syntax.
import pytest
from pyo3_pytests import enums


@pytest.mark.parametrize(
    "variant",
    [
        enums.ComplexEnum.Int(42),
        enums.ComplexEnum.Float(3.14),
        enums.ComplexEnum.Str("hello"),
        enums.ComplexEnum.EmptyStruct(),
        enums.ComplexEnum.MultiFieldStruct(42, 3.14, True),
    ],
)
def test_complex_enum_match_statement(variant: enums.ComplexEnum):
    match variant:
        case enums.ComplexEnum.Int(i=x):
            assert x == 42
        case enums.ComplexEnum.Float(f=x):
            assert x == 3.14
        case enums.ComplexEnum.Str(s=x):
            assert x == "hello"
        case enums.ComplexEnum.EmptyStruct():
            assert True
        case enums.ComplexEnum.MultiFieldStruct(a=x, b=y, c=z):
            assert x == 42
            assert y == 3.14
            assert z is True
        case _:
            assert False


@pytest.mark.parametrize(
    "variant",
    [
        enums.ComplexEnum.Int(42),
        enums.ComplexEnum.Float(3.14),
        enums.ComplexEnum.Str("hello"),
        enums.ComplexEnum.EmptyStruct(),
        enums.ComplexEnum.MultiFieldStruct(42, 3.14, True),
    ],
)
def test_complex_enum_pyfunction_in_out(variant: enums.ComplexEnum):
    match enums.do_complex_stuff(variant):
        case enums.ComplexEnum.Int(i=x):
            assert x == 5
        case enums.ComplexEnum.Float(f=x):
            assert x == 9.8596
        case enums.ComplexEnum.Str(s=x):
            assert x == "42"
        case enums.ComplexEnum.EmptyStruct():
            assert True
        case enums.ComplexEnum.MultiFieldStruct(a=x, b=y, c=z):
            assert x == 42
            assert y == 3.14
            assert z is True
        case _:
            assert False


@pytest.mark.parametrize(
    "variant",
    [
        enums.ComplexEnum.MultiFieldStruct(42, 3.14, True),
    ],
)
def test_complex_enum_partial_match(variant: enums.ComplexEnum):
    match variant:
        case enums.ComplexEnum.MultiFieldStruct(a):
            assert a == 42
        case _:
            assert False


@pytest.mark.parametrize(
    "variant",
    [
        enums.TupleEnum.Full(42, 3.14, True),
        enums.TupleEnum.EmptyTuple(),
    ],
)
def test_tuple_enum_match_statement(variant: enums.TupleEnum):
    match variant:
        case enums.TupleEnum.Full(_0=x, _1=y, _2=z):
            assert x == 42
            assert y == 3.14
            assert z is True
        case enums.TupleEnum.EmptyTuple():
            assert True
        case _:
            print(variant)
            assert False


@pytest.mark.parametrize(
    "variant",
    [
        enums.SimpleTupleEnum.Int(42),
        enums.SimpleTupleEnum.Str("hello"),
    ],
)
def test_simple_tuple_enum_match_statement(variant: enums.SimpleTupleEnum):
    match variant:
        case enums.SimpleTupleEnum.Int(x):
            assert x == 42
        case enums.SimpleTupleEnum.Str(x):
            assert x == "hello"
        case _:
            assert False


@pytest.mark.parametrize(
    "variant",
    [
        enums.TupleEnum.Full(42, 3.14, True),
    ],
)
def test_tuple_enum_match_match_args(variant: enums.TupleEnum):
    match variant:
        case enums.TupleEnum.Full(x, y, z):
            assert x == 42
            assert y == 3.14
            assert z is True
            assert True
        case _:
            assert False


@pytest.mark.parametrize(
    "variant",
    [
        enums.TupleEnum.Full(42, 3.14, True),
    ],
)
def test_tuple_enum_partial_match(variant: enums.TupleEnum):
    match variant:
        case enums.TupleEnum.Full(a):
            assert a == 42
        case _:
            assert False


@pytest.mark.parametrize(
    "variant",
    [
        enums.MixedComplexEnum.Nothing(),
        enums.MixedComplexEnum.Empty(),
    ],
)
def test_mixed_complex_enum_match_statement(variant: enums.MixedComplexEnum):
    match variant:
        case enums.MixedComplexEnum.Nothing():
            assert True
        case enums.MixedComplexEnum.Empty():
            assert True
        case _:
            assert False
