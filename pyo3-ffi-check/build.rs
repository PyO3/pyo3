use std::env;
use std::path::PathBuf;

fn main() {
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
