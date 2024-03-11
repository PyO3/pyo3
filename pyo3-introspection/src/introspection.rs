use crate::model::{Class, Function, Module};
use anyhow::{bail, Context, Result};
use goblin::elf::Elf;
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
        Object::Mach(Mach::Binary(matcho)) => {
            find_introspection_chunks_in_matcho(&matcho, &library_content)
        }
        Object::Mach(Mach::Fat(multi_arch)) => {
            for arch in &multi_arch {
                match arch? {
                    SingleArch::MachO(matcho) => {
                        return find_introspection_chunks_in_matcho(&matcho, &library_content)
                    }
                    SingleArch::Archive(_) => (),
                }
            }
            bail!("No Match-o chunk found in the multi-arch Match-o container")
        }
        Object::PE(pe) => find_introspection_chunks_in_pe(&pe, &library_content),
        _ => {
            bail!("Only ELF, Match-o and PE containers can be introspected")
        }
    }
}

fn find_introspection_chunks_in_elf(elf: &Elf<'_>, library_content: &[u8]) -> Result<Vec<Chunk>> {
    let pyo3_data_section_header = elf
        .section_headers
        .iter()
        .find(|section| elf.shdr_strtab.get_at(section.sh_name).unwrap_or_default() == ".pyo3i0")
        .context("No .pyo3i0 section found")?;
    let sh_offset =
        usize::try_from(pyo3_data_section_header.sh_offset).context("Section offset overflow")?;
    let sh_size =
        usize::try_from(pyo3_data_section_header.sh_size).context("Section len overflow")?;
    if elf.is_64 {
        read_section_with_ptr_and_len_64bits(
            &library_content[sh_offset..sh_offset + sh_size],
            0,
            library_content,
        )
    } else {
        read_section_with_ptr_and_len_32bits(
            &library_content[sh_offset..sh_offset + sh_size],
            0,
            library_content,
        )
    }
}

fn find_introspection_chunks_in_matcho(
    matcho: &MachO<'_>,
    library_content: &[u8],
) -> Result<Vec<Chunk>> {
    if !matcho.little_endian {
        bail!("Only little endian Match-o binaries are supported");
    }
    let text_segment = matcho
        .segments
        .iter()
        .find(|s| s.segname == *b"__TEXT\0\0\0\0\0\0\0\0\0\0")
        .context("No __TEXT segment found")?;
    let (_, pyo3_data_section) = text_segment
        .sections()?
        .into_iter()
        .find(|s| s.0.sectname == *b"__pyo3i0\0\0\0\0\0\0\0\0")
        .context("No __pyo3i0 section found")?;
    if matcho.is_64 {
        read_section_with_ptr_and_len_64bits(pyo3_data_section, 0, library_content)
    } else {
        read_section_with_ptr_and_len_32bits(pyo3_data_section, 0, library_content)
    }
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
    let pyo3_data_section = pe
        .sections
        .iter()
        .find(|section| section.name().unwrap_or_default() == ".pyo3i0")
        .context("No .pyo3i0 section found")?;
    let pyo3_data = pyo3_data_section
        .data(library_content)?
        .context("Not able to find the .pyo3i0 section content")?;
    if pe.is_64 {
        read_section_with_ptr_and_len_64bits(&pyo3_data, rdata_shift, library_content)
    } else {
        read_section_with_ptr_and_len_32bits(&pyo3_data, rdata_shift, library_content)
    }
}

fn read_section_with_ptr_and_len_32bits(
    slice: &[u8],
    shift: usize,
    full_library_content: &[u8],
) -> Result<Vec<Chunk>> {
    slice
        .chunks_exact(8)
        .filter_map(|element| {
            let (ptr, len) = element.split_at(4);
            let ptr = match usize::try_from(u32::from_le_bytes(ptr.try_into().unwrap())) {
                Ok(ptr) => ptr,
                Err(e) => return Some(Err(e).context("Pointer overflow")),
            };
            let len = match usize::try_from(u32::from_le_bytes(len.try_into().unwrap())) {
                Ok(ptr) => ptr,
                Err(e) => return Some(Err(e).context("Length overflow")),
            };
            if ptr == 0 || len == 0 {
                // Workaround for PE
                return None;
            }
            Some(
                serde_json::from_slice(&full_library_content[ptr - shift..ptr - shift + len])
                    .context("Failed to parse introspection chunk"),
            )
        })
        .collect()
}

fn read_section_with_ptr_and_len_64bits(
    slice: &[u8],
    shift: usize,
    full_library_content: &[u8],
) -> Result<Vec<Chunk>> {
    slice
        .chunks_exact(16)
        .filter_map(|element| {
            let (ptr, len) = element.split_at(8);
            let ptr = match usize::try_from(u64::from_le_bytes(ptr.try_into().unwrap())) {
                Ok(ptr) => ptr,
                Err(e) => return Some(Err(e).context("Pointer overflow")),
            };
            let len = match usize::try_from(u64::from_le_bytes(len.try_into().unwrap())) {
                Ok(ptr) => ptr,
                Err(e) => return Some(Err(e).context("Length overflow")),
            };
            if ptr == 0 || len == 0 {
                // Workaround for PE
                return None;
            }
            Some(
                serde_json::from_slice(&full_library_content[ptr - shift..ptr - shift + len])
                    .context("Failed to parse introspection chunk"),
            )
        })
        .collect()
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
