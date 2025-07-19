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

    variant_with_default_1 = enums.ComplexEnum.VariantWithDefault()
    assert isinstance(variant_with_default_1, enums.ComplexEnum.VariantWithDefault)

    variant_with_default_2 = enums.ComplexEnum.VariantWithDefault(25, "Hello")
    assert isinstance(variant_with_default_2, enums.ComplexEnum.VariantWithDefault)


@pytest.mark.parametrize(
    "variant",
    [
        enums.ComplexEnum.Int(42),
        enums.ComplexEnum.Float(3.14),
        enums.ComplexEnum.Str("hello"),
        enums.ComplexEnum.EmptyStruct(),
        enums.ComplexEnum.MultiFieldStruct(42, 3.14, True),
        enums.ComplexEnum.VariantWithDefault(),
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

    variant_with_default = enums.ComplexEnum.VariantWithDefault()
    assert variant_with_default.a == 42
    assert variant_with_default.b is None


@pytest.mark.parametrize(
    "variant",
    [
        enums.ComplexEnum.Int(42),
        enums.ComplexEnum.Float(3.14),
        enums.ComplexEnum.Str("hello"),
        enums.ComplexEnum.EmptyStruct(),
        enums.ComplexEnum.MultiFieldStruct(42, 3.14, True),
        enums.ComplexEnum.VariantWithDefault(),
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
    elif isinstance(variant, enums.ComplexEnum.VariantWithDefault):
        x = variant.a
        y = variant.b
        assert x == 42
        assert y is None
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
        enums.ComplexEnum.VariantWithDefault(b="hello"),
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
    elif isinstance(variant, enums.ComplexEnum.VariantWithDefault):
        x = variant.a
        y = variant.b
        assert x == 84
        assert y == "HELLO"
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
        enums.TupleEnum.FullWithDefault(),
        enums.TupleEnum.Full(42, 3.14, False),
        enums.TupleEnum.EmptyTuple(),
    ],
)
def test_tuple_enum_variant_subclasses(variant: enums.TupleEnum):
    assert isinstance(variant, enums.TupleEnum)


def test_tuple_enum_defaults():
    variant = enums.TupleEnum.FullWithDefault()
    assert variant._0 == 1
    assert variant._1 == 1.0
    assert variant._2 is True


def test_tuple_enum_field_getters():
    tuple_variant = enums.TupleEnum.Full(42, 3.14, False)
    assert tuple_variant._0 == 42
    assert tuple_variant._1 == 3.14
    assert tuple_variant._2 is False


def test_tuple_enum_index_getter():
    tuple_variant = enums.TupleEnum.Full(42, 3.14, False)
    assert len(tuple_variant) == 3
    assert tuple_variant[0] == 42


@pytest.mark.parametrize(
    "variant",
    [enums.MixedComplexEnum.Nothing()],
)
def test_mixed_complex_enum_pyfunction_instance_nothing(
    variant: enums.MixedComplexEnum,
):
    assert isinstance(variant, enums.MixedComplexEnum.Nothing)
    assert isinstance(
        enums.do_mixed_complex_stuff(variant), enums.MixedComplexEnum.Empty
    )


@pytest.mark.parametrize(
    "variant",
    [enums.MixedComplexEnum.Empty()],
)
def test_mixed_complex_enum_pyfunction_instance_empty(variant: enums.MixedComplexEnum):
    assert isinstance(variant, enums.MixedComplexEnum.Empty)
    assert isinstance(
        enums.do_mixed_complex_stuff(variant), enums.MixedComplexEnum.Nothing
    )
