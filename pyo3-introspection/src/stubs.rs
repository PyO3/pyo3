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
    for class in &module.classes {
        elements.push(class_stubs(class));
    }
    for function in &module.functions {
        elements.push(function_stubs(function));
    }
    for konst in &module.consts {
        elements.push(const_stubs(konst, &mut modules_to_import));
    }

    let mut output = String::new();

    for module_to_import in &modules_to_import {
        output.push_str(&format!("import {module_to_import}\n"));
    }

    if !modules_to_import.is_empty() {
        output.push('\n')
    }

    // We insert two line jumps (i.e. empty strings) only above and below multiple line elements (classes with methods, functions with decorators)
    for element in elements {
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

fn class_stubs(class: &Class) -> String {
    let mut buffer = format!("class {}:", class.name);
    if class.methods.is_empty() {
        buffer.push_str(" ...");
        return buffer;
    }
    for method in &class.methods {
        // We do the indentation
        buffer.push_str("\n    ");
        buffer.push_str(&function_stubs(method).replace('\n', "\n    "));
    }
    buffer
}

fn function_stubs(function: &Function) -> String {
    // Signature
    let mut parameters = Vec::new();
    for argument in &function.arguments.positional_only_arguments {
        parameters.push(argument_stub(argument));
    }
    if !function.arguments.positional_only_arguments.is_empty() {
        parameters.push("/".into());
    }
    for argument in &function.arguments.arguments {
        parameters.push(argument_stub(argument));
    }
    if let Some(argument) = &function.arguments.vararg {
        parameters.push(format!("*{}", variable_length_argument_stub(argument)));
    } else if !function.arguments.keyword_only_arguments.is_empty() {
        parameters.push("*".into());
    }
    for argument in &function.arguments.keyword_only_arguments {
        parameters.push(argument_stub(argument));
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

fn argument_stub(argument: &Argument) -> String {
    let mut output = argument.name.clone();
    if let Some(default_value) = &argument.default_value {
        output.push('=');
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
                }],
                arguments: vec![Argument {
                    name: "arg".into(),
                    default_value: None,
                }],
                vararg: Some(VariableLengthArgument {
                    name: "varargs".into(),
                }),
                keyword_only_arguments: vec![Argument {
                    name: "karg".into(),
                    default_value: None,
                }],
                kwarg: Some(VariableLengthArgument {
                    name: "kwarg".into(),
                }),
            },
        };
        assert_eq!(
            "def func(posonly, /, arg, *varargs, karg, **kwarg): ...",
            function_stubs(&function)
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
                }],
                arguments: vec![Argument {
                    name: "arg".into(),
                    default_value: Some("True".into()),
                }],
                vararg: None,
                keyword_only_arguments: vec![Argument {
                    name: "karg".into(),
                    default_value: Some("\"foo\"".into()),
                }],
                kwarg: None,
            },
        };
        assert_eq!(
            "def afunc(posonly=1, /, arg=True, *, karg=\"foo\"): ...",
            function_stubs(&function)
        )
    }
}
