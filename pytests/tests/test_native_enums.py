import enum
import sys

import pytest
from pyo3_pytests import native_enums


def test_color_is_enum_subclass():
    assert issubclass(native_enums.Color, enum.Enum)


def test_color_member_isinstance():
    assert isinstance(native_enums.Color.Red, enum.Enum)
    assert isinstance(native_enums.Color.Green, enum.Enum)
    assert isinstance(native_enums.Color.Blue, enum.Enum)


def test_color_member_isinstance_of_class():
    assert isinstance(native_enums.Color.Red, native_enums.Color)


def test_color_name_attribute():
    assert native_enums.Color.Red.name == "Red"
    assert native_enums.Color.Green.name == "Green"
    assert native_enums.Color.Blue.name == "Blue"


def test_color_len():
    assert len(native_enums.Color) == 3


def test_color_iter():
    members = list(native_enums.Color)
    assert members == [native_enums.Color.Red, native_enums.Color.Green, native_enums.Color.Blue]


def test_color_contains():
    assert native_enums.Color.Red in native_enums.Color
    assert native_enums.Color.Blue in native_enums.Color


def test_color_members_mapping():
    assert "Red" in native_enums.Color._member_names_
    assert "Green" in native_enums.Color._member_names_
    assert "Blue" in native_enums.Color._member_names_


def test_color_lookup_by_name():
    assert native_enums.Color["Red"] is native_enums.Color.Red
    assert native_enums.Color["Blue"] is native_enums.Color.Blue


def test_color_lookup_by_value():
    assert native_enums.Color(native_enums.Color.Red.value) is native_enums.Color.Red
    assert native_enums.Color(native_enums.Color.Blue.value) is native_enums.Color.Blue


def test_color_member_identity():
    a = native_enums.Color.Green
    b = native_enums.Color.Green
    assert a is b


def test_color_class_identity():
    cls1 = type(native_enums.Color.Red)
    cls2 = native_enums.Color
    assert cls1 is cls2


def test_identity_color_roundtrip():
    result = native_enums.identity_color(native_enums.Color.Red)
    assert result is native_enums.Color.Red


@pytest.mark.parametrize("variant", list(native_enums.Color))
def test_identity_color_all_variants(variant):
    assert native_enums.identity_color(variant) is variant


def test_status_is_int_enum_subclass():
    assert issubclass(native_enums.Status, enum.IntEnum)
    assert issubclass(native_enums.Status, int)


def test_status_values():
    assert native_enums.Status.Active == 1
    assert native_enums.Status.Inactive == 2
    assert native_enums.Status.Pending == 3


def test_status_isinstance_int():
    assert isinstance(native_enums.Status.Active, int)


def test_status_lookup_by_value():
    assert native_enums.Status(1) is native_enums.Status.Active
    assert native_enums.Status(2) is native_enums.Status.Inactive
    assert native_enums.Status(3) is native_enums.Status.Pending


def test_identity_status_roundtrip():
    result = native_enums.identity_status(native_enums.Status.Active)
    assert result is native_enums.Status.Active


def test_bits_is_int_flag_subclass():
    assert issubclass(native_enums.Bits, enum.IntFlag)
    assert issubclass(native_enums.Bits, int)


def test_bits_bitwise_or():
    ab = native_enums.Bits.A | native_enums.Bits.B
    assert native_enums.Bits.A in ab
    assert native_enums.Bits.B in ab
    assert native_enums.Bits.C not in ab


def test_bits_isinstance_int():
    assert isinstance(native_enums.Bits.A, int)
    assert native_enums.Bits.A == 1
    assert native_enums.Bits.B == 2
    assert native_enums.Bits.C == 4


def test_identity_bits_roundtrip():
    result = native_enums.identity_bits(native_enums.Bits.A)
    assert result is native_enums.Bits.A


def test_permission_is_flag_subclass():
    assert issubclass(native_enums.Permission, enum.Flag)


def test_permission_bitwise_or():
    rw = native_enums.Permission.Read | native_enums.Permission.Write
    assert native_enums.Permission.Read in rw
    assert native_enums.Permission.Write in rw
    assert native_enums.Permission.Exec not in rw


def test_permission_bitwise_and():
    rw = native_enums.Permission.Read | native_enums.Permission.Write
    assert rw & native_enums.Permission.Read == native_enums.Permission.Read


def test_identity_permission_roundtrip():
    result = native_enums.identity_permission(native_enums.Permission.Read)
    assert result is native_enums.Permission.Read


@pytest.mark.skipif(sys.version_info < (3, 11), reason="StrEnum requires Python 3.11+")
def test_size_is_str_enum_subclass():
    assert issubclass(native_enums.Size, enum.StrEnum)
    assert issubclass(native_enums.Size, str)


@pytest.mark.skipif(sys.version_info < (3, 11), reason="StrEnum requires Python 3.11+")
def test_size_members_are_strings():
    assert isinstance(native_enums.Size.Small, str)
    assert native_enums.Size.Small == "Small"
    assert native_enums.Size.Medium == "Medium"
    assert native_enums.Size.Large == "Large"


@pytest.mark.skipif(sys.version_info < (3, 11), reason="StrEnum requires Python 3.11+")
def test_size_lookup_by_value():
    assert native_enums.Size("Small") is native_enums.Size.Small
