#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Module {
    pub name: String,
    pub modules: Vec<Module>,
    pub classes: Vec<Class>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Class {
    pub name: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Function {
    pub name: String,
    pub arguments: Vec<Argument>,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Argument {
    pub name: String,
    pub kind: ParameterKind,
    /// Default value as a Python expression
    pub default_value: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum ParameterKind {
    /// Before /
    PositionalOnly,
    /// Between / and *
    PositionalOrKeyword,
    /// *args
    VarPositional,
    /// After *
    KeywordOnly,
    /// *kwargs
    VarKeyword,
}
