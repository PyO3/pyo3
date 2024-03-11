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
}
