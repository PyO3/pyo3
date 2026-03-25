//! Small CLI entry point to introspect a Python cdylib built using PyO3 and generate [type stubs](https://typing.readthedocs.io/en/latest/source/stubs.html).

use anyhow::{anyhow, Context, Result};
use pyo3_introspection::{introspect_cdylib, module_stub_files};
use std::path::Path;
use std::{env, fs};

fn main() -> Result<()> {
    let [_, binary_path, module_name, output_path] = env::args().collect::<Vec<_>>().try_into().map_err(|_| anyhow!("pyo3-introspection takes three arguments, the path of the binary to introspect, the name of the python module to introspect and and the path of the directory to write the stub to"))?;
    let module = introspect_cdylib(&binary_path, &module_name)
        .with_context(|| format!("Failed to introspect module {binary_path}"))?;
    let actual_stubs = module_stub_files(&module);
    for (path, module) in actual_stubs {
        let file_path = Path::new(&output_path).join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory {}", file_path.display())
            })?;
        }
        fs::write(&file_path, module)
            .with_context(|| format!("Failed to write module {}", file_path.display()))?;
    }
    Ok(())
}
