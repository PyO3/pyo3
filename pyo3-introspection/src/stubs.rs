use crate::model::{
    Argument, Arguments, Attribute, Class, Function, Module, TypeHint, TypeHintImport,
    VariableLengthArgument,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

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
    let mut imports = Imports::new();
    let mut elements = Vec::new();
    for attribute in &module.attributes {
        elements.push(attribute_stubs(attribute, &mut imports));
    }
    for class in &module.classes {
        elements.push(class_stubs(class, &mut imports));
    }
    for function in &module.functions {
        elements.push(function_stubs(function, &mut imports));
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
                        annotation: Some(TypeHint {
                            annotation: "str".into(),
                            imports: Vec::new(),
                        }),
                    }],
                    vararg: None,
                    keyword_only_arguments: Vec::new(),
                    kwarg: None,
                },
                returns: Some(TypeHint {
                    annotation: "Incomplete".into(),
                    imports: vec![TypeHintImport {
                        module: "_typeshed".into(),
                        name: "Incomplete".into(),
                    }],
                }),
            },
            &mut imports,
        ));
    }

    // We validate the imports
    imports.filter_for_module(&module.name, parents);

    let mut final_elements = imports.to_lines();
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

fn class_stubs(class: &Class, imports: &mut Imports) -> String {
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

fn function_stubs(function: &Function, imports: &mut Imports) -> String {
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
        buffer.push_str(type_hint_stub(returns, imports));
    }
    buffer.push_str(": ...");
    buffer
}

fn attribute_stubs(attribute: &Attribute, imports: &mut Imports) -> String {
    let mut output = attribute.name.clone();
    if let Some(annotation) = &attribute.annotation {
        output.push_str(": ");
        output.push_str(type_hint_stub(annotation, imports));
    }
    if let Some(value) = &attribute.value {
        output.push_str(" = ");
        output.push_str(value);
    }
    output
}

fn argument_stub(argument: &Argument, imports: &mut Imports) -> String {
    let mut output = argument.name.clone();
    if let Some(annotation) = &argument.annotation {
        output.push_str(": ");
        output.push_str(type_hint_stub(annotation, imports));
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
    imports: &mut Imports,
) -> String {
    let mut output = argument.name.clone();
    if let Some(annotation) = &argument.annotation {
        output.push_str(": ");
        output.push_str(type_hint_stub(annotation, imports));
    }
    output
}

fn type_hint_stub<'a>(annotation: &'a TypeHint, imports: &mut Imports) -> &'a str {
    for import in &annotation.imports {
        imports.add(import);
    }
    &annotation.annotation
}

/// Datastructure to deduplicate, validate and generate imports
struct Imports {
    /// module -> names
    imports: BTreeMap<String, BTreeSet<String>>,
}

impl Imports {
    fn new() -> Self {
        Self {
            imports: BTreeMap::new(),
        }
    }

    fn add(&mut self, import: &TypeHintImport) {
        self.imports
            .entry(import.module.clone())
            .or_default()
            .insert(import.name.clone());
    }

    /// Remove all local import paths i.e. 'foo' and 'bar.foo' if the module is 'bar.foo' (encoded as name = 'foo' and parents = \['bar'\]
    fn filter_for_module(&mut self, name: &str, parents: &[&str]) {
        let mut local_import_path = name.to_string();
        self.imports.remove(name);
        for parent in parents {
            local_import_path = format!("{local_import_path}.{parent}");
            self.imports.remove(&local_import_path);
        }
    }

    fn to_lines(&self) -> Vec<String> {
        let mut lines = Vec::with_capacity(self.imports.len());
        for (module, names) in &self.imports {
            let mut output = String::new();
            output.push_str("from ");
            output.push_str(module);
            output.push_str(" import ");
            for (i, name) in names.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(name);
            }
            lines.push(output);
        }
        lines
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
                    annotation: Some(TypeHint {
                        annotation: "str".into(),
                        imports: Vec::new(),
                    }),
                }],
                kwarg: Some(VariableLengthArgument {
                    name: "kwarg".into(),
                    annotation: Some(TypeHint {
                        annotation: "str".into(),
                        imports: Vec::new(),
                    }),
                }),
            },
            returns: Some(TypeHint {
                annotation: "list[str]".into(),
                imports: Vec::new(),
            }),
        };
        assert_eq!(
            "def func(posonly, /, arg, *varargs, karg: str, **kwarg: str) -> list[str]: ...",
            function_stubs(&function, &mut Imports::new())
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
                    annotation: Some(TypeHint {
                        annotation: "str".into(),
                        imports: Vec::new(),
                    }),
                }],
                kwarg: None,
            },
            returns: None,
        };
        assert_eq!(
            "def afunc(posonly=1, /, arg=True, *, karg: str = \"foo\"): ...",
            function_stubs(&function, &mut Imports::new())
        )
    }
}
