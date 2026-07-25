use crate::model::{
    Argument, Arguments, Attribute, Class, Constant, Expr, Function, Module, Operator,
    VariableLengthArgument,
};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
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

    let dunder_all = dunder_all_stubs(module, &imports);

    let mut final_elements = Vec::new();
    if let Some(docstring) = &module.docstring {
        final_elements.push(format!("\"\"\"\n{docstring}\n\"\"\""));
    }
    final_elements.extend(imports.imports);
    final_elements.extend(dunder_all);
    final_elements.extend(elements);

    let mut output = String::new();

    // We insert two line jumps (i.e. empty strings) only above and below multiple line elements
    // (classes with methods, functions with decorators) and the `__all__` declaration
    for element in final_elements {
        let needs_empty_lines = element.contains('\n') || element.starts_with("__all__");
        if needs_empty_lines && !output.is_empty() && !output.ends_with("\n\n") {
            output.push('\n');
        }
        output.push_str(&element);
        output.push('\n');
        if needs_empty_lines {
            output.push('\n');
        }
    }

    // We remove a line jump at the end if they are two
    if output.ends_with("\n\n") {
        output.pop();
    }
    output
}

/// Generates the `__all__` declaration of a module, if we are able to write an accurate one.
///
/// [`PyModuleMethods::add`] appends every name it adds to the module `__all__`, so any `#[pymodule]`
/// with at least one member has an `__all__` at runtime and the stub must declare it too.
///
/// We only emit it for complete modules: for an incomplete one we do not know the full list of
/// members, and an `__all__` missing some of them would hide from type checkers names that do exist
/// at runtime.
///
/// [`PyModuleMethods::add`]: https://docs.rs/pyo3/latest/pyo3/types/trait.PyModuleMethods.html#tymethod.add
fn dunder_all_stubs(module: &Module, imports: &Imports) -> Option<String> {
    if module.incomplete {
        return None;
    }
    if module.attributes.iter().any(|a| a.name == "__all__") {
        // The introspection data carries an explicit `__all__`, it is more accurate than ours
        return None;
    }
    let member_count = module.attributes.len()
        + module.classes.len()
        + module.functions.len()
        + module.modules.len();
    if member_count == 0 {
        // Nothing was ever added to the module, so it has no `__all__` at runtime either
        return None;
    }

    // Each of these lists is already sorted by name, so we just list the members in the order the
    // stub declares them, with the submodules (declared in their own file) last.
    let mut elts = Vec::with_capacity(member_count);
    elts.extend(
        module
            .attributes
            .iter()
            .map(|a| &a.name)
            .chain(module.classes.iter().map(|c| &c.name))
            .chain(module.functions.iter().map(|f| &f.name))
            .chain(module.modules.iter().map(|m| &m.name))
            .map(|name| Expr::Constant {
                value: Constant::Str(name.clone()),
            }),
    );

    let mut buffer = "__all__ = ".to_string();
    imports.serialize_expr(&Expr::List { elts }, &mut buffer);
    Some(buffer)
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
        push_docstring(&mut buffer, "    ", docstring);
    }
    for attribute in &class.attributes {
        // We do the indentation
        buffer.push_str("\n    ");
        push_indented(&mut buffer, "    ", &attribute_stubs(attribute, imports));
    }
    for method in &class.methods {
        // We do the indentation
        buffer.push_str("\n    ");
        push_indented(
            &mut buffer,
            "    ",
            &function_stubs(method, imports, Some(&class.name)),
        );
    }
    for inner_class in &class.inner_classes {
        // We do the indentation
        buffer.push_str("\n    ");
        push_indented(&mut buffer, "    ", &class_stubs(inner_class, imports));
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
        buffer.push(':');
        push_docstring(&mut buffer, "    ", docstring);
    } else {
        buffer.push_str(": ...");
    }
    buffer
}

/// Appends `text` to `buffer`, prefixing every line after the first with `indent`.
///
/// The first line is left alone because callers have already written the indentation for it; this
/// is the same contract `text.replace('\n', "\n{indent}")` had, minus one thing: a blank line stays
/// blank instead of being padded out to the indentation. Trailing whitespace on an otherwise empty
/// line is invisible in the source but still trailing whitespace, it trips `W293` in every Python
/// linter, and a generated file is exactly the kind of file nobody gets to hand-fix.
fn push_indented(buffer: &mut String, indent: &str, text: &str) {
    for (index, line) in text.split('\n').enumerate() {
        if index > 0 {
            buffer.push('\n');
            if !line.is_empty() {
                buffer.push_str(indent);
            }
        }
        buffer.push_str(line);
    }
}

/// Appends a `"""`-quoted docstring indented by `indent`, starting on a fresh line.
fn push_docstring(buffer: &mut String, indent: &str, docstring: &str) {
    buffer.push('\n');
    buffer.push_str(indent);
    buffer.push_str("\"\"\"");
    for line in docstring.lines() {
        buffer.push('\n');
        if !line.is_empty() {
            buffer.push_str(indent);
            buffer.push_str(line);
        }
    }
    buffer.push('\n');
    buffer.push_str(indent);
    buffer.push_str("\"\"\"");
}

/// Collects the operands of a `|` chain in source order, skipping repeats.
fn flatten_union<'a>(expr: &'a Expr, operands: &mut Vec<&'a Expr>, seen: &mut HashSet<&'a Expr>) {
    if let Expr::BinOp {
        left,
        op: Operator::BitOr,
        right,
    } = expr
    {
        flatten_union(left, operands, seen);
        flatten_union(right, operands, seen);
    } else if seen.insert(expr) {
        operands.push(expr);
    }
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
        push_docstring(&mut buffer, "", docstring);
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
        let module_is_package = !module.modules.is_empty();
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
                    "from {} import {}",
                    make_module_path_relative(module, &current_module_name, module_is_package),
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
            Expr::BinOp {
                op: Operator::BitOr,
                ..
            } => {
                // Union deduplication needs to happen here because the macro
                // generation only sees unresolved associated constants.
                let mut operands = Vec::new();
                flatten_union(expr, &mut operands, &mut HashSet::new());
                for (index, operand) in operands.into_iter().enumerate() {
                    if index > 0 {
                        buffer.push_str(" | ");
                    }
                    self.serialize_expr(operand, buffer);
                }
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

/// Returns the path of {module_path} relative to {current_module_path}
///
/// {module_path} must be different from {current_module_path}
///
/// Doc: <https://peps.python.org/pep-0328/#guido-s-decision>
fn make_module_path_relative<'a>(
    target_module_path: &'a str,
    current_module_path: &str,
    current_module_is_a_package: bool,
) -> Cow<'a, str> {
    assert_ne!(
        target_module_path, current_module_path,
        "Internal error: it is not possible to import elements declared locally"
    );

    // We split by component
    let mut target_module_path_components = target_module_path.split('.').peekable();
    let mut current_module_path_components = current_module_path.split('.').peekable();

    // We check if we can do a relative import, if not we do an absolute one
    if current_module_path_components.peek() != target_module_path_components.peek() {
        return Cow::Borrowed(target_module_path);
    }

    // We discard the equal ones
    while current_module_path_components.peek() == target_module_path_components.peek() {
        current_module_path_components.next();
        target_module_path_components.next();
    }

    let mut output = String::new();

    // We move "up" the remaining components in the current module
    for _ in current_module_path_components {
        output.push('.');
    }
    if current_module_is_a_package {
        // The current module is a package, we need to add an extra '.'
        output.push('.');
    }

    // We move "down" the remaining component in the target module
    for (i, component) in target_module_path_components.enumerate() {
        if i > 0 {
            output.push('.');
        }
        output.push_str(component);
    }

    Cow::Owned(output)
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
                "from . import A as A3, B",
                "from _typeshed import Incomplete",
                "from bat import A as A2",
                "from builtins import int as int2",
                "from typing import final"
            ]
        );
        let mut output = String::new();
        imports.serialize_expr(&big_type, &mut output);
        assert_eq!(output, "dict[A, (A3.C, A3.D, B, A2, int, int2, float)]");
    }

    fn empty_module(name: &str) -> Module {
        Module {
            name: name.into(),
            modules: Vec::new(),
            classes: Vec::new(),
            functions: Vec::new(),
            attributes: Vec::new(),
            incomplete: false,
            docstring: None,
        }
    }

    /// The `pytests` stubs cover the common cases, we only test the ones they don't have:
    /// a module with a submodule, and an empty module.
    #[test]
    fn test_dunder_all() {
        let module = Module {
            modules: vec![empty_module("sub")],
            classes: vec![Class {
                name: "Zulu".into(),
                bases: Vec::new(),
                methods: Vec::new(),
                attributes: Vec::new(),
                decorators: Vec::new(),
                inner_classes: Vec::new(),
                docstring: None,
            }],
            functions: vec![Function {
                name: "func".into(),
                decorators: Vec::new(),
                arguments: Arguments {
                    positional_only_arguments: Vec::new(),
                    arguments: Vec::new(),
                    vararg: None,
                    keyword_only_arguments: Vec::new(),
                    kwarg: None,
                },
                returns: None,
                is_async: false,
                docstring: None,
            }],
            attributes: vec![Attribute {
                name: "CONST".into(),
                value: Some(Expr::Constant {
                    value: Constant::Int("1".into()),
                }),
                annotation: None,
                docstring: None,
            }],
            ..empty_module("bar")
        };
        // The names are the ones `PyModuleMethods::add` puts in `__all__` at runtime, including
        // the submodule which is declared in its own stub file.
        assert_eq!(
            module_stubs(&module, &["foo"]),
            "__all__ = [\"CONST\", \"Zulu\", \"func\", \"sub\"]\n\nCONST = 1\nclass Zulu: ...\ndef func(): ...\n"
        );
        // Nothing was added to an empty module, so it has no `__all__` at runtime either
        assert_eq!(module_stubs(&empty_module("bar"), &["foo"]), "");
    }

    #[test]
    fn test_make_module_path_relative() {
        assert_eq!(
            make_module_path_relative("foo.bar", "foo.bar.baz", false),
            "."
        );
        assert_eq!(make_module_path_relative("foo", "foo.bar.baz", false), "..");
        assert_eq!(
            make_module_path_relative("foo.lat", "foo.bar.baz", false),
            "..lat"
        );
        assert_eq!(
            make_module_path_relative("foo.bar.baz", "foo.bar", true),
            ".baz"
        );
        assert_eq!(make_module_path_relative("foo", "foo.bar", true), "..");
        assert_eq!(
            make_module_path_relative("foo.lat", "foo.bar", true),
            "..lat"
        );
        assert_eq!(
            make_module_path_relative("foo.bar.baz", "foo", true),
            ".bar.baz"
        );
        assert_eq!(make_module_path_relative("foo.bar", "foo", true), ".bar");
        assert_eq!(make_module_path_relative("foo.lat", "foo", true), ".lat");
        assert_eq!(
            make_module_path_relative("foo.bar.baz", "foo.lat", false),
            ".bar.baz"
        );
        assert_eq!(
            make_module_path_relative("foo.bar", "foo.la", false),
            ".bar"
        );
        assert_eq!(make_module_path_relative("foo", "foo.la", false), ".");
        assert_eq!(make_module_path_relative("foo", "bar", true), "foo");
    }

    /// Docstrings are re-indented into the class or function body, and a paragraph break inside one
    /// is an empty line. Padding it out to the body indentation is trailing whitespace, which
    /// `W293` flags and which nobody can fix by hand in a generated file.
    #[test]
    fn docstring_blank_lines_are_not_padded_with_indentation() {
        let module = Module {
            name: "bar".into(),
            modules: Vec::new(),
            classes: vec![Class {
                name: "Zulu".into(),
                bases: Vec::new(),
                methods: vec![Function {
                    name: "method".into(),
                    decorators: Vec::new(),
                    arguments: Arguments {
                        positional_only_arguments: Vec::new(),
                        arguments: Vec::new(),
                        vararg: None,
                        keyword_only_arguments: Vec::new(),
                        kwarg: None,
                    },
                    returns: None,
                    is_async: false,
                    docstring: Some("Summary.\n\nDetail.".into()),
                }],
                attributes: Vec::new(),
                decorators: Vec::new(),
                inner_classes: Vec::new(),
                docstring: Some("Class summary.\n\nClass detail.".into()),
            }],
            functions: Vec::new(),
            attributes: vec![Attribute {
                name: "CONST".into(),
                value: None,
                annotation: None,
                docstring: Some("Const summary.\n\nConst detail.".into()),
            }],
            incomplete: false,
            docstring: None,
        };

        let stubs = module_stubs(&module, &["foo"]);
        assert!(
            !stubs
                .lines()
                .any(|line| !line.is_empty() && line.trim().is_empty()),
            "generated stubs contain a blank line padded with whitespace:\n{stubs:?}"
        );
        // The indentation of the non-empty lines is unaffected.
        assert!(stubs.contains("\n    Class summary.\n\n    Class detail.\n"));
        assert!(stubs.contains("\n        Summary.\n\n        Detail.\n"));
        assert!(stubs.contains("\nConst summary.\n\nConst detail.\n"));
    }

    #[test]
    fn union_members_are_deduplicated_and_spaced() {
        let str_ = || Expr::Name { id: "str".into() };
        let path_like = || Expr::Subscript {
            value: Box::new(Expr::Attribute {
                value: Box::new(Expr::Name { id: "os".into() }),
                attr: "PathLike".into(),
            }),
            slice: Box::new(str_()),
        };
        let union = |left: Expr, right: Expr| Expr::BinOp {
            left: Box::new(left),
            op: Operator::BitOr,
            right: Box::new(right),
        };
        let imports = Imports {
            imports: Vec::new(),
            renaming: BTreeMap::from([
                (("builtins".into(), "str".into()), "str".into()),
                (("os".into(), "PathLike".into()), "PathLike".into()),
            ]),
        };
        let serialize = |expr| {
            let mut buffer = String::new();
            imports.serialize_expr(&expr, &mut buffer);
            buffer
        };

        // `str | os.PathLike[str] | str`, nested to the right
        assert_eq!(
            serialize(union(str_(), union(path_like(), str_()))),
            "str | PathLike[str]"
        );
        // and the same chain nested to the left
        assert_eq!(
            serialize(union(union(str_(), path_like()), str_())),
            "str | PathLike[str]"
        );
    }
}
