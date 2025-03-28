use crate::model::{Class, Function, Module};
use anyhow::{bail, Context, Result};
use goblin::elf::Elf;
use goblin::mach::symbols::N_SECT;
use goblin::mach::{Mach, MachO, SingleArch};
use goblin::pe::PE;
use goblin::Object;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Introspect a cdylib built with PyO3 and returns the definition of a Python module.
///
/// This function currently supports the ELF (most *nix including Linux), Match-O (macOS) and PE (Windows) formats.
pub fn introspect_cdylib(library_path: impl AsRef<Path>, main_module_name: &str) -> Result<Module> {
    let chunks = find_introspection_chunks_in_binary_object(library_path.as_ref())?;
    parse_chunks(&chunks, main_module_name)
}

/// Parses the introspection chunks found in the binary
fn parse_chunks(chunks: &[Chunk], main_module_name: &str) -> Result<Module> {
    let chunks_by_id = chunks
        .iter()
        .map(|c| {
            (
                match c {
                    Chunk::Module { id, .. } => id,
                    Chunk::Class { id, .. } => id,
                    Chunk::Function { id, .. } => id,
                },
                c,
            )
        })
        .collect::<HashMap<_, _>>();
    // We look for the root chunk
    for chunk in chunks {
        if let Chunk::Module {
            name,
            members,
            id: _,
        } = chunk
        {
            if name == main_module_name {
                return parse_module(name, members, &chunks_by_id);
            }
        }
    }
    bail!("No module named {main_module_name} found")
}

fn parse_module(
    name: &str,
    members: &[String],
    chunks_by_id: &HashMap<&String, &Chunk>,
) -> Result<Module> {
    let mut modules = Vec::new();
    let mut classes = Vec::new();
    let mut functions = Vec::new();
    for member in members {
        if let Some(chunk) = chunks_by_id.get(member) {
            match chunk {
                Chunk::Module {
                    name,
                    members,
                    id: _,
                } => {
                    modules.push(parse_module(name, members, chunks_by_id)?);
                }
                Chunk::Class { name, id: _ } => classes.push(Class { name: name.into() }),
                Chunk::Function { name, id: _ } => functions.push(Function { name: name.into() }),
            }
        }
    }
    Ok(Module {
        name: name.into(),
        modules,
        classes,
        functions,
    })
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
            let section_header = &elf.section_headers[sym.st_shndx];
            let data_offset = sym.st_value + section_header.sh_offset - section_header.sh_addr;
            chunks.push(read_symbol_value_with_ptr_and_len(
                &library_content[usize::try_from(data_offset).context("File offset overflow")?..],
                0,
                library_content,
                elf.is_64,
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

    let sections = macho
        .segments
        .sections()
        .flatten()
        .map(|t| t.map(|s| s.0))
        .collect::<Result<Vec<_>, _>>()?;
    let mut chunks = Vec::new();
    for (name, nlist) in macho.symbols().flatten() {
        if nlist.is_global() && nlist.get_type() == N_SECT && is_introspection_symbol(name) {
            let section = &sections[nlist.n_sect];
            let data_offset = nlist.n_value + u64::from(section.offset) - section.addr;
            chunks.push(read_symbol_value_with_ptr_and_len(
                &library_content[usize::try_from(data_offset).context("File offset overflow")?..],
                0,
                library_content,
                macho.is_64,
            )?);
        }
    }
    Ok(chunks)
}

fn find_introspection_chunks_in_pe(pe: &PE<'_>, library_content: &[u8]) -> Result<Vec<Chunk>> {
    let rdata_data_section = pe
        .sections
        .iter()
        .find(|section| section.name().unwrap_or_default() == ".rdata")
        .context("No .rdata section found")?;
    let rdata_shift = pe.image_base
        + usize::try_from(rdata_data_section.virtual_address)
            .context(".rdata virtual_address overflow")?
        - usize::try_from(rdata_data_section.pointer_to_raw_data)
            .context(".rdata pointer_to_raw_data overflow")?;

    let mut chunks = Vec::new();
    for export in &pe.exports {
        if is_introspection_symbol(export.name.unwrap_or_default()) {
            chunks.push(read_symbol_value_with_ptr_and_len(
                &library_content[export.offset.context("No symbol offset")?..],
                rdata_shift,
                library_content,
                pe.is_64,
            )?);
        }
    }
    Ok(chunks)
}

fn read_symbol_value_with_ptr_and_len(
    value_slice: &[u8],
    shift: usize,
    full_library_content: &[u8],
    is_64: bool,
) -> Result<Chunk> {
    let (ptr, len) = if is_64 {
        let (ptr, len) = value_slice[..16].split_at(8);
        let ptr = usize::try_from(u64::from_le_bytes(
            ptr.try_into().context("Too short symbol value")?,
        ))
        .context("Pointer overflow")?;
        let len = usize::try_from(u64::from_le_bytes(
            len.try_into().context("Too short symbol value")?,
        ))
        .context("Length overflow")?;
        (ptr, len)
    } else {
        let (ptr, len) = value_slice[..8].split_at(4);
        let ptr = usize::try_from(u32::from_le_bytes(
            ptr.try_into().context("Too short symbol value")?,
        ))
        .context("Pointer overflow")?;
        let len = usize::try_from(u32::from_le_bytes(
            len.try_into().context("Too short symbol value")?,
        ))
        .context("Length overflow")?;
        (ptr, len)
    };
    let chunk = &full_library_content[ptr - shift..ptr - shift + len];
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
        .starts_with("PYO3_INTROSPECTION_0_")
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Chunk {
    Module {
        id: String,
        name: String,
        members: Vec<String>,
    },
    Class {
        id: String,
        name: String,
    },
    Function {
        id: String,
        name: String,
    },
}
