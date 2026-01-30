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
    pub bases: Vec<Expr>,
    pub methods: Vec<Function>,
    pub attributes: Vec<Attribute>,
    /// decorator like 'typing.final'
    pub decorators: Vec<Expr>,
    pub inner_classes: Vec<Class>,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Function {
    pub name: String,
    /// decorator like 'property' or 'staticmethod'
    pub decorators: Vec<Expr>,
    pub arguments: Arguments,
    /// return type
    pub returns: Option<Expr>,
    pub is_async: bool,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Attribute {
    pub name: String,
    /// Value as a Python expression if easily expressible
    pub value: Option<Expr>,
    /// Type annotation as a Python expression
    pub annotation: Option<Expr>,
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
    pub default_value: Option<Expr>,
    /// Type annotation as a Python expression
    pub annotation: Option<Expr>,
}

/// A variable length argument ie. *vararg or **kwarg
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct VariableLengthArgument {
    pub name: String,
    /// Type annotation as a Python expression
    pub annotation: Option<Expr>,
}

/// A python expression
///
/// This is the `expr` production of the [Python `ast` module grammar](https://docs.python.org/3/library/ast.html#abstract-grammar)
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Expr {
    /// A constant like `None` or `123`
    Constant { value: Constant },
    /// A name
    Name { id: String },
    /// An attribute `value.attr`
    Attribute { value: Box<Self>, attr: String },
    /// A binary operator
    BinOp {
        left: Box<Self>,
        op: Operator,
        right: Box<Self>,
    },
    /// A tuple
    Tuple { elts: Vec<Self> },
    /// A list
    List { elts: Vec<Self> },
    /// A subscript `value[slice]`
    Subscript { value: Box<Self>, slice: Box<Self> },
}

/// A PyO3 extension to the Python AST to know more about [`Expr::Constant`].
///
/// This enables advanced features like escaping.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Constant {
    /// `None`
    None,
    /// `True` or `False`
    Bool(bool),
    /// An integer in base 10
    Int(String),
    /// A float in base 10 (does not include Inf and NaN)
    Float(String),
    /// A string (unescaped!)
    Str(String),
    /// `...`
    Ellipsis,
}

/// An operator used in [`Expr::BinOp`].
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum Operator {
    /// `|` operator
    BitOr,
}
