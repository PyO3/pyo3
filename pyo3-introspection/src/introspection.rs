use crate::model::{Class, Function, Module};
use anyhow::{bail, Context, Result};
use goblin::mach::Mach;
use goblin::Object;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Introspect a cdylib built with PyO3 and returns the definition of a Python module
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
    let mut chunks = Vec::new();
    match Object::parse(&library_content)
        .context("The built library is not valid or not supported by our binary parser")?
    {
        Object::Mach(Mach::Binary(matcho)) => {
            if !matcho.is_64 {
                bail!("Only 64 bits binaries are supported");
            }
            if !matcho.little_endian {
                bail!("Only little endian binaries are supported");
            }
            let Some(text_segment) = matcho
                .segments
                .iter()
                .find(|s| s.segname == *b"__TEXT\0\0\0\0\0\0\0\0\0\0")
            else {
                bail!("No __TEXT segment found");
            };
            for (sec, sec_content) in text_segment.sections()? {
                println!(
                    "{} {}",
                    String::from_utf8_lossy(&sec.sectname),
                    sec_content.len()
                );
            }
            let Some((_, pyo3_data_section)) = text_segment
                .sections()?
                .into_iter()
                .find(|s| s.0.sectname == *b"__pyo3_data0\0\0\0\0")
            else {
                bail!("No __pyo3_data0 section found");
            };
            for element in pyo3_data_section.chunks(16) {
                let ptr = usize::from_le_bytes(element[..8].try_into().unwrap());
                let len = usize::from_le_bytes(element[8..].try_into().unwrap());
                chunks.push(serde_json::from_slice(&library_content[ptr..ptr + len])?);
            }
        }
        _ => bail!("Only Match-O files can be introspected"),
    };
    Ok(chunks)
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
