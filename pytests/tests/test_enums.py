import pytest
from pyo3_pytests import enums


def test_complex_enum_variant_constructors():
    int_variant = enums.ComplexEnum.Int(42)
    assert isinstance(int_variant, enums.ComplexEnum.Int)

    float_variant = enums.ComplexEnum.Float(3.14)
    assert isinstance(float_variant, enums.ComplexEnum.Float)

    str_variant = enums.ComplexEnum.Str("hello")
    assert isinstance(str_variant, enums.ComplexEnum.Str)

    empty_struct_variant = enums.ComplexEnum.EmptyStruct()
    assert isinstance(empty_struct_variant, enums.ComplexEnum.EmptyStruct)

    multi_field_struct_variant = enums.ComplexEnum.MultiFieldStruct(42, 3.14, True)
    assert isinstance(multi_field_struct_variant, enums.ComplexEnum.MultiFieldStruct)


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
def test_complex_enum_variant_subclasses(variant: enums.ComplexEnum):
    assert isinstance(variant, enums.ComplexEnum)


def test_complex_enum_field_getters():
    int_variant = enums.ComplexEnum.Int(42)
    assert int_variant.i == 42

    float_variant = enums.ComplexEnum.Float(3.14)
    assert float_variant.f == 3.14

    str_variant = enums.ComplexEnum.Str("hello")
    assert str_variant.s == "hello"

    multi_field_struct_variant = enums.ComplexEnum.MultiFieldStruct(42, 3.14, True)
    assert multi_field_struct_variant.a == 42
    assert multi_field_struct_variant.b == 3.14
    assert multi_field_struct_variant.c is True


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
def test_complex_enum_desugared_match(variant: enums.ComplexEnum):
    if isinstance(variant, enums.ComplexEnum.Int):
        x = variant.i
        assert x == 42
    elif isinstance(variant, enums.ComplexEnum.Float):
        x = variant.f
        assert x == 3.14
    elif isinstance(variant, enums.ComplexEnum.Str):
        x = variant.s
        assert x == "hello"
    elif isinstance(variant, enums.ComplexEnum.EmptyStruct):
        assert True
    elif isinstance(variant, enums.ComplexEnum.MultiFieldStruct):
        x = variant.a
        y = variant.b
        z = variant.c
        assert x == 42
        assert y == 3.14
        assert z is True
    else:
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
def test_complex_enum_pyfunction_in_out_desugared_match(variant: enums.ComplexEnum):
    variant = enums.do_complex_stuff(variant)
    if isinstance(variant, enums.ComplexEnum.Int):
        x = variant.i
        assert x == 5
    elif isinstance(variant, enums.ComplexEnum.Float):
        x = variant.f
        assert x == 9.8596
    elif isinstance(variant, enums.ComplexEnum.Str):
        x = variant.s
        assert x == "42"
    elif isinstance(variant, enums.ComplexEnum.EmptyStruct):
        assert True
    elif isinstance(variant, enums.ComplexEnum.MultiFieldStruct):
        x = variant.a
        y = variant.b
        z = variant.c
        assert x == 42
        assert y == 3.14
        assert z is True
    else:
        assert False

def test_tuple_enum_variant_constructors():
    tuple_variant = enums.TupleEnum.Full(42, 3.14, False)
    assert isinstance(tuple_variant, enums.TupleEnum.Full)

    empty_tuple_variant = enums.TupleEnum.EmptyTuple()
    assert isinstance(empty_tuple_variant, enums.TupleEnum.EmptyTuple)

@pytest.mark.parametrize(
    "variant",
    [
        enums.TupleEnum.Full(42, 3.14, False),
        enums.TupleEnum.EmptyTuple(),
    ],
)
def test_tuple_enum_variant_subclasses(variant: enums.TupleEnum):
    assert isinstance(variant, enums.TupleEnum)

def test_tuple_enum_field_getters():
    tuple_variant = enums.TupleEnum.Full(42, 3.14, False)
    assert tuple_variant._0 == 42
    assert tuple_variant._1 == 3.14
    assert tuple_variant._2 is False