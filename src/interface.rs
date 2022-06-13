//! Data required for Python Interface files (.pyi, also called 'stub files').
//!
//! This module creates three data structures, [`ModuleInfo`], [`ClassInfo`] and [`FieldInfo`],
//! which are responsible for generating parts of the interface files.

/// Python Interface information for a module.
#[derive(Debug)]
pub struct ModuleInfo {}

#[derive(Debug)]
pub struct ClassInfo<'a> {
    pub name: &'a str,
    pub base: &'a str,
    pub fields: &'a [FieldInfo<'a>],
}

/// Python Interface information for a field (attribute, function, method…).
#[derive(Debug)]
pub struct FieldInfo<'a> {
    pub name: &'a str,
    pub kind: FieldKind,
    pub py_type: Option<&'a TypeInfo<'a>>,
    pub arguments: &'a [ArgumentInfo<'a>],
}

#[derive(Debug)]
pub enum FieldKind {
    /// The special 'new' method
    New,
    /// A top-level or instance attribute
    Attribute,
    /// A top-level or instance getter
    Getter,
    /// A top-level or instance setter
    Setter,
    /// A top-level function or an instance method
    Function,
    /// A class method
    ClassMethod,
    /// A class attribute
    ClassAttribute,
    /// A static method
    StaticMethod,
}

#[derive(Debug)]
pub struct ArgumentInfo<'a> {
    pub name: &'a str,
    pub kind: ArgumentKind,
    pub py_type: Option<&'a TypeInfo<'a>>,
    pub default_value: bool,
    pub is_modified: bool,
}

#[derive(Debug)]
pub enum ArgumentKind {
    /// A normal argument, that can be passed positionally or by keyword.
    Regular,
    /// A normal argument that can only be passed positionally (not by keyword).
    PositionalOnly,
    /// An argument that represents all positional arguments that were provided on the call-site
    /// but do not match any declared regular argument.
    Vararg,
    /// An argument that represents all keyword arguments that were provided on the call-site
    /// but do not match any declared regular argument.
    KeywordArg,
}

/// The various Python types handled by PyO3.
///
/// The current implementation is not able to generate custom generics.
/// All generic types can only be hardcoded in PyO3.
#[derive(Debug)]
pub enum TypeInfo<'a> {
    /// The type `typing.Any`, which represents a dynamically-typed value (unknown type).
    Any,

    /// The type `typing.None`.
    None,

    /// The type `typing.Callable`, which represents a function-like object.
    Callable(Option<&'a [&'a TypeInfo<'a>]>, &'a TypeInfo<'a>),

    /// The type `typing.NoReturn`, which represents a function that never returns.
    NoReturn,

    /// A tuple of any value and size: `typing.Tuple[...]`.
    AnyTuple,

    /// A tuple of the specified types.
    Tuple(&'a [&'a TypeInfo<'a>]),

    /// A tuple of unknown size, in which all elements have the same type.
    UnsizedTuple(&'a TypeInfo<'a>),

    /// A union of multiple types.
    Union(&'a [&'a TypeInfo<'a>]),

    /// An optional value.
    Optional(&'a TypeInfo<'a>),

    Dict(&'a TypeInfo<'a>, &'a TypeInfo<'a>),

    Mapping(&'a TypeInfo<'a>, &'a TypeInfo<'a>),

    List(&'a TypeInfo<'a>),

    Set(&'a TypeInfo<'a>),

    FrozenSet(&'a TypeInfo<'a>),

    Sequence(&'a TypeInfo<'a>),

    Iterable(&'a TypeInfo<'a>),

    Iterator(&'a TypeInfo<'a>),

    Builtin(&'a str),

    /// Any type that doesn't receive special treatment from PyO3.
    Other {
        module: &'a str,
        name: &'a str,
    },
}

pub trait GetClassInfo {
    fn info() -> &'static ClassInfo<'static>;
}

pub trait GetClassFields {
    fn fields_info() -> &'static [&'static FieldInfo<'static>];
}
