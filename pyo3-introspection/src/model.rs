#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Module {
    pub name: String,
    pub modules: Vec<Module>,
    pub classes: Vec<Class>,
    pub functions: Vec<Function>,
    pub attributes: Vec<Attribute>,
    pub incomplete: bool,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Function>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Function {
    pub name: String,
    /// decorator like 'property' or 'staticmethod'
    pub decorators: Vec<String>,
    pub arguments: Arguments,
    /// return type
    pub returns: Option<TypeHint>,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Attribute {
    pub name: String,
    /// Value as a Python expression if easily expressible
    pub value: Option<String>,
    /// Type annotation as a Python expression
    pub annotation: Option<TypeHint>,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Arguments {
    /// Arguments before /
    pub positional_only_arguments: Vec<Argument>,
    /// Regular arguments (between / and *)
    pub arguments: Vec<Argument>,
    /// *vararg
    pub vararg: Option<VariableLengthArgument>,
    /// Arguments after *
    pub keyword_only_arguments: Vec<Argument>,
    /// **kwarg
    pub kwarg: Option<VariableLengthArgument>,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Argument {
    pub name: String,
    /// Default value as a Python expression
    pub default_value: Option<String>,
    /// Type annotation as a Python expression
    pub annotation: Option<TypeHint>,
}

/// A variable length argument ie. *vararg or **kwarg
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct VariableLengthArgument {
    pub name: String,
    /// Type annotation as a Python expression
    pub annotation: Option<TypeHint>,
}

/// A type hint annotation
///
/// Might be a plain string or an AST fragment
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum TypeHint {
    Ast(TypeHintExpr),
    Plain(String),
}

/// A type hint annotation as an AST fragment
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum TypeHintExpr {
    /// A Python builtin like `int`
    Builtin { id: String },
    /// The attribute of a python object like `{value}.{attr}`
    Attribute { module: String, attr: String },
    /// A union `{left} | {right}`
    Union { elts: Vec<TypeHintExpr> },
    /// A subscript `{value}[*slice]`
    Subscript {
        value: Box<TypeHintExpr>,
        slice: Vec<TypeHintExpr>,
    },
}
