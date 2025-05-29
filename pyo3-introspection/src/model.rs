#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Module {
    pub name: String,
    pub modules: Vec<Module>,
    pub classes: Vec<Class>,
    pub functions: Vec<Function>,
    pub consts: Vec<Const>,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Function>,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Function {
    pub name: String,
    /// decorator like 'property' or 'staticmethod'
    pub decorators: Vec<String>,
    pub arguments: Arguments,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Const {
    pub name: String,
    pub value: String,
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
}

/// A variable length argument ie. *vararg or **kwarg
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct VariableLengthArgument {
    pub name: String,
}
