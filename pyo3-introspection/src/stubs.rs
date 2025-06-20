use crate::model::{Argument, Class, Const, Function, Module, VariableLengthArgument};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

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
    for konst in &module.consts {
        elements.push(const_stubs(konst, &mut modules_to_import));
    }
    for class in &module.classes {
        elements.push(class_stubs(class, &mut modules_to_import));
    }
    for function in &module.functions {
        elements.push(function_stubs(function, &mut modules_to_import));
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
    if class.methods.is_empty() {
        buffer.push_str(" ...");
        return buffer;
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
        parameters.push(format!("*{}", variable_length_argument_stub(argument)));
    } else if !function.arguments.keyword_only_arguments.is_empty() {
        parameters.push("*".into());
    }
    for argument in &function.arguments.keyword_only_arguments {
        parameters.push(argument_stub(argument, modules_to_import));
    }
    if let Some(argument) = &function.arguments.kwarg {
        parameters.push(format!("**{}", variable_length_argument_stub(argument)));
    }
    let output = format!("def {}({}): ...", function.name, parameters.join(", "));
    if function.decorators.is_empty() {
        return output;
    }
    let mut buffer = String::new();
    for decorator in &function.decorators {
        buffer.push('@');
        buffer.push_str(decorator);
        buffer.push('\n');
    }
    buffer.push_str(&output);
    buffer
}

fn const_stubs(konst: &Const, modules_to_import: &mut BTreeSet<String>) -> String {
    modules_to_import.insert("typing".to_string());
    let Const { name, value } = konst;
    format!("{name}: typing.Final = {value}")
}

fn argument_stub(argument: &Argument, modules_to_import: &mut BTreeSet<String>) -> String {
    let mut output = argument.name.clone();
    if let Some(annotation) = &argument.annotation {
        output.push_str(": ");
        output.push_str(annotation);
        if let Some((module, _)) = annotation.rsplit_once('.') {
            // TODO: this is very naive
            modules_to_import.insert(module.into());
        }
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

fn variable_length_argument_stub(argument: &VariableLengthArgument) -> String {
    argument.name.clone()
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
                }),
                keyword_only_arguments: vec![Argument {
                    name: "karg".into(),
                    default_value: None,
                    annotation: Some("str".into()),
                }],
                kwarg: Some(VariableLengthArgument {
                    name: "kwarg".into(),
                }),
            },
        };
        assert_eq!(
            "def func(posonly, /, arg, *varargs, karg: str, **kwarg): ...",
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
        };
        assert_eq!(
            "def afunc(posonly=1, /, arg=True, *, karg: str = \"foo\"): ...",
            function_stubs(&function, &mut BTreeSet::new())
        )
    }
}
