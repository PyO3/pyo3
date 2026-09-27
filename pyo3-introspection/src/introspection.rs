use crate::model::{
    Argument, Arguments, Attribute, Class, Constant, Expr, Function, Module, Operator,
    VariableLengthArgument,
};
use anyhow::{Context, Result, anyhow, bail, ensure};
use goblin::Object;
use goblin::elf::program_header::PT_LOAD;
use goblin::elf::section_header::SHN_UNDEF;
use goblin::elf::{Elf, Sym};
use goblin::mach::symbols::{N_SECT, NO_SECT};
use goblin::mach::{Mach, MachO, SingleArch};
use goblin::pe::PE;
use serde::Deserialize;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
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
                attach_to_root(chunk, id, &chunks_by_id, &mut chunks_by_parent);
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

/// Adds to the root module the classes declared with `attach_to_root` that it refers to.
///
/// A library crate cannot know the id of the root module it will be linked into, so a class it
/// declares itself (e.g. a `Protocol` for one of its `FromPyObject` implementations) cannot set
/// `parent`. These chunks are `#[used]` statics, linked in whether they are used or not, so only
/// the ones an annotation reachable from the root module refers to are kept.
fn attach_to_root<'a>(
    root: &'a Chunk,
    root_id: &'a str,
    chunks_by_id: &HashMap<&str, &'a Chunk>,
    chunks_by_parent: &mut HashMap<&'a str, Vec<&'a Chunk>>,
) {
    let mut attached = Vec::new();
    let mut to_visit = vec![root];
    while let Some(chunk) = to_visit.pop() {
        let (id, members) = match chunk {
            Chunk::Module { id, members, .. } => (Some(id.as_str()), members.as_slice()),
            Chunk::Class { id, .. } => (Some(id.as_str()), [].as_slice()),
            Chunk::Function { id, .. } | Chunk::Attribute { id, .. } => {
                (id.as_deref(), [].as_slice())
            }
        };
        to_visit.extend(
            id.and_then(|id| chunks_by_parent.get(id))
                .into_iter()
                .flatten(),
        );
        to_visit.extend(
            members
                .iter()
                .filter_map(|id| chunks_by_id.get(id.as_str())),
        );
        for id in referenced_ids(chunk) {
            let Some(&class) = chunks_by_id.get(id) else {
                continue;
            };
            if matches!(
                class,
                Chunk::Class {
                    attach_to_root: true,
                    ..
                }
            ) && !attached.contains(&id)
            {
                attached.push(id);
                to_visit.push(class);
            }
        }
    }
    let root_members = chunks_by_parent.entry(root_id).or_default();
    root_members.extend(attached.into_iter().map(|id| chunks_by_id[id]));
}

/// The introspection ids the expressions of a chunk refer to
fn referenced_ids(chunk: &Chunk) -> Vec<&str> {
    fn visit<'a>(expr: &'a ChunkExpr, output: &mut Vec<&'a str>) {
        match expr {
            ChunkExpr::Id { id } => output.push(id),
            ChunkExpr::Attribute { value, .. } => visit(value, output),
            ChunkExpr::BinOp { left, right, .. }
            | ChunkExpr::Subscript {
                value: left,
                slice: right,
            } => {
                visit(left, output);
                visit(right, output);
            }
            ChunkExpr::Tuple { elts } | ChunkExpr::List { elts } => {
                elts.iter().for_each(|e| visit(e, output))
            }
            ChunkExpr::Name { .. } | ChunkExpr::Constant { .. } => (),
        }
    }

    let exprs: Vec<&ChunkExpr> = match chunk {
        Chunk::Module { .. } => Vec::new(),
        Chunk::Class {
            bases, decorators, ..
        } => bases.iter().chain(decorators).collect(),
        Chunk::Function {
            arguments,
            decorators,
            returns,
            ..
        } => arguments
            .posonlyargs
            .iter()
            .chain(&arguments.args)
            .chain(&arguments.vararg)
            .chain(&arguments.kwonlyargs)
            .chain(&arguments.kwarg)
            .flat_map(|a| a.annotation.iter().chain(&a.default))
            .chain(decorators)
            .chain(returns)
            .collect(),
        Chunk::Attribute {
            value, annotation, ..
        } => value.iter().chain(annotation).collect(),
    };
    let mut output = Vec::new();
    for expr in exprs {
        visit(expr, &mut output);
    }
    output
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
                attach_to_root: _,
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
            for (arch, fat_arch) in multi_arch.into_iter().zip(multi_arch.iter_arches()) {
                match arch? {
                    SingleArch::MachO(macho) => {
                        // Mach-O offsets are relative to the beginning of the architecture slice
                        return find_introspection_chunks_in_macho(
                            &macho,
                            fat_arch?.slice(&library_content),
                        );
                    }
                    SingleArch::Archive(_) => (),
                }
            }
            bail!("No Mach-o chunk found in the multi-arch Mach-o container")
        }
        Object::PE(pe) => find_introspection_chunks_in_pe(&pe, &library_content),
        other => {
            bail!(
                "Only ELF, Mach-o and PE containers can be introspected, got {other:?} from file {path:?}",
                path = path.display()
            )
        }
    }
}

fn find_introspection_chunks_in_elf(elf: &Elf<'_>, library_content: &[u8]) -> Result<Vec<Chunk>> {
    // The introspection symbols are exported, so they are in the dynamic symbol table that
    // survives stripping. We also look at the full symbol table, if present, to support
    // binaries where they are not exported.
    let symbols = elf
        .dynsyms
        .iter()
        .map(|sym| (sym, elf.dynstrtab.get_at(sym.st_name)))
        .chain(
            elf.syms
                .iter()
                .map(|sym| (sym, elf.strtab.get_at(sym.st_name))),
        );
    let mut seen_names = HashSet::new();
    let mut chunks = Vec::new();
    for (sym, sym_name) in symbols {
        let Some(sym_name) = sym_name else {
            continue;
        };
        if !is_introspection_symbol(sym_name)
            || sym.st_shndx == SHN_UNDEF as usize
            || !seen_names.insert(sym_name)
        {
            continue;
        }
        let data_offset = elf_symbol_file_offset(elf, &sym).with_context(|| {
            format!("Failed to locate the introspection symbol {sym_name} in the ELF file")
        })?;
        chunks.push(deserialize_chunk(
            sym_name,
            library_content
                .get(data_offset..)
                .context("The introspection symbol is outside of the ELF file")?,
            elf.little_endian,
        )?);
    }
    Ok(chunks)
}

/// Maps the virtual address of a symbol to its offset in the file using the loadable segments.
///
/// We use program headers and not section headers because the latter are optional.
fn elf_symbol_file_offset(elf: &Elf<'_>, sym: &Sym) -> Result<usize> {
    let address = sym.st_value;
    let segment = elf
        .program_headers
        .iter()
        .find(|header| {
            header.p_type == PT_LOAD
                && header.p_vaddr <= address
                && address - header.p_vaddr < header.p_filesz
        })
        .with_context(|| {
            format!("No loadable segment with file content contains the address {address:#x}")
        })?;
    segment
        .p_offset
        .checked_add(address - segment.p_vaddr)
        .and_then(|offset| usize::try_from(offset).ok())
        .context("File offset overflow")
}

fn find_introspection_chunks_in_macho(
    macho: &MachO<'_>,
    library_content: &[u8],
) -> Result<Vec<Chunk>> {
    if !macho.little_endian {
        bail!("Only little endian Mach-o binaries are supported");
    }

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
            let section = sections
                .get(nlist.n_sect - 1) // Sections are counted from 1
                .context("The introspection symbol section does not exist")?;
            let data_offset = nlist
                .n_value
                .checked_sub(section.addr)
                .and_then(|offset| offset.checked_add(section.offset.into()))
                .context("The introspection symbol is outside of its section")?;
            chunks.push(deserialize_chunk(
                name,
                library_content
                    .get(usize::try_from(data_offset).context("File offset overflow")?..)
                    .context("The introspection symbol is outside of the Mach-O file")?,
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
                export.name.unwrap_or_default(),
                library_content
                    .get(export.offset.context("No symbol offset")?..)
                    .context("The introspection symbol is outside of the PE file")?,
                true,
            )?);
        }
    }
    Ok(chunks)
}

fn deserialize_chunk(
    symbol_name: &str,
    content_with_chunk_at_the_beginning: &[u8],
    is_little_endian: bool,
) -> Result<Chunk> {
    let symbol_version = introspection_symbol_version(symbol_name).unwrap_or_default();
    ensure!(
        symbol_version == "1",
        "The introspection format version '{symbol_version}' is not supported by this version of pyo3-introspection. Please upgrade your build tool like maturin or downgrade pyo3"
    );

    let length = content_with_chunk_at_the_beginning
        .get(..4)
        .and_then(|length| length.try_into().ok())
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
    introspection_symbol_version(name).is_some()
}

fn introspection_symbol_version(name: &str) -> Option<&str> {
    let name = name
        .strip_prefix('_')
        .unwrap_or(name)
        .strip_prefix("PYO3_INTROSPECTION_")?;
    Some(name.split_once('_').unwrap_or((name, "")).0)
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
        #[serde(default)]
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
        #[serde(default)]
        attach_to_root: bool,
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
