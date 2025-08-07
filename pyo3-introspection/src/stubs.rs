use crate::model::{
    Argument, Arguments, Attribute, Class, Function, Module, VariableLengthArgument,
};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use unicode_ident::{is_xid_continue, is_xid_start};

/// Generates the [type stubs](https://typing.readthedocs.io/en/latest/source/stubs.html) of a given module.
/// It returns a map between the file name and the file content.
/// The root module stubs will be in the `__init__.pyi` file and the submodules directory
/// in files with a relevant name.
pub fn module_stub_files(module: &Module) -> HashMap<PathBuf, String> {
    let mut output_files = HashMap::new();
    add_module_stub_files(module, Path::new(""), &mut output_files);
    output_files
}

fn add_module_stub_files(
    module: &Module,
    module_path: &Path,
    output_files: &mut HashMap<PathBuf, String>,
) {
    output_files.insert(module_path.join("__init__.pyi"), module_stubs(module));
    for submodule in &module.modules {
        if submodule.modules.is_empty() {
            output_files.insert(
                module_path.join(format!("{}.pyi", submodule.name)),
                module_stubs(submodule),
            );
        } else {
            add_module_stub_files(submodule, &module_path.join(&submodule.name), output_files);
        }
    }
}

/// Generates the module stubs to a String, not including submodules
fn module_stubs(module: &Module) -> String {
    let mut modules_to_import = BTreeSet::new();
    let mut elements = Vec::new();
    for attribute in &module.attributes {
        elements.push(attribute_stubs(attribute, &mut modules_to_import));
    }
    for class in &module.classes {
        elements.push(class_stubs(class, &mut modules_to_import));
    }
    for function in &module.functions {
        elements.push(function_stubs(function, &mut modules_to_import));
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
                        annotation: Some("str".into()),
                    }],
                    vararg: None,
                    keyword_only_arguments: Vec::new(),
                    kwarg: None,
                },
                returns: Some("_typeshed.Incomplete".into()),
            },
            &mut modules_to_import,
        ));
    }

    let mut final_elements = Vec::new();
    for module_to_import in &modules_to_import {
        final_elements.push(format!("import {module_to_import}"));
    }
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

fn class_stubs(class: &Class, modules_to_import: &mut BTreeSet<String>) -> String {
    let mut buffer = format!("class {}:", class.name);
    if class.methods.is_empty() && class.attributes.is_empty() {
        buffer.push_str(" ...");
        return buffer;
    }
    for attribute in &class.attributes {
        // We do the indentation
        buffer.push_str("\n    ");
        buffer.push_str(&attribute_stubs(attribute, modules_to_import).replace('\n', "\n    "));
    }
    for method in &class.methods {
        // We do the indentation
        buffer.push_str("\n    ");
        buffer.push_str(&function_stubs(method, modules_to_import).replace('\n', "\n    "));
    }
    buffer
}

fn function_stubs(function: &Function, modules_to_import: &mut BTreeSet<String>) -> String {
    // Signature
    let mut parameters = Vec::new();
    for argument in &function.arguments.positional_only_arguments {
        parameters.push(argument_stub(argument, modules_to_import));
    }
    if !function.arguments.positional_only_arguments.is_empty() {
        parameters.push("/".into());
    }
    for argument in &function.arguments.arguments {
        parameters.push(argument_stub(argument, modules_to_import));
    }
    if let Some(argument) = &function.arguments.vararg {
        parameters.push(format!(
            "*{}",
            variable_length_argument_stub(argument, modules_to_import)
        ));
    } else if !function.arguments.keyword_only_arguments.is_empty() {
        parameters.push("*".into());
    }
    for argument in &function.arguments.keyword_only_arguments {
        parameters.push(argument_stub(argument, modules_to_import));
    }
    if let Some(argument) = &function.arguments.kwarg {
        parameters.push(format!(
            "**{}",
            variable_length_argument_stub(argument, modules_to_import)
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
        buffer.push_str(annotation_stub(returns, modules_to_import));
    }
    buffer.push_str(": ...");
    buffer
}

fn attribute_stubs(attribute: &Attribute, modules_to_import: &mut BTreeSet<String>) -> String {
    let mut output = attribute.name.clone();
    if let Some(annotation) = &attribute.annotation {
        output.push_str(": ");
        output.push_str(annotation_stub(annotation, modules_to_import));
    }
    if let Some(value) = &attribute.value {
        output.push_str(" = ");
        output.push_str(value);
    }
    output
}

fn argument_stub(argument: &Argument, modules_to_import: &mut BTreeSet<String>) -> String {
    let mut output = argument.name.clone();
    if let Some(annotation) = &argument.annotation {
        output.push_str(": ");
        output.push_str(annotation_stub(annotation, modules_to_import));
    }
    if let Some(default_value) = &argument.default_value {
        output.push_str(if argument.annotation.is_some() {
            " = "
        } else {
            "="
        });
        output.push_str(default_value);
    }
    output
}

fn variable_length_argument_stub(
    argument: &VariableLengthArgument,
    modules_to_import: &mut BTreeSet<String>,
) -> String {
    let mut output = argument.name.clone();
    if let Some(annotation) = &argument.annotation {
        output.push_str(": ");
        output.push_str(annotation_stub(annotation, modules_to_import));
    }
    output
}

fn annotation_stub<'a>(annotation: &'a str, modules_to_import: &mut BTreeSet<String>) -> &'a str {
    // We iterate on the annotation string
    // If it starts with a Python path like foo.bar, we add the module name (here foo) to the import list
    // and we skip after it
    let mut i = 0;
    while i < annotation.len() {
        if let Some(path) = path_prefix(&annotation[i..]) {
            // We found a path!
            i += path.len();
            if let Some((module, _)) = path.rsplit_once('.') {
                modules_to_import.insert(module.into());
            }
        }
        i += 1;
    }
    annotation
}

// If the input starts with a path like foo.bar, returns it
fn path_prefix(input: &str) -> Option<&str> {
    let mut length = identifier_prefix(input)?.len();
    loop {
        // We try to add another identifier to the path
        let Some(remaining) = input[length..].strip_prefix('.') else {
            break;
        };
        let Some(id) = identifier_prefix(remaining) else {
            break;
        };
        length += id.len() + 1;
    }
    Some(&input[..length])
}

// If the input starts with an identifier like foo, returns it
fn identifier_prefix(input: &str) -> Option<&str> {
    // We get the first char and validate it
    let mut iter = input.chars();
    let first_char = iter.next()?;
    if first_char != '_' && !is_xid_start(first_char) {
        return None;
    }
    let mut length = first_char.len_utf8();
    // We add extra chars as much as we can
    for c in iter {
        if is_xid_continue(c) {
            length += c.len_utf8();
        } else {
            break;
        }
    }
    Some(&input[0..length])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Arguments;

    #[test]
    fn annotation_stub_proper_imports() {
        let mut modules_to_import = BTreeSet::new();

        // Basic int
        annotation_stub("int", &mut modules_to_import);
        assert!(modules_to_import.is_empty());

        // Simple path
        annotation_stub("collections.abc.Iterable", &mut modules_to_import);
        assert!(modules_to_import.contains("collections.abc"));

        // With underscore
        annotation_stub("_foo._bar_baz", &mut modules_to_import);
        assert!(modules_to_import.contains("_foo"));

        // Basic generic
        annotation_stub("typing.List[int]", &mut modules_to_import);
        assert!(modules_to_import.contains("typing"));

        // Complex generic
        annotation_stub("typing.List[foo.Bar[int]]", &mut modules_to_import);
        assert!(modules_to_import.contains("foo"));

        // Callable
        annotation_stub(
            "typing.Callable[[int, baz.Bar], bar.Baz[bool]]",
            &mut modules_to_import,
        );
        assert!(modules_to_import.contains("bar"));
        assert!(modules_to_import.contains("baz"));

        // Union
        annotation_stub("a.B | b.C", &mut modules_to_import);
        assert!(modules_to_import.contains("a"));
        assert!(modules_to_import.contains("b"));
    }

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
                    annotation: Some("str".into()),
                }],
                kwarg: Some(VariableLengthArgument {
                    name: "kwarg".into(),
                    annotation: Some("str".into()),
                }),
            },
            returns: Some("list[str]".into()),
        };
        assert_eq!(
            "def func(posonly, /, arg, *varargs, karg: str, **kwarg: str) -> list[str]: ...",
            function_stubs(&function, &mut BTreeSet::new())
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
                    annotation: Some("str".into()),
                }],
                kwarg: None,
            },
            returns: None,
        };
        assert_eq!(
            "def afunc(posonly=1, /, arg=True, *, karg: str = \"foo\"): ...",
            function_stubs(&function, &mut BTreeSet::new())
        )
    }
}
