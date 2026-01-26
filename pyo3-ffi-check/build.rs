use std::env;
use std::path::PathBuf;

#[derive(Debug)]
struct ParseCallbacks;

impl bindgen::callbacks::ParseCallbacks for ParseCallbacks {
    // these are anonymous fields and structs in CPython that we needed to
    // invent names for. Bindgen seems to generate stable names, so we remap the
    // automatically generated names to the names we invented in the FFI
    fn item_name(&self, _original_item_name: &str) -> Option<String> {
        if _original_item_name == "_object__bindgen_ty_1__bindgen_ty_1" {
            Some("PyObjectObFlagsAndRefcnt".into())
        } else if _original_item_name == "_object__bindgen_ty_1" {
            Some("PyObjectObRefcnt".into())
        } else {
            None
        }
    }
}

fn main() {
    pyo3_build_config::add_libpython_rpath_link_args();

    let config = pyo3_build_config::get();

    let python_include_dir = config
        .run_python_script(
            "import sysconfig; print(sysconfig.get_config_var('INCLUDEPY'), end='');",
        )
        .expect("failed to get lib dir");
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

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(clang_args)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(ParseCallbacks))
        // blocklist some values which apparently have conflicting definitions on unix
        .blocklist_item("FP_NORMAL")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_INT_UPWARD")
        .blocklist_item("FP_INT_DOWNWARD")
        .blocklist_item("FP_INT_TOWARDZERO")
        .blocklist_item("FP_INT_TONEARESTFROMZERO")
        .blocklist_item("FP_INT_TONEAREST")
        .blocklist_item("FP_ZERO")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
