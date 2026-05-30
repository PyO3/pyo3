/// The Python `enum` base class to use when building a native enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeEnumBase {
    /// `enum.Enum` — general-purpose enum.
    Enum,
    /// `enum.IntEnum` — enum whose members are also integers.
    IntEnum,
    /// `enum.StrEnum` — enum whose members are also strings (Python 3.11+).
    StrEnum,
    /// `enum.Flag` — enum supporting bitwise operations.
    Flag,
    /// `enum.IntFlag` — integer-valued enum supporting bitwise operations.
    IntFlag,
}

impl NativeEnumBase {
    pub(crate) fn class_name(self) -> &'static str {
        match self {
            Self::Enum => "Enum",
            Self::IntEnum => "IntEnum",
            Self::StrEnum => "StrEnum",
            Self::Flag => "Flag",
            Self::IntFlag => "IntFlag",
        }
    }
}

/// The value assigned to an enum variant when building the Python class.
#[derive(Debug, Clone, Copy)]
pub enum VariantValue {
    /// A fixed integer value.
    Int(i64),
    /// A fixed string value.
    Str(&'static str),
    /// Use `enum.auto()` to assign the value automatically.
    Auto,
}

/// Static specification used by [`build_native_enum`] to construct a Python `enum` subclass.
///
/// [`build_native_enum`]: super::build_native_enum
#[derive(Debug, Clone, Copy)]
pub struct NativeEnumSpec {
    /// The name passed as the first argument to the Python `enum` functional API.
    pub name: &'static str,
    /// The Python base class to subclass.
    pub base: NativeEnumBase,
    /// Ordered list of `(variant_name, value)` pairs.
    pub variants: &'static [(&'static str, VariantValue)],
    /// Optional `module` keyword argument forwarded to the functional API.
    pub module: Option<&'static str>,
    /// Optional `qualname` keyword argument forwarded to the functional API.
    pub qualname: Option<&'static str>,
}
