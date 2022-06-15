use crate::inspect::types::TypeInfo;

/// Python Interface information for a field (attribute, function, method…).
#[derive(Debug)]
pub struct FieldInfo<'a> {
    pub name: &'a str,
    pub kind: FieldKind,
    pub py_type: Option<fn() -> TypeInfo>,
    pub arguments: &'a [ArgumentInfo<'a>],
}

#[derive(Debug)]
pub enum FieldKind {
    /// The special 'new' method
    New,
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
    pub py_type: Option<fn() -> TypeInfo>,
    pub default_value: bool,
    pub is_modified: bool,
}

#[derive(Debug)]
pub enum ArgumentKind {
    /// A normal argument, that can be passed positionally or by keyword.
    PositionOrKeyword,
    /// A normal argument that can only be passed positionally (not by keyword).
    Position,
    /// A normal argument that can only be passed by keyword (not positionally).
    Keyword,
    /// An argument that represents all positional arguments that were provided on the call-site
    /// but do not match any declared regular argument.
    VarArg,
    /// An argument that represents all keyword arguments that were provided on the call-site
    /// but do not match any declared regular argument.
    KeywordArg,
}
