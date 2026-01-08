use crate::model::{
    Argument, Arguments, Attribute, Class, Function, Module, PythonIdentifier, TypeHint,
    TypeHintExpr, VariableLengthArgument,
};
use anyhow::{anyhow, bail, ensure, Context, Result};
use goblin::elf::section_header::SHN_XINDEX;
use goblin::elf::Elf;
use goblin::mach::load_command::CommandVariant;
use goblin::mach::symbols::{NO_SECT, N_SECT};
use goblin::mach::{Mach, MachO, SingleArch};
use goblin::pe::PE;
use goblin::Object;
use serde::de::value::MapAccessDeserializer;
use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;
use std::{fmt, fs, str};

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
            Chunk::Module { id, .. } | Chunk::Class { id, .. } => (Some(id.as_str()), None),
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
            incomplete,
        } = chunk
        {
            if name == main_module_name {
                return convert_module(
                    id,
                    name,
                    members,
                    *incomplete,
                    &chunks_by_id,
                    &chunks_by_parent,
                );
            }
        }
    }
    bail!("No module named {main_module_name} found")
}

fn convert_module(
    id: &str,
    name: &str,
    members: &[String],
    mut incomplete: bool,
    chunks_by_id: &HashMap<&str, &Chunk>,
    chunks_by_parent: &HashMap<&str, Vec<&Chunk>>,
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
    let (modules, classes, functions, attributes) =
        convert_members(member_chunks, chunks_by_id, chunks_by_parent)?;

    Ok(Module {
        name: name.into(),
        modules,
        classes,
        functions,
        attributes,
        incomplete,
    })
}

type Members = (Vec<Module>, Vec<Class>, Vec<Function>, Vec<Attribute>);

/// Convert a list of members of a module or a class
fn convert_members<'a>(
    chunks: impl IntoIterator<Item = &'a Chunk>,
    chunks_by_id: &HashMap<&str, &Chunk>,
    chunks_by_parent: &HashMap<&str, Vec<&Chunk>>,
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
            } => {
                modules.push(convert_module(
                    id,
                    name,
                    members,
                    *incomplete,
                    chunks_by_id,
                    chunks_by_parent,
                )?);
            }
            Chunk::Class {
                name,
                id,
                bases,
                decorators,
            } => classes.push(convert_class(
                id,
                name,
                bases,
                decorators,
                chunks_by_id,
                chunks_by_parent,
            )?),
            Chunk::Function {
                name,
                id: _,
                arguments,
                parent: _,
                decorators,
                returns,
            } => functions.push(convert_function(name, arguments, decorators, returns)?),
            Chunk::Attribute {
                name,
                id: _,
                parent: _,
                value,
                annotation,
            } => attributes.push(convert_attribute(name, value, annotation)),
        }
    }
    // We sort elements to get a stable output
    modules.sort_by(|l, r| l.name.cmp(&r.name));
    classes.sort_by(|l, r| l.name.cmp(&r.name));
    functions.sort_by(|l, r| match l.name.cmp(&r.name) {
        Ordering::Equal => {
            // We put the getter before the setter. For that, we put @property before the other ones
            if l.decorators
                .iter()
                .any(|d| d.name == "property" && d.module.as_deref() == Some("builtins"))
            {
                Ordering::Less
            } else if r
                .decorators
                .iter()
                .any(|d| d.name == "property" && d.module.as_deref() == Some("builtins"))
            {
                Ordering::Greater
            } else {
                // We pick an ordering based on decorators
                l.decorators
                    .iter()
                    .map(|d| &d.name)
                    .cmp(r.decorators.iter().map(|d| &d.name))
            }
        }
        o => o,
    });
    attributes.sort_by(|l, r| l.name.cmp(&r.name));
    Ok((modules, classes, functions, attributes))
}

fn convert_class(
    id: &str,
    name: &str,
    bases: &[ChunkTypeHint],
    decorators: &[ChunkTypeHint],
    chunks_by_id: &HashMap<&str, &Chunk>,
    chunks_by_parent: &HashMap<&str, Vec<&Chunk>>,
) -> Result<Class> {
    let (nested_modules, nested_classes, methods, attributes) = convert_members(
        chunks_by_parent.get(&id).into_iter().flatten().copied(),
        chunks_by_id,
        chunks_by_parent,
    )?;
    ensure!(
        nested_modules.is_empty(),
        "Classes cannot contain nested modules"
    );
    ensure!(
        nested_classes.is_empty(),
        "Nested classes are not supported yet"
    );
    Ok(Class {
        name: name.into(),
        bases: bases
            .iter()
            .map(convert_python_identifier)
            .collect::<Result<_>>()?,
        methods,
        attributes,
        decorators: decorators
            .iter()
            .map(convert_python_identifier)
            .collect::<Result<_>>()?,
    })
}

fn convert_python_identifier(decorator: &ChunkTypeHint) -> Result<PythonIdentifier> {
    match convert_type_hint(decorator) {
        TypeHint::Plain(id) => Ok(PythonIdentifier {
            module: None,
            name: id.clone(),
        }),
        TypeHint::Ast(expr) => {
            if let TypeHintExpr::Identifier(i) = expr {
                Ok(i)
            } else {
                bail!("PyO3 introspection currently only support decorators that are identifiers of a Python function, got {expr:?}")
            }
        }
    }
}

fn convert_function(
    name: &str,
    arguments: &ChunkArguments,
    decorators: &[ChunkTypeHint],
    returns: &Option<ChunkTypeHint>,
) -> Result<Function> {
    Ok(Function {
        name: name.into(),
        decorators: decorators
            .iter()
            .map(convert_python_identifier)
            .collect::<Result<_>>()?,
        arguments: Arguments {
            positional_only_arguments: arguments.posonlyargs.iter().map(convert_argument).collect(),
            arguments: arguments.args.iter().map(convert_argument).collect(),
            vararg: arguments
                .vararg
                .as_ref()
                .map(convert_variable_length_argument),
            keyword_only_arguments: arguments.kwonlyargs.iter().map(convert_argument).collect(),
            kwarg: arguments
                .kwarg
                .as_ref()
                .map(convert_variable_length_argument),
        },
        returns: returns.as_ref().map(convert_type_hint),
    })
}

fn convert_argument(arg: &ChunkArgument) -> Argument {
    Argument {
        name: arg.name.clone(),
        default_value: arg.default.clone(),
        annotation: arg.annotation.as_ref().map(convert_type_hint),
    }
}

fn convert_variable_length_argument(arg: &ChunkArgument) -> VariableLengthArgument {
    VariableLengthArgument {
        name: arg.name.clone(),
        annotation: arg.annotation.as_ref().map(convert_type_hint),
    }
}

fn convert_attribute(
    name: &str,
    value: &Option<String>,
    annotation: &Option<ChunkTypeHint>,
) -> Attribute {
    Attribute {
        name: name.into(),
        value: value.clone(),
        annotation: annotation.as_ref().map(convert_type_hint),
    }
}

fn convert_type_hint(arg: &ChunkTypeHint) -> TypeHint {
    match arg {
        ChunkTypeHint::Ast(expr) => TypeHint::Ast(convert_type_hint_expr(expr)),
        ChunkTypeHint::Plain(t) => TypeHint::Plain(t.clone()),
    }
}

fn convert_type_hint_expr(expr: &ChunkTypeHintExpr) -> TypeHintExpr {
    match expr {
        ChunkTypeHintExpr::Local { id } => PythonIdentifier {
            module: None,
            name: id.clone(),
        }
        .into(),
        ChunkTypeHintExpr::Builtin { id } => PythonIdentifier {
            module: Some("builtins".into()),
            name: id.clone(),
        }
        .into(),
        ChunkTypeHintExpr::Attribute { module, attr } => PythonIdentifier {
            module: Some(module.clone()),
            name: attr.clone(),
        }
        .into(),
        ChunkTypeHintExpr::Union { elts } => {
            TypeHintExpr::Union(elts.iter().map(convert_type_hint_expr).collect())
        }
        ChunkTypeHintExpr::Subscript { value, slice } => TypeHintExpr::Subscript {
            value: Box::new(convert_type_hint_expr(value)),
            slice: slice.iter().map(convert_type_hint_expr).collect(),
        },
    }
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
            "Failed to parse introspection chunk: '{}'",
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
        incomplete: bool,
    },
    Class {
        id: String,
        name: String,
        #[serde(default)]
        bases: Vec<ChunkTypeHint>,
        #[serde(default)]
        decorators: Vec<ChunkTypeHint>,
    },
    Function {
        #[serde(default)]
        id: Option<String>,
        name: String,
        arguments: Box<ChunkArguments>,
        #[serde(default)]
        parent: Option<String>,
        #[serde(default)]
        decorators: Vec<ChunkTypeHint>,
        #[serde(default)]
        returns: Option<ChunkTypeHint>,
    },
    Attribute {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        parent: Option<String>,
        name: String,
        #[serde(default)]
        value: Option<String>,
        #[serde(default)]
        annotation: Option<ChunkTypeHint>,
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
    default: Option<String>,
    #[serde(default)]
    annotation: Option<ChunkTypeHint>,
}

/// Variant of [`TypeHint`] that implements deserialization.
///
/// We keep separated type to allow them to evolve independently (this type will need to handle backward compatibility).
enum ChunkTypeHint {
    Ast(ChunkTypeHintExpr),
    Plain(String),
}

impl<'de> Deserialize<'de> for ChunkTypeHint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AnnotationVisitor;

        impl<'de> Visitor<'de> for AnnotationVisitor {
            type Value = ChunkTypeHint;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("annotation")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_string(v.into())
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(ChunkTypeHint::Plain(v))
            }

            fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<ChunkTypeHint, M::Error> {
                Ok(ChunkTypeHint::Ast(Deserialize::deserialize(
                    MapAccessDeserializer::new(map),
                )?))
            }
        }

        deserializer.deserialize_any(AnnotationVisitor)
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ChunkTypeHintExpr {
    Local {
        id: String,
    },
    Builtin {
        id: String,
    },
    Attribute {
        module: String,
        attr: String,
    },
    Union {
        elts: Vec<ChunkTypeHintExpr>,
    },
    Subscript {
        value: Box<ChunkTypeHintExpr>,
        slice: Vec<ChunkTypeHintExpr>,
    },
}
