use crate::model::{
    Argument, Arguments, Attribute, Class, Constant, Expr, Function, Module, Operator,
    VariableLengthArgument,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write;
use std::iter::once;
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
        elements.push(function_stubs(function, &imports, None));
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
                        annotation: Some(Expr::Name { id: "str".into() }),
                    }],
                    vararg: None,
                    keyword_only_arguments: Vec::new(),
                    kwarg: None,
                },
                returns: Some(Expr::Attribute {
                    value: Box::new(Expr::Name {
                        id: "_typeshed".into(),
                    }),
                    attr: "Incomplete".into(),
                }),
                is_async: false,
                docstring: None,
            },
            &imports,
            None,
        ));
    }

    let mut final_elements = Vec::new();
    if let Some(docstring) = &module.docstring {
        final_elements.push(format!("\"\"\"\n{docstring}\n\"\"\""));
    }
    final_elements.extend(imports.imports);
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
    let mut buffer = String::new();
    for decorator in &class.decorators {
        buffer.push('@');
        imports.serialize_expr(decorator, &mut buffer);
        buffer.push('\n');
    }
    buffer.push_str("class ");
    buffer.push_str(&class.name);
    if !class.bases.is_empty() {
        buffer.push('(');
        for (i, base) in class.bases.iter().enumerate() {
            if i > 0 {
                buffer.push_str(", ");
            }
            imports.serialize_expr(base, &mut buffer);
        }
        buffer.push(')');
    }
    buffer.push(':');
    if class.docstring.is_none()
        && class.methods.is_empty()
        && class.attributes.is_empty()
        && class.inner_classes.is_empty()
    {
        buffer.push_str(" ...");
    }
    if let Some(docstring) = &class.docstring {
        buffer.push_str("\n    \"\"\"");
        for line in docstring.lines() {
            buffer.push_str("\n    ");
            buffer.push_str(line);
        }
        buffer.push_str("\n    \"\"\"");
    }
    for attribute in &class.attributes {
        // We do the indentation
        buffer.push_str("\n    ");
        buffer.push_str(&attribute_stubs(attribute, imports).replace('\n', "\n    "));
    }
    for method in &class.methods {
        // We do the indentation
        buffer.push_str("\n    ");
        buffer
            .push_str(&function_stubs(method, imports, Some(&class.name)).replace('\n', "\n    "));
    }
    for inner_class in &class.inner_classes {
        // We do the indentation
        buffer.push_str("\n    ");
        buffer.push_str(&class_stubs(inner_class, imports).replace('\n', "\n    "));
    }
    buffer
}

fn function_stubs(function: &Function, imports: &Imports, class_name: Option<&str>) -> String {
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
        // We remove the class name if it's a prefix to get nicer decorators
        let mut decorator_buffer = String::new();
        imports.serialize_expr(decorator, &mut decorator_buffer);
        if let Some(class_name) = class_name {
            if let Some(decorator) = decorator_buffer.strip_prefix(&format!("{class_name}.")) {
                decorator_buffer = decorator.into();
            }
        }
        buffer.push_str(&decorator_buffer);
        buffer.push('\n');
    }
    if function.is_async {
        buffer.push_str("async ");
    }

    buffer.push_str("def ");
    buffer.push_str(&function.name);
    buffer.push('(');
    buffer.push_str(&parameters.join(", "));
    buffer.push(')');
    if let Some(returns) = &function.returns {
        buffer.push_str(" -> ");
        imports.serialize_expr(returns, &mut buffer);
    }
    if let Some(docstring) = &function.docstring {
        buffer.push_str(":\n    \"\"\"");
        for line in docstring.lines() {
            buffer.push_str("\n    ");
            buffer.push_str(line);
        }
        buffer.push_str("\n    \"\"\"");
    } else {
        buffer.push_str(": ...");
    }
    buffer
}

fn attribute_stubs(attribute: &Attribute, imports: &Imports) -> String {
    let mut buffer = attribute.name.clone();
    if let Some(annotation) = &attribute.annotation {
        buffer.push_str(": ");
        imports.serialize_expr(annotation, &mut buffer);
    }
    if let Some(value) = &attribute.value {
        buffer.push_str(" = ");
        imports.serialize_expr(value, &mut buffer);
    }
    if let Some(docstring) = &attribute.docstring {
        buffer.push_str("\n\"\"\"");
        for line in docstring.lines() {
            buffer.push('\n');
            buffer.push_str(line);
        }
        buffer.push_str("\n\"\"\"");
    }
    buffer
}

fn argument_stub(argument: &Argument, imports: &Imports) -> String {
    let mut buffer = argument.name.clone();
    if let Some(annotation) = &argument.annotation {
        buffer.push_str(": ");
        imports.serialize_expr(annotation, &mut buffer);
    }
    if let Some(default_value) = &argument.default_value {
        buffer.push_str(if argument.annotation.is_some() {
            " = "
        } else {
            "="
        });
        imports.serialize_expr(default_value, &mut buffer);
    }
    buffer
}

fn variable_length_argument_stub(argument: &VariableLengthArgument, imports: &Imports) -> String {
    let mut buffer = argument.name.clone();
    if let Some(annotation) = &argument.annotation {
        buffer.push_str(": ");
        imports.serialize_expr(annotation, &mut buffer);
    }
    buffer
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

        // We get the current module full name
        let current_module_name = module_parents
            .iter()
            .copied()
            .chain(once(module.name.as_str()))
            .collect::<Vec<_>>()
            .join(".");

        // We first list local elements, they are never aliased or imported
        for name in module
            .classes
            .iter()
            .map(|c| c.name.clone())
            .chain(module.functions.iter().map(|f| f.name.clone()))
            .chain(module.attributes.iter().map(|a| a.name.clone()))
        {
            local_name_to_module_and_attribute
                .insert(name.clone(), (current_module_name.clone(), name.clone()));
        }
        // We don't process the current module elements, no need to care about them
        local_name_to_module_and_attribute.remove(&current_module_name);

        // We process then imports, normalizing local imports
        for (module, attrs) in &elements_used_in_annotations.module_to_name {
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
                    if possible_conflict_module == module && *possible_conflict_attr == root_attr {
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
                    local_name_to_module_and_attribute
                        .insert(local_name.clone(), (module.clone(), root_attr.to_owned()));
                    let is_not_aliased_builtin = module == "builtins" && local_name == root_attr;
                    if !is_not_aliased_builtin {
                        import_for_module.push(if local_name == root_attr {
                            local_name
                        } else {
                            format!("{root_attr} as {local_name}")
                        });
                    }
                }
            }
            if !import_for_module.is_empty() {
                imports.push(format!(
                    "from {module} import {}",
                    import_for_module.join(", ")
                ));
            }
        }
        imports.sort(); // We make sure they are sorted

        Self { imports, renaming }
    }

    fn serialize_expr(&self, expr: &Expr, buffer: &mut String) {
        match expr {
            Expr::Constant { value } => match value {
                Constant::None => buffer.push_str("None"),
                Constant::Bool(value) => buffer.push_str(if *value { "True" } else { "False" }),
                Constant::Int(value) => buffer.push_str(value),
                Constant::Float(value) => {
                    buffer.push_str(value);
                    if !value.contains(['.', 'e', 'E']) {
                        buffer.push('.'); // We make sure it's not parsed as an int
                    }
                }
                Constant::Str(value) => {
                    buffer.push('"');
                    for c in value.chars() {
                        match c {
                            '"' => buffer.push_str("\\\""),
                            '\n' => buffer.push_str("\\n"),
                            '\r' => buffer.push_str("\\r"),
                            '\t' => buffer.push_str("\\t"),
                            '\\' => buffer.push_str("\\\\"),
                            '\0' => buffer.push_str("\\0"),
                            c @ '\x00'..'\x20' => {
                                write!(buffer, "\\x{:02x}", u32::from(c)).unwrap()
                            }
                            c => buffer.push(c),
                        }
                    }
                    buffer.push('"');
                }
                Constant::Ellipsis => buffer.push_str("..."),
            },
            Expr::Name { id } => {
                buffer.push_str(
                    self.renaming
                        .get(&("builtins".into(), id.clone()))
                        .expect("All type hint attributes should have been visited"),
                );
            }
            Expr::Attribute { value, attr } => {
                if let Expr::Name { id, .. } = &**value {
                    buffer.push_str(
                        self.renaming
                            .get(&(id.clone(), attr.clone()))
                            .expect("All type hint attributes should have been visited"),
                    );
                } else {
                    self.serialize_expr(value, buffer);
                    buffer.push('.');
                    buffer.push_str(attr);
                }
            }
            Expr::BinOp { left, op, right } => {
                self.serialize_expr(left, buffer);
                buffer.push(' ');
                buffer.push(match op {
                    Operator::BitOr => '|',
                });
                self.serialize_expr(right, buffer);
            }
            Expr::Tuple { elts } => {
                buffer.push('(');
                self.serialize_elts(elts, buffer);
                if elts.len() == 1 {
                    buffer.push(',');
                }
                buffer.push(')')
            }
            Expr::List { elts } => {
                buffer.push('[');
                self.serialize_elts(elts, buffer);
                buffer.push(']')
            }
            Expr::Subscript { value, slice } => {
                self.serialize_expr(value, buffer);
                buffer.push('[');
                if let Expr::Tuple { elts } = &**slice {
                    // We don't display the tuple parentheses
                    self.serialize_elts(elts, buffer);
                } else {
                    self.serialize_expr(slice, buffer);
                }
                buffer.push(']');
            }
        }
    }

    fn serialize_elts(&self, elts: &[Expr], buffer: &mut String) {
        for (i, elt) in elts.iter().enumerate() {
            if i > 0 {
                buffer.push_str(", ");
            }
            self.serialize_expr(elt, buffer);
        }
    }
}

/// Lists all the elements used in annotations
struct ElementsUsedInAnnotations {
    /// module -> name where module is global (from the root of the interpreter).
    module_to_name: BTreeMap<String, BTreeSet<String>>,
}

impl ElementsUsedInAnnotations {
    fn new() -> Self {
        Self {
            module_to_name: BTreeMap::new(),
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
            self.module_to_name
                .entry("builtins".into())
                .or_default()
                .insert("str".into());
            self.module_to_name
                .entry("_typeshed".into())
                .or_default()
                .insert("Incomplete".into());
        }
    }

    fn walk_class(&mut self, class: &Class) {
        for base in &class.bases {
            self.walk_expr(base);
        }
        for decorator in &class.decorators {
            self.walk_expr(decorator);
        }
        for method in &class.methods {
            self.walk_function(method);
        }
        for attr in &class.attributes {
            self.walk_attribute(attr);
        }
        for class in &class.inner_classes {
            self.walk_class(class);
        }
    }

    fn walk_attribute(&mut self, attribute: &Attribute) {
        if let Some(type_hint) = &attribute.annotation {
            self.walk_expr(type_hint);
        }
    }

    fn walk_function(&mut self, function: &Function) {
        for decorator in &function.decorators {
            self.walk_expr(decorator);
        }
        for arg in function
            .arguments
            .positional_only_arguments
            .iter()
            .chain(&function.arguments.arguments)
            .chain(&function.arguments.keyword_only_arguments)
        {
            if let Some(type_hint) = &arg.annotation {
                self.walk_expr(type_hint);
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
                self.walk_expr(type_hint);
            }
        }
        if let Some(type_hint) = &function.returns {
            self.walk_expr(type_hint);
        }
    }

    fn walk_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Name { id } => {
                self.module_to_name
                    .entry("builtins".into())
                    .or_default()
                    .insert(id.clone());
            }
            Expr::Attribute { value, attr } => {
                if let Expr::Name { id } = &**value {
                    self.module_to_name
                        .entry(id.into())
                        .or_default()
                        .insert(attr.clone());
                } else {
                    self.walk_expr(value)
                }
            }
            Expr::BinOp { left, right, .. } => {
                self.walk_expr(left);
                self.walk_expr(right);
            }
            Expr::Subscript { value, slice } => {
                self.walk_expr(value);
                self.walk_expr(slice);
            }
            Expr::Tuple { elts } | Expr::List { elts } => {
                for elt in elts {
                    self.walk_expr(elt)
                }
            }
            Expr::Constant { .. } => (),
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
                    annotation: Some(Expr::Constant {
                        value: Constant::Str("str".into()),
                    }),
                }],
                kwarg: Some(VariableLengthArgument {
                    name: "kwarg".into(),
                    annotation: Some(Expr::Constant {
                        value: Constant::Str("str".into()),
                    }),
                }),
            },
            returns: Some(Expr::Constant {
                value: Constant::Str("list[str]".into()),
            }),
            is_async: false,
            docstring: None,
        };
        assert_eq!(
            "def func(posonly, /, arg, *varargs, karg: \"str\", **kwarg: \"str\") -> \"list[str]\": ...",
            function_stubs(&function, &Imports::default(), None)
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
                    default_value: Some(Expr::Constant {
                        value: Constant::Int("1".into()),
                    }),
                    annotation: None,
                }],
                arguments: vec![Argument {
                    name: "arg".into(),
                    default_value: Some(Expr::Constant {
                        value: Constant::Bool(true),
                    }),
                    annotation: None,
                }],
                vararg: None,
                keyword_only_arguments: vec![Argument {
                    name: "karg".into(),
                    default_value: Some(Expr::Constant {
                        value: Constant::Str("foo".into()),
                    }),
                    annotation: Some(Expr::Constant {
                        value: Constant::Str("str".into()),
                    }),
                }],
                kwarg: None,
            },
            returns: None,
            is_async: false,
            docstring: None,
        };
        assert_eq!(
            "def afunc(posonly=1, /, arg=True, *, karg: \"str\" = \"foo\"): ...",
            function_stubs(&function, &Imports::default(), None)
        )
    }

    #[test]
    fn test_function_async() {
        let function = Function {
            name: "foo".into(),
            decorators: Vec::new(),
            arguments: Arguments {
                positional_only_arguments: Vec::new(),
                arguments: Vec::new(),
                vararg: None,
                keyword_only_arguments: Vec::new(),
                kwarg: None,
            },
            returns: None,
            is_async: true,
            docstring: None,
        };
        assert_eq!(
            "async def foo(): ...",
            function_stubs(&function, &Imports::default(), None)
        )
    }

    #[test]
    fn test_import() {
        let big_type = Expr::Subscript {
            value: Box::new(Expr::Name { id: "dict".into() }),
            slice: Box::new(Expr::Tuple {
                elts: vec![
                    Expr::Attribute {
                        value: Box::new(Expr::Name {
                            id: "foo.bar".into(),
                        }),
                        attr: "A".into(),
                    },
                    Expr::Tuple {
                        elts: vec![
                            Expr::Attribute {
                                value: Box::new(Expr::Name { id: "foo".into() }),
                                attr: "A.C".into(),
                            },
                            Expr::Attribute {
                                value: Box::new(Expr::Attribute {
                                    value: Box::new(Expr::Name { id: "foo".into() }),
                                    attr: "A".into(),
                                }),
                                attr: "D".into(),
                            },
                            Expr::Attribute {
                                value: Box::new(Expr::Name { id: "foo".into() }),
                                attr: "B".into(),
                            },
                            Expr::Attribute {
                                value: Box::new(Expr::Name { id: "bat".into() }),
                                attr: "A".into(),
                            },
                            Expr::Attribute {
                                value: Box::new(Expr::Name {
                                    id: "foo.bar".into(),
                                }),
                                attr: "int".into(),
                            },
                            Expr::Name { id: "int".into() },
                            Expr::Name { id: "float".into() },
                        ],
                    },
                ],
            }),
        };
        let imports = Imports::create(
            &Module {
                name: "bar".into(),
                modules: Vec::new(),
                classes: vec![
                    Class {
                        name: "A".into(),
                        bases: vec![Expr::Name { id: "dict".into() }],
                        methods: Vec::new(),
                        attributes: Vec::new(),
                        decorators: vec![Expr::Attribute {
                            value: Box::new(Expr::Name {
                                id: "typing".into(),
                            }),
                            attr: "final".into(),
                        }],
                        inner_classes: Vec::new(),
                        docstring: None,
                    },
                    Class {
                        name: "int".into(),
                        bases: Vec::new(),
                        methods: Vec::new(),
                        attributes: Vec::new(),
                        decorators: Vec::new(),
                        inner_classes: Vec::new(),
                        docstring: None,
                    },
                ],
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
                    returns: Some(big_type.clone()),
                    is_async: false,
                    docstring: None,
                }],
                attributes: Vec::new(),
                incomplete: true,
                docstring: None,
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
                "from typing import final"
            ]
        );
        let mut output = String::new();
        imports.serialize_expr(&big_type, &mut output);
        assert_eq!(output, "dict[A, (A3.C, A3.D, B, A2, int, int2, float)]");
    }
}
