use std::env;
use std::path::PathBuf;

use bindgen::callbacks::{ItemInfo, ItemKind};
use target_lexicon::{Architecture, OperatingSystem, Triple};

#[derive(Debug)]
struct ParseCallbacks;

impl bindgen::callbacks::ParseCallbacks for ParseCallbacks {
    // these are anonymous fields and structs in CPython that we needed to
    // invent names for. Bindgen seems to generate stable names, so we remap the
    // automatically generated names to the names we invented in the FFI
    fn item_name(&self, item_info: ItemInfo<'_>) -> Option<String> {
        match item_info.name {
            "_object__bindgen_ty_1__bindgen_ty_1" => Some("PyObjectObFlagsAndRefcnt".into()),
            "_object__bindgen_ty_1" => Some("PyObjectObRefcnt".into()),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct WindowsX86RawDylibCallbacks;

// Matches the adjustment in `pyo3-ffi` to force the link name for functions starting
// with `_Py` (see `pyo3-ffi/src/impl_/macros.rs`)
impl bindgen::callbacks::ParseCallbacks for WindowsX86RawDylibCallbacks {
    fn generated_link_name_override(&self, item: ItemInfo<'_>) -> Option<String> {
        if item.kind == ItemKind::Function && item.name.starts_with("_Py") {
            Some(format!("_{}", item.name))
        } else {
            None
        }
    }
}

fn main() {
    let config = pyo3_build_config::get();
    let target: Triple = env::var("TARGET").unwrap().parse().unwrap();

    let python_include_dir = config
        .run_python_script(
            "import sysconfig; print(sysconfig.get_config_var('INCLUDEPY'), end='');",
        )
        .expect("failed to get include dir");
    let gil_disabled_on_windows = config
        .run_python_script(
            "import sysconfig; import platform; print(sysconfig.get_config_var('Py_GIL_DISABLED') == 1 and platform.system() == 'Windows');",
        )
        .expect("failed to get Py_GIL_DISABLED").trim_end() == "True";

    let clang_args = if gil_disabled_on_windows {
        vec![
            format!("-I{python_include_dir}"),
            "-DPy_GIL_DISABLED".to_string(),
        ]
    } else {
        vec![format!("-I{python_include_dir}")]
    };

    println!("cargo:rerun-if-changed=wrapper.h");

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(clang_args)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(ParseCallbacks))
        // Minimising bindgen output to `Py` symbols and their dependencies, avoiding
        // system declarations etc which are not relevant to `pyo3-ffi-check`.
        .allowlist_type("_?Py.*")
        .allowlist_function("_?Py.*")
        .allowlist_var("_?Py.*|PY.*");

    // Match PyO3's choice to use raw-dylib linking on Windows for the bindgen symbols
    // so that link resolution is done identically
    if target.operating_system == OperatingSystem::Windows {
        println!("cargo:rerun-if-env-changed=PYO3_USE_RAW_DYLIB");
        let lib_name = config.lib_name().expect("missing Python library name");
        if env::var("PYO3_USE_RAW_DYLIB").map_or(true, |value| value == "1") {
            let import_name_type = if matches!(target.architecture, Architecture::X86_32(_)) {
                builder = builder.parse_callbacks(Box::new(WindowsX86RawDylibCallbacks));
                ", import_name_type = \"undecorated\""
            } else {
                ""
            };
            builder = builder.extern_block_attrs(format!(
                "#[link(name = \"{lib_name}\", kind = \"raw-dylib\"{import_name_type})]"
            ));
        }
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
