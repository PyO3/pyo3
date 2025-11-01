use crate::model::{
    Argument, Arguments, Attribute, Class, Function, Module, TypeHint, TypeHintExpr,
    VariableLengthArgument,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;
use std::str::FromStr;

/// Generates the [type stubs](https://typing.readthedocs.io/en/latest/source/stubs.html) of a given module.
/// It returns a map between the file name and the file content.
/// The root module stubs will be in the `__init__.pyi` file and the submodules directory
/// in files with a relevant name.
pub fn module_stub_files(module: &Module) -> HashMap<PathBuf, String> {
    let mut output_files = HashMap::new();
    add_module_stub_files(module, &[], &mut output_files);
    output_files
}

fn add_module_stub_files(
    module: &Module,
    module_path: &[&str],
    output_files: &mut HashMap<PathBuf, String>,
) {
    let mut file_path = PathBuf::new();
    for e in module_path {
        file_path = file_path.join(e);
    }
    output_files.insert(
        file_path.join("__init__.pyi"),
        module_stubs(module, module_path),
    );
    let mut module_path = module_path.to_vec();
    module_path.push(&module.name);
    for submodule in &module.modules {
        if submodule.modules.is_empty() {
            output_files.insert(
                file_path.join(format!("{}.pyi", submodule.name)),
                module_stubs(submodule, &module_path),
            );
        } else {
            add_module_stub_files(submodule, &module_path, output_files);
        }
    }
}

/// Generates the module stubs to a String, not including submodules
fn module_stubs(module: &Module, parents: &[&str]) -> String {
    let imports = Imports::create(module, parents);
    let mut elements = Vec::new();
    for attribute in &module.attributes {
        elements.push(attribute_stubs(attribute, &imports));
    }
    for class in &module.classes {
        elements.push(class_stubs(class, &imports));
    }
    for function in &module.functions {
        elements.push(function_stubs(function, &imports));
    }

    // We generate a __getattr__ method to tag incomplete stubs
    // See https://typing.python.org/en/latest/guides/writing_stubs.html#incomplete-stubs
    if module.incomplete && !module.functions.iter().any(|f| f.name == "__getattr__") {
        elements.push(function_stubs(
            &Function {
                name: "__getattr__".into(),
                decorators: Vec::new(),
                arguments: Arguments {
                    positional_only_arguments: Vec::new(),
                    arguments: vec![Argument {
                        name: "name".to_string(),
                        default_value: None,
                        annotation: Some(TypeHint::Ast(TypeHintExpr::Builtin { id: "str".into() })),
                    }],
                    vararg: None,
                    keyword_only_arguments: Vec::new(),
                    kwarg: None,
                },
                returns: Some(TypeHint::Ast(TypeHintExpr::Attribute {
                    module: "_typeshed".into(),
                    attr: "Incomplete".into(),
                })),
            },
            &imports,
        ));
    }

    let mut final_elements = imports.imports;
    final_elements.extend(elements);

    let mut output = String::new();

    // We insert two line jumps (i.e. empty strings) only above and below multiple line elements (classes with methods, functions with decorators)
    for element in final_elements {
        let is_multiline = element.contains('\n');
        if is_multiline && !output.is_empty() && !output.ends_with("\n\n") {
            output.push('\n');
        }
        output.push_str(&element);
        output.push('\n');
        if is_multiline {
            output.push('\n');
        }
    }

    // We remove a line jump at the end if they are two
    if output.ends_with("\n\n") {
        output.pop();
    }
    output
}

fn class_stubs(class: &Class, imports: &Imports) -> String {
    let mut buffer = format!("class {}:", class.name);
    if class.methods.is_empty() && class.attributes.is_empty() {
        buffer.push_str(" ...");
        return buffer;
    }
    for attribute in &class.attributes {
        // We do the indentation
        buffer.push_str("\n    ");
        buffer.push_str(&attribute_stubs(attribute, imports).replace('\n', "\n    "));
    }
    for method in &class.methods {
        // We do the indentation
        buffer.push_str("\n    ");
        buffer.push_str(&function_stubs(method, imports).replace('\n', "\n    "));
    }
    buffer
}

fn function_stubs(function: &Function, imports: &Imports) -> String {
    // Signature
    let mut parameters = Vec::new();
    for argument in &function.arguments.positional_only_arguments {
        parameters.push(argument_stub(argument, imports));
    }
    if !function.arguments.positional_only_arguments.is_empty() {
        parameters.push("/".into());
    }
    for argument in &function.arguments.arguments {
        parameters.push(argument_stub(argument, imports));
    }
    if let Some(argument) = &function.arguments.vararg {
        parameters.push(format!(
            "*{}",
            variable_length_argument_stub(argument, imports)
        ));
    } else if !function.arguments.keyword_only_arguments.is_empty() {
        parameters.push("*".into());
    }
    for argument in &function.arguments.keyword_only_arguments {
        parameters.push(argument_stub(argument, imports));
    }
    if let Some(argument) = &function.arguments.kwarg {
        parameters.push(format!(
            "**{}",
            variable_length_argument_stub(argument, imports)
        ));
    }
    let mut buffer = String::new();
    for decorator in &function.decorators {
        buffer.push('@');
        buffer.push_str(decorator);
        buffer.push('\n');
    }
    buffer.push_str("def ");
    buffer.push_str(&function.name);
    buffer.push('(');
    buffer.push_str(&parameters.join(", "));
    buffer.push(')');
    if let Some(returns) = &function.returns {
        buffer.push_str(" -> ");
        type_hint_stub(returns, imports, &mut buffer);
    }
    buffer.push_str(": ...");
    buffer
}

fn attribute_stubs(attribute: &Attribute, imports: &Imports) -> String {
    let mut buffer = attribute.name.clone();
    if let Some(annotation) = &attribute.annotation {
        buffer.push_str(": ");
        type_hint_stub(annotation, imports, &mut buffer);
    }
    if let Some(value) = &attribute.value {
        buffer.push_str(" = ");
        buffer.push_str(value);
    }
    buffer
}

fn argument_stub(argument: &Argument, imports: &Imports) -> String {
    let mut buffer = argument.name.clone();
    if let Some(annotation) = &argument.annotation {
        buffer.push_str(": ");
        type_hint_stub(annotation, imports, &mut buffer);
    }
    if let Some(default_value) = &argument.default_value {
        buffer.push_str(if argument.annotation.is_some() {
            " = "
        } else {
            "="
        });
        buffer.push_str(default_value);
    }
    buffer
}

fn variable_length_argument_stub(argument: &VariableLengthArgument, imports: &Imports) -> String {
    let mut buffer = argument.name.clone();
    if let Some(annotation) = &argument.annotation {
        buffer.push_str(": ");
        type_hint_stub(annotation, imports, &mut buffer);
    }
    buffer
}

fn type_hint_stub(type_hint: &TypeHint, imports: &Imports, buffer: &mut String) {
    match type_hint {
        TypeHint::Ast(t) => imports.serialize_type_hint(t, buffer),
        TypeHint::Plain(t) => buffer.push_str(t),
    }
}

/// Datastructure to deduplicate, validate and generate imports
#[derive(Default)]
struct Imports {
    /// Import lines ready to use
    imports: Vec<String>,
    /// Renaming map: from module name and member name return the name to use in type hints
    renaming: BTreeMap<(String, String), String>,
}

impl Imports {
    /// This generates a map from the builtin or module name to the actual alias used in the file
    ///
    /// For Python builtins and elements declared by the module the alias is always the actual name.
    ///
    /// For other elements, we can alias them using the `from X import Y as Z` syntax.
    /// So, we first list all builtins and local elements, then iterate on imports
    /// and create the aliases when needed.
    fn create(module: &Module, module_parents: &[&str]) -> Self {
        let mut elements_used_in_annotations = ElementsUsedInAnnotations::new();
        elements_used_in_annotations.walk_module(module);

        let mut imports = Vec::new();
        let mut renaming = BTreeMap::new();
        let mut local_name_to_module_and_attribute = BTreeMap::new();

        // We first process local and built-ins elements, they are never aliased or imported
        for name in module
            .classes
            .iter()
            .map(|c| c.name.clone())
            .chain(module.functions.iter().map(|f| f.name.clone()))
            .chain(module.attributes.iter().map(|a| a.name.clone()))
            .chain(elements_used_in_annotations.locals)
        {
            local_name_to_module_and_attribute.insert(name.clone(), (None, name.clone()));
        }

        // We compute the set of ways the current module can be named
        let mut possible_current_module_names = vec![module.name.clone()];
        let mut current_module_name = Some(module.name.clone());
        for parent in module_parents.iter().rev() {
            let path = if let Some(current) = current_module_name {
                format!("{parent}.{current}")
            } else {
                parent.to_string()
            };
            possible_current_module_names.push(path.clone());
            current_module_name = Some(path);
        }

        // We process then imports, normalizing local imports
        for (module, attrs) in elements_used_in_annotations.module_members {
            let normalized_module = if possible_current_module_names.contains(&module) {
                None
            } else {
                Some(module.clone())
            };
            let mut import_for_module = Vec::new();
            for attr in attrs {
                // We split nested classes A.B in "A" (the part that must be imported and can have naming conflicts) and ".B"
                let (root_attr, attr_path) = attr
                    .split_once('.')
                    .map_or((attr.as_str(), None), |(root, path)| (root, Some(path)));
                let mut local_name = root_attr.to_owned();
                let mut already_imported = false;
                while let Some((possible_conflict_module, possible_conflict_attr)) =
                    local_name_to_module_and_attribute.get(&local_name)
                {
                    if *possible_conflict_module == normalized_module
                        && *possible_conflict_attr == root_attr
                    {
                        // It's the same
                        already_imported = true;
                        break;
                    }
                    // We generate a new local name
                    // TODO: we use currently a format like Foo2. It might be nicer to use something like ModFoo
                    let number_of_digits_at_the_end = local_name
                        .bytes()
                        .rev()
                        .take_while(|b| b.is_ascii_digit())
                        .count();
                    let (local_name_prefix, local_name_number) =
                        local_name.split_at(local_name.len() - number_of_digits_at_the_end);
                    local_name = format!(
                        "{local_name_prefix}{}",
                        u64::from_str(local_name_number).unwrap_or(1) + 1
                    );
                }
                renaming.insert(
                    (module.clone(), attr.clone()),
                    if let Some(attr_path) = attr_path {
                        format!("{local_name}.{attr_path}")
                    } else {
                        local_name.clone()
                    },
                );
                if !already_imported {
                    local_name_to_module_and_attribute.insert(
                        local_name.clone(),
                        (normalized_module.clone(), root_attr.to_owned()),
                    );
                    let is_not_aliased_builtin =
                        normalized_module.as_deref() == Some("builtins") && local_name == root_attr;
                    if !is_not_aliased_builtin {
                        import_for_module.push(if local_name == root_attr {
                            local_name
                        } else {
                            format!("{root_attr} as {local_name}")
                        });
                    }
                }
            }
            if let Some(module) = normalized_module {
                if !import_for_module.is_empty() {
                    imports.push(format!(
                        "from {module} import {}",
                        import_for_module.join(", ")
                    ));
                }
            }
        }

        Self { imports, renaming }
    }

    fn serialize_type_hint(&self, expr: &TypeHintExpr, buffer: &mut String) {
        match expr {
            TypeHintExpr::Local { id } => buffer.push_str(id),
            TypeHintExpr::Builtin { id } => {
                let alias = self
                    .renaming
                    .get(&("builtins".to_string(), id.clone()))
                    .expect("All type hint attributes should have been visited");
                buffer.push_str(alias)
            }
            TypeHintExpr::Attribute { module, attr } => {
                let alias = self
                    .renaming
                    .get(&(module.clone(), attr.clone()))
                    .expect("All type hint attributes should have been visited");
                buffer.push_str(alias)
            }
            TypeHintExpr::Union { elts } => {
                for (i, elt) in elts.iter().enumerate() {
                    if i > 0 {
                        buffer.push_str(" | ");
                    }
                    self.serialize_type_hint(elt, buffer);
                }
            }
            TypeHintExpr::Subscript { value, slice } => {
                self.serialize_type_hint(value, buffer);
                buffer.push('[');
                for (i, elt) in slice.iter().enumerate() {
                    if i > 0 {
                        buffer.push_str(", ");
                    }
                    self.serialize_type_hint(elt, buffer);
                }
                buffer.push(']');
            }
        }
    }
}

/// Lists all the elements used in annotations
struct ElementsUsedInAnnotations {
    /// module -> name
    module_members: BTreeMap<String, BTreeSet<String>>,
    locals: BTreeSet<String>,
}

impl ElementsUsedInAnnotations {
    fn new() -> Self {
        Self {
            module_members: BTreeMap::new(),
            locals: BTreeSet::new(),
        }
    }

    fn walk_module(&mut self, module: &Module) {
        for attr in &module.attributes {
            self.walk_attribute(attr);
        }
        for class in &module.classes {
            self.walk_class(class);
        }
        for function in &module.functions {
            self.walk_function(function);
        }
        if module.incomplete {
            self.module_members
                .entry("builtins".into())
                .or_default()
                .insert("str".into());
            self.module_members
                .entry("_typeshed".into())
                .or_default()
                .insert("Incomplete".into());
        }
    }

    fn walk_class(&mut self, class: &Class) {
        for method in &class.methods {
            self.walk_function(method);
        }
        for attr in &class.attributes {
            self.walk_attribute(attr);
        }
    }

    fn walk_attribute(&mut self, attribute: &Attribute) {
        if let Some(type_hint) = &attribute.annotation {
            self.walk_type_hint(type_hint);
        }
    }

    fn walk_function(&mut self, function: &Function) {
        for decorator in &function.decorators {
            self.locals.insert(decorator.clone()); // TODO: better decorator support
        }
        for arg in function
            .arguments
            .positional_only_arguments
            .iter()
            .chain(&function.arguments.arguments)
            .chain(&function.arguments.keyword_only_arguments)
        {
            if let Some(type_hint) = &arg.annotation {
                self.walk_type_hint(type_hint);
            }
        }
        for arg in function
            .arguments
            .vararg
            .as_ref()
            .iter()
            .chain(&function.arguments.kwarg.as_ref())
        {
            if let Some(type_hint) = &arg.annotation {
                self.walk_type_hint(type_hint);
            }
        }
        if let Some(type_hint) = &function.returns {
            self.walk_type_hint(type_hint);
        }
    }

    fn walk_type_hint(&mut self, type_hint: &TypeHint) {
        if let TypeHint::Ast(type_hint) = type_hint {
            self.walk_type_hint_expr(type_hint);
        }
    }

    fn walk_type_hint_expr(&mut self, expr: &TypeHintExpr) {
        match expr {
            TypeHintExpr::Local { id } => {
                self.locals.insert(id.clone());
            }
            TypeHintExpr::Builtin { id } => {
                self.module_members
                    .entry("builtins".into())
                    .or_default()
                    .insert(id.clone());
            }
            TypeHintExpr::Attribute { module, attr } => {
                self.module_members
                    .entry(module.clone())
                    .or_default()
                    .insert(attr.clone());
            }
            TypeHintExpr::Union { elts } => {
                for elt in elts {
                    self.walk_type_hint_expr(elt)
                }
            }
            TypeHintExpr::Subscript { value, slice } => {
                self.walk_type_hint_expr(value);
                for elt in slice {
                    self.walk_type_hint_expr(elt);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Arguments;

    #[test]
    fn function_stubs_with_variable_length() {
        let function = Function {
            name: "func".into(),
            decorators: Vec::new(),
            arguments: Arguments {
                positional_only_arguments: vec![Argument {
                    name: "posonly".into(),
                    default_value: None,
                    annotation: None,
                }],
                arguments: vec![Argument {
                    name: "arg".into(),
                    default_value: None,
                    annotation: None,
                }],
                vararg: Some(VariableLengthArgument {
                    name: "varargs".into(),
                    annotation: None,
                }),
                keyword_only_arguments: vec![Argument {
                    name: "karg".into(),
                    default_value: None,
                    annotation: Some(TypeHint::Plain("str".into())),
                }],
                kwarg: Some(VariableLengthArgument {
                    name: "kwarg".into(),
                    annotation: Some(TypeHint::Plain("str".into())),
                }),
            },
            returns: Some(TypeHint::Plain("list[str]".into())),
        };
        assert_eq!(
            "def func(posonly, /, arg, *varargs, karg: str, **kwarg: str) -> list[str]: ...",
            function_stubs(&function, &Imports::default())
        )
    }

    #[test]
    fn function_stubs_without_variable_length() {
        let function = Function {
            name: "afunc".into(),
            decorators: Vec::new(),
            arguments: Arguments {
                positional_only_arguments: vec![Argument {
                    name: "posonly".into(),
                    default_value: Some("1".into()),
                    annotation: None,
                }],
                arguments: vec![Argument {
                    name: "arg".into(),
                    default_value: Some("True".into()),
                    annotation: None,
                }],
                vararg: None,
                keyword_only_arguments: vec![Argument {
                    name: "karg".into(),
                    default_value: Some("\"foo\"".into()),
                    annotation: Some(TypeHint::Plain("str".into())),
                }],
                kwarg: None,
            },
            returns: None,
        };
        assert_eq!(
            "def afunc(posonly=1, /, arg=True, *, karg: str = \"foo\"): ...",
            function_stubs(&function, &Imports::default())
        )
    }

    #[test]
    fn test_import() {
        let big_type = TypeHintExpr::Subscript {
            value: Box::new(TypeHintExpr::Builtin { id: "dict".into() }),
            slice: vec![
                TypeHintExpr::Attribute {
                    module: "foo.bar".into(),
                    attr: "A".into(),
                },
                TypeHintExpr::Union {
                    elts: vec![
                        TypeHintExpr::Attribute {
                            module: "bar".into(),
                            attr: "A".into(),
                        },
                        TypeHintExpr::Attribute {
                            module: "foo".into(),
                            attr: "A.C".into(),
                        },
                        TypeHintExpr::Attribute {
                            module: "foo".into(),
                            attr: "A.D".into(),
                        },
                        TypeHintExpr::Attribute {
                            module: "foo".into(),
                            attr: "B".into(),
                        },
                        TypeHintExpr::Attribute {
                            module: "bat".into(),
                            attr: "A".into(),
                        },
                        TypeHintExpr::Local { id: "int".into() },
                        TypeHintExpr::Builtin { id: "int".into() },
                        TypeHintExpr::Builtin { id: "float".into() },
                    ],
                },
            ],
        };
        let imports = Imports::create(
            &Module {
                name: "bar".into(),
                modules: Vec::new(),
                classes: vec![Class {
                    name: "A".into(),
                    methods: Vec::new(),
                    attributes: Vec::new(),
                }],
                functions: vec![Function {
                    name: String::new(),
                    decorators: Vec::new(),
                    arguments: Arguments {
                        positional_only_arguments: Vec::new(),
                        arguments: Vec::new(),
                        vararg: None,
                        keyword_only_arguments: Vec::new(),
                        kwarg: None,
                    },
                    returns: Some(TypeHint::Ast(big_type.clone())),
                }],
                attributes: Vec::new(),
                incomplete: true,
            },
            &["foo"],
        );
        assert_eq!(
            &imports.imports,
            &[
                "from _typeshed import Incomplete",
                "from bat import A as A2",
                "from builtins import int as int2",
                "from foo import A as A3, B",
            ]
        );
        let mut output = String::new();
        imports.serialize_type_hint(&big_type, &mut output);
        assert_eq!(
            output,
            "dict[A, A | A3.C | A3.D | B | A2 | int | int2 | float]"
        );
    }
}
