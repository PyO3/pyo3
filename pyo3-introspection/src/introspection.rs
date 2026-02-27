use crate::model::{
    Argument, Arguments, Attribute, Class, Constant, Expr, Function, Module, Operator,
    VariableLengthArgument,
};
use anyhow::{anyhow, bail, ensure, Context, Result};
use goblin::elf::section_header::SHN_XINDEX;
use goblin::elf::Elf;
use goblin::mach::load_command::CommandVariant;
use goblin::mach::symbols::{NO_SECT, N_SECT};
use goblin::mach::{Mach, MachO, SingleArch};
use goblin::pe::PE;
use goblin::Object;
use serde::Deserialize;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;
use std::{fs, str};

/// Introspect a cdylib built with PyO3 and returns the definition of a Python module.
///
/// This function currently supports the ELF (most *nix including Linux), Match-O (macOS) and PE (Windows) formats.
pub fn introspect_cdylib(library_path: impl AsRef<Path>, main_module_name: &str) -> Result<Module> {
    let chunks = find_introspection_chunks_in_binary_object(library_path.as_ref())?;
    parse_chunks(&chunks, main_module_name)
}

/// Parses the introspection chunks found in the binary
fn parse_chunks(chunks: &[Chunk], main_module_name: &str) -> Result<Module> {
    let mut chunks_by_id = HashMap::<&str, &Chunk>::new();
    let mut chunks_by_parent = HashMap::<&str, Vec<&Chunk>>::new();
    for chunk in chunks {
        let (id, parent) = match chunk {
            Chunk::Module { id, .. } => (Some(id.as_str()), None),
            Chunk::Class { id, parent, .. } => (Some(id.as_str()), parent.as_deref()),
            Chunk::Function { id, parent, .. } | Chunk::Attribute { id, parent, .. } => {
                (id.as_deref(), parent.as_deref())
            }
        };
        if let Some(id) = id {
            chunks_by_id.insert(id, chunk);
        }
        if let Some(parent) = parent {
            chunks_by_parent.entry(parent).or_default().push(chunk);
        }
    }
    // We look for the root chunk
    for chunk in chunks {
        if let Chunk::Module {
            id,
            name,
            members,
            doc,
            incomplete,
        } = chunk
        {
            if name == main_module_name {
                let type_hint_for_annotation_id = introspection_id_to_type_hint_for_root_module(
                    chunk,
                    &chunks_by_id,
                    &chunks_by_parent,
                );
                return convert_module(
                    id,
                    name,
                    members,
                    *incomplete,
                    doc.as_deref(),
                    &chunks_by_id,
                    &chunks_by_parent,
                    &type_hint_for_annotation_id,
                );
            }
        }
    }
    bail!("No module named {main_module_name} found")
}

#[expect(clippy::too_many_arguments)]
fn convert_module(
    id: &str,
    name: &str,
    members: &[String],
    mut incomplete: bool,
    docstring: Option<&str>,
    chunks_by_id: &HashMap<&str, &Chunk>,
    chunks_by_parent: &HashMap<&str, Vec<&Chunk>>,
    type_hint_for_annotation_id: &HashMap<String, Expr>,
) -> Result<Module> {
    let mut member_chunks = chunks_by_parent
        .get(&id)
        .into_iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    for member in members {
        if let Some(c) = chunks_by_id.get(member.as_str()) {
            member_chunks.push(*c);
        } else {
            incomplete = true; // We don't find an element
        }
    }
    let (modules, classes, functions, attributes) = convert_members(
        member_chunks,
        chunks_by_id,
        chunks_by_parent,
        type_hint_for_annotation_id,
    )?;

    Ok(Module {
        name: name.into(),
        modules,
        classes,
        functions,
        attributes,
        incomplete,
        docstring: docstring.map(Into::into),
    })
}

type Members = (Vec<Module>, Vec<Class>, Vec<Function>, Vec<Attribute>);

/// Convert a list of members of a module or a class
fn convert_members<'a>(
    chunks: impl IntoIterator<Item = &'a Chunk>,
    chunks_by_id: &HashMap<&str, &Chunk>,
    chunks_by_parent: &HashMap<&str, Vec<&Chunk>>,
    type_hint_for_annotation_id: &HashMap<String, Expr>,
) -> Result<Members> {
    let mut modules = Vec::new();
    let mut classes = Vec::new();
    let mut functions = Vec::new();
    let mut attributes = Vec::new();
    for chunk in chunks {
        match chunk {
            Chunk::Module {
                name,
                id,
                members,
                incomplete,
                doc,
            } => {
                modules.push(convert_module(
                    id,
                    name,
                    members,
                    *incomplete,
                    doc.as_deref(),
                    chunks_by_id,
                    chunks_by_parent,
                    type_hint_for_annotation_id,
                )?);
            }
            Chunk::Class {
                name,
                id,
                bases,
                decorators,
                doc,
                parent: _,
            } => classes.push(convert_class(
                id,
                name,
                bases,
                decorators,
                doc.as_deref(),
                chunks_by_id,
                chunks_by_parent,
                type_hint_for_annotation_id,
            )?),
            Chunk::Function {
                name,
                id: _,
                arguments,
                parent: _,
                decorators,
                is_async,
                returns,
                doc,
            } => functions.push(convert_function(
                name,
                arguments,
                decorators,
                returns,
                *is_async,
                doc.as_deref(),
                type_hint_for_annotation_id,
            )),
            Chunk::Attribute {
                name,
                id: _,
                parent: _,
                value,
                annotation,
                doc,
            } => attributes.push(convert_attribute(
                name,
                value,
                annotation,
                doc.as_deref(),
                type_hint_for_annotation_id,
            )),
        }
    }
    // We sort elements to get a stable output
    modules.sort_by(|l, r| l.name.cmp(&r.name));
    classes.sort_by(|l, r| l.name.cmp(&r.name));
    functions.sort_by(|l, r| match l.name.cmp(&r.name) {
        Ordering::Equal => {
            fn decorator_expr_key(expr: &Expr) -> (u32, Cow<'_, str>) {
                // We put plain names before attributes for @property to be before @foo.property
                match expr {
                    Expr::Name { id, .. } => (0, Cow::Borrowed(id)),
                    Expr::Attribute { value, attr } => {
                        let (c, v) = decorator_expr_key(value);
                        (c + 1, Cow::Owned(format!("{v}.{attr}")))
                    }
                    _ => (0, Cow::Borrowed("")), // We don't care
                }
            }
            // We pick an ordering based on decorators
            l.decorators
                .iter()
                .map(decorator_expr_key)
                .cmp(r.decorators.iter().map(decorator_expr_key))
        }
        o => o,
    });
    attributes.sort_by(|l, r| l.name.cmp(&r.name));
    Ok((modules, classes, functions, attributes))
}

#[expect(clippy::too_many_arguments)]
fn convert_class(
    id: &str,
    name: &str,
    bases: &[ChunkExpr],
    decorators: &[ChunkExpr],
    docstring: Option<&str>,
    chunks_by_id: &HashMap<&str, &Chunk>,
    chunks_by_parent: &HashMap<&str, Vec<&Chunk>>,
    type_hint_for_annotation_id: &HashMap<String, Expr>,
) -> Result<Class> {
    let (nested_modules, nested_classes, methods, attributes) = convert_members(
        chunks_by_parent.get(&id).into_iter().flatten().copied(),
        chunks_by_id,
        chunks_by_parent,
        type_hint_for_annotation_id,
    )?;
    ensure!(
        nested_modules.is_empty(),
        "Classes cannot contain nested modules"
    );
    Ok(Class {
        name: name.into(),
        bases: bases
            .iter()
            .map(|e| convert_expr(e, type_hint_for_annotation_id))
            .collect(),
        methods,
        attributes,
        decorators: decorators
            .iter()
            .map(|e| convert_expr(e, type_hint_for_annotation_id))
            .collect(),
        inner_classes: nested_classes,
        docstring: docstring.map(Into::into),
    })
}

fn convert_function(
    name: &str,
    arguments: &ChunkArguments,
    decorators: &[ChunkExpr],
    returns: &Option<ChunkExpr>,
    is_async: bool,
    docstring: Option<&str>,
    type_hint_for_annotation_id: &HashMap<String, Expr>,
) -> Function {
    Function {
        name: name.into(),
        decorators: decorators
            .iter()
            .map(|e| convert_expr(e, type_hint_for_annotation_id))
            .collect(),
        arguments: Arguments {
            positional_only_arguments: arguments
                .posonlyargs
                .iter()
                .map(|a| convert_argument(a, type_hint_for_annotation_id))
                .collect(),
            arguments: arguments
                .args
                .iter()
                .map(|a| convert_argument(a, type_hint_for_annotation_id))
                .collect(),
            vararg: arguments
                .vararg
                .as_ref()
                .map(|a| convert_variable_length_argument(a, type_hint_for_annotation_id)),
            keyword_only_arguments: arguments
                .kwonlyargs
                .iter()
                .map(|e| convert_argument(e, type_hint_for_annotation_id))
                .collect(),
            kwarg: arguments
                .kwarg
                .as_ref()
                .map(|a| convert_variable_length_argument(a, type_hint_for_annotation_id)),
        },
        returns: returns
            .as_ref()
            .map(|a| convert_expr(a, type_hint_for_annotation_id)),
        is_async,
        docstring: docstring.map(Into::into),
    }
}

fn convert_argument(
    arg: &ChunkArgument,
    type_hint_for_annotation_id: &HashMap<String, Expr>,
) -> Argument {
    Argument {
        name: arg.name.clone(),
        default_value: arg
            .default
            .as_ref()
            .map(|e| convert_expr(e, type_hint_for_annotation_id)),
        annotation: arg
            .annotation
            .as_ref()
            .map(|a| convert_expr(a, type_hint_for_annotation_id)),
    }
}

fn convert_variable_length_argument(
    arg: &ChunkArgument,
    type_hint_for_annotation_id: &HashMap<String, Expr>,
) -> VariableLengthArgument {
    VariableLengthArgument {
        name: arg.name.clone(),
        annotation: arg
            .annotation
            .as_ref()
            .map(|a| convert_expr(a, type_hint_for_annotation_id)),
    }
}

fn convert_attribute(
    name: &str,
    value: &Option<ChunkExpr>,
    annotation: &Option<ChunkExpr>,
    docstring: Option<&str>,
    type_hint_for_annotation_id: &HashMap<String, Expr>,
) -> Attribute {
    Attribute {
        name: name.into(),
        value: value
            .as_ref()
            .map(|v| convert_expr(v, type_hint_for_annotation_id)),
        annotation: annotation
            .as_ref()
            .map(|a| convert_expr(a, type_hint_for_annotation_id)),
        docstring: docstring.map(ToString::to_string),
    }
}

fn convert_expr(expr: &ChunkExpr, type_hint_for_annotation_id: &HashMap<String, Expr>) -> Expr {
    match expr {
        ChunkExpr::Name { id } => Expr::Name { id: id.clone() },
        ChunkExpr::Attribute { value, attr } => Expr::Attribute {
            value: Box::new(convert_expr(value, type_hint_for_annotation_id)),
            attr: attr.clone(),
        },
        ChunkExpr::BinOp { left, op, right } => Expr::BinOp {
            left: Box::new(convert_expr(left, type_hint_for_annotation_id)),
            op: match op {
                ChunkOperator::BitOr => Operator::BitOr,
            },
            right: Box::new(convert_expr(right, type_hint_for_annotation_id)),
        },
        ChunkExpr::Subscript { value, slice } => Expr::Subscript {
            value: Box::new(convert_expr(value, type_hint_for_annotation_id)),
            slice: Box::new(convert_expr(slice, type_hint_for_annotation_id)),
        },
        ChunkExpr::Tuple { elts } => Expr::Tuple {
            elts: elts
                .iter()
                .map(|e| convert_expr(e, type_hint_for_annotation_id))
                .collect(),
        },
        ChunkExpr::List { elts } => Expr::List {
            elts: elts
                .iter()
                .map(|e| convert_expr(e, type_hint_for_annotation_id))
                .collect(),
        },
        ChunkExpr::Constant { value } => Expr::Constant {
            value: match value {
                ChunkConstant::None => Constant::None,
                ChunkConstant::Bool { value } => Constant::Bool(*value),
                ChunkConstant::Int { value } => Constant::Int(value.clone()),
                ChunkConstant::Float { value } => Constant::Float(value.clone()),
                ChunkConstant::Str { value } => Constant::Str(value.clone()),
                ChunkConstant::Ellipsis => Constant::Ellipsis,
            },
        },
        ChunkExpr::Id { id } => {
            if let Some(expr) = type_hint_for_annotation_id.get(id) {
                expr.clone()
            } else {
                // This is a pyclass not exposed, we fallback to Any
                Expr::Attribute {
                    value: Box::new(Expr::Name {
                        id: "typing".into(),
                    }),
                    attr: "Any".to_string(),
                }
            }
        }
    }
}

/// Returns the type hint for each class introspection id defined in the module and its submodule
fn introspection_id_to_type_hint_for_root_module(
    module_chunk: &Chunk,
    chunks_by_id: &HashMap<&str, &Chunk>,
    chunks_by_parent: &HashMap<&str, Vec<&Chunk>>,
) -> HashMap<String, Expr> {
    fn add_introspection_id_to_type_hint_for_module_members(
        module_id: &str,
        module_full_name: &str,
        module_members: &[String],
        chunks_by_id: &HashMap<&str, &Chunk>,
        chunks_by_parent: &HashMap<&str, Vec<&Chunk>>,
        output: &mut HashMap<String, Expr>,
    ) {
        for member in chunks_by_parent
            .get(&module_id)
            .into_iter()
            .flatten()
            .chain(
                module_members
                    .iter()
                    .filter_map(|id| chunks_by_id.get(id.as_str())),
            )
            .copied()
        {
            match member {
                Chunk::Module {
                    name, id, members, ..
                } => {
                    add_introspection_id_to_type_hint_for_module_members(
                        id,
                        &format!("{}.{}", module_full_name, name),
                        members,
                        chunks_by_id,
                        chunks_by_parent,
                        output,
                    );
                }
                Chunk::Class { id, name, .. } => {
                    output.insert(
                        id.clone(),
                        Expr::Attribute {
                            value: Box::new(Expr::Name {
                                id: module_full_name.into(),
                            }),
                            attr: name.clone(),
                        },
                    );
                    add_introspection_id_to_type_hint_for_class_subclasses(
                        id,
                        name,
                        module_full_name,
                        chunks_by_parent,
                        output,
                    );
                }
                _ => (),
            }
        }
    }

    fn add_introspection_id_to_type_hint_for_class_subclasses(
        class_id: &str,
        class_name: &str,
        class_module: &str,
        chunks_by_parent: &HashMap<&str, Vec<&Chunk>>,
        output: &mut HashMap<String, Expr>,
    ) {
        for member in chunks_by_parent.get(&class_id).into_iter().flatten() {
            if let Chunk::Class { id, name, .. } = member {
                let class_name = format!("{}.{}", class_name, name);
                add_introspection_id_to_type_hint_for_class_subclasses(
                    id,
                    &class_name,
                    class_module,
                    chunks_by_parent,
                    output,
                );
                output.insert(
                    id.clone(),
                    Expr::Attribute {
                        value: Box::new(Expr::Name {
                            id: class_module.into(),
                        }),
                        attr: class_name,
                    },
                );
            }
        }
    }

    let mut output = HashMap::new();
    let Chunk::Module {
        id, name, members, ..
    } = module_chunk
    else {
        unreachable!("The chunk must be a module")
    };
    add_introspection_id_to_type_hint_for_module_members(
        id,
        name,
        members,
        chunks_by_id,
        chunks_by_parent,
        &mut output,
    );
    output
}

fn find_introspection_chunks_in_binary_object(path: &Path) -> Result<Vec<Chunk>> {
    let library_content =
        fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
    match Object::parse(&library_content)
        .context("The built library is not valid or not supported by our binary parser")?
    {
        Object::Elf(elf) => find_introspection_chunks_in_elf(&elf, &library_content),
        Object::Mach(Mach::Binary(macho)) => {
            find_introspection_chunks_in_macho(&macho, &library_content)
        }
        Object::Mach(Mach::Fat(multi_arch)) => {
            for arch in &multi_arch {
                match arch? {
                    SingleArch::MachO(macho) => {
                        return find_introspection_chunks_in_macho(&macho, &library_content)
                    }
                    SingleArch::Archive(_) => (),
                }
            }
            bail!("No Mach-o chunk found in the multi-arch Mach-o container")
        }
        Object::PE(pe) => find_introspection_chunks_in_pe(&pe, &library_content),
        _ => {
            bail!("Only ELF, Mach-o and PE containers can be introspected")
        }
    }
}

fn find_introspection_chunks_in_elf(elf: &Elf<'_>, library_content: &[u8]) -> Result<Vec<Chunk>> {
    let mut chunks = Vec::new();
    for sym in &elf.syms {
        if is_introspection_symbol(elf.strtab.get_at(sym.st_name).unwrap_or_default()) {
            ensure!(u32::try_from(sym.st_shndx)? != SHN_XINDEX, "Section names length is greater than SHN_LORESERVE in ELF, this is not supported by PyO3 yet");
            let section_header = &elf.section_headers[sym.st_shndx];
            let data_offset = sym.st_value + section_header.sh_offset - section_header.sh_addr;
            chunks.push(deserialize_chunk(
                &library_content[usize::try_from(data_offset).context("File offset overflow")?..],
                elf.little_endian,
            )?);
        }
    }
    Ok(chunks)
}

fn find_introspection_chunks_in_macho(
    macho: &MachO<'_>,
    library_content: &[u8],
) -> Result<Vec<Chunk>> {
    if !macho.little_endian {
        bail!("Only little endian Mach-o binaries are supported");
    }
    ensure!(
        !macho.load_commands.iter().any(|command| {
            matches!(command.command, CommandVariant::DyldChainedFixups(_))
        }),
        "Mach-O binaries with fixup chains are not supported yet, to avoid using fixup chains, use `--codegen=link-arg=-no_fixup_chains` option."
    );

    let sections = macho
        .segments
        .sections()
        .flatten()
        .map(|t| t.map(|s| s.0))
        .collect::<Result<Vec<_>, _>>()?;
    let mut chunks = Vec::new();
    for symbol in macho.symbols() {
        let (name, nlist) = symbol?;
        if nlist.is_global()
            && nlist.get_type() == N_SECT
            && nlist.n_sect != NO_SECT as usize
            && is_introspection_symbol(name)
        {
            let section = &sections[nlist.n_sect - 1]; // Sections are counted from 1
            let data_offset = nlist.n_value + u64::from(section.offset) - section.addr;
            chunks.push(deserialize_chunk(
                &library_content[usize::try_from(data_offset).context("File offset overflow")?..],
                macho.little_endian,
            )?);
        }
    }
    Ok(chunks)
}

fn find_introspection_chunks_in_pe(pe: &PE<'_>, library_content: &[u8]) -> Result<Vec<Chunk>> {
    let mut chunks = Vec::new();
    for export in &pe.exports {
        if is_introspection_symbol(export.name.unwrap_or_default()) {
            chunks.push(deserialize_chunk(
                &library_content[export.offset.context("No symbol offset")?..],
                true,
            )?);
        }
    }
    Ok(chunks)
}

fn deserialize_chunk(
    content_with_chunk_at_the_beginning: &[u8],
    is_little_endian: bool,
) -> Result<Chunk> {
    let length = content_with_chunk_at_the_beginning
        .split_at(4)
        .0
        .try_into()
        .context("The introspection chunk must contain a length")?;
    let length = if is_little_endian {
        u32::from_le_bytes(length)
    } else {
        u32::from_be_bytes(length)
    };
    let chunk = content_with_chunk_at_the_beginning
        .get(4..4 + length as usize)
        .ok_or_else(|| {
            anyhow!("The introspection chunk length {length} is greater that the binary size")
        })?;
    serde_json::from_slice(chunk).with_context(|| {
        format!(
            "Failed to parse introspection chunk: {:?}",
            String::from_utf8_lossy(chunk)
        )
    })
}

fn is_introspection_symbol(name: &str) -> bool {
    name.strip_prefix('_')
        .unwrap_or(name)
        .starts_with("PYO3_INTROSPECTION_1_")
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Chunk {
    Module {
        id: String,
        name: String,
        members: Vec<String>,
        #[serde(default)]
        doc: Option<String>,
        incomplete: bool,
    },
    Class {
        id: String,
        name: String,
        #[serde(default)]
        bases: Vec<ChunkExpr>,
        #[serde(default)]
        decorators: Vec<ChunkExpr>,
        #[serde(default)]
        parent: Option<String>,
        #[serde(default)]
        doc: Option<String>,
    },
    Function {
        #[serde(default)]
        id: Option<String>,
        name: String,
        arguments: Box<ChunkArguments>,
        #[serde(default)]
        parent: Option<String>,
        #[serde(default)]
        decorators: Vec<ChunkExpr>,
        #[serde(default)]
        returns: Option<ChunkExpr>,
        #[serde(default, rename = "async")]
        is_async: bool,
        #[serde(default)]
        doc: Option<String>,
    },
    Attribute {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        parent: Option<String>,
        name: String,
        #[serde(default)]
        value: Option<ChunkExpr>,
        #[serde(default)]
        annotation: Option<ChunkExpr>,
        #[serde(default)]
        doc: Option<String>,
    },
}

#[derive(Deserialize)]
struct ChunkArguments {
    #[serde(default)]
    posonlyargs: Vec<ChunkArgument>,
    #[serde(default)]
    args: Vec<ChunkArgument>,
    #[serde(default)]
    vararg: Option<ChunkArgument>,
    #[serde(default)]
    kwonlyargs: Vec<ChunkArgument>,
    #[serde(default)]
    kwarg: Option<ChunkArgument>,
}

#[derive(Deserialize)]
struct ChunkArgument {
    name: String,
    #[serde(default)]
    default: Option<ChunkExpr>,
    #[serde(default)]
    annotation: Option<ChunkExpr>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ChunkExpr {
    /// A constant like `None` or `123`
    Constant {
        #[serde(flatten)]
        value: ChunkConstant,
    },
    /// A name
    Name { id: String },
    /// An attribute `value.attr`
    Attribute { value: Box<Self>, attr: String },
    /// A binary operator
    BinOp {
        left: Box<Self>,
        op: ChunkOperator,
        right: Box<Self>,
    },
    /// A tuple
    Tuple { elts: Vec<Self> },
    /// A list
    List { elts: Vec<Self> },
    /// A subscript `value[slice]`
    Subscript { value: Box<Self>, slice: Box<Self> },
    /// An introspection id
    Id { id: String },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ChunkConstant {
    None,
    Bool { value: bool },
    Int { value: String },
    Float { value: String },
    Str { value: String },
    Ellipsis,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChunkOperator {
    BitOr,
}
