use std::{env, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=py.c");
    println!("cargo:rerun-if-env-changed=PYO3_PYTHON");

    let python = env::var("PYO3_PYTHON").unwrap_or_else(|_| "python".to_owned());
    let output = Command::new(&python)
        .args(["-c", "import sysconfig; print(sysconfig.get_path('include'))"])
        .output()
        .expect("failed to query Python include directory");

    if !output.status.success() {
        panic!("python failed to report include directory");
    }

    let include_dir = String::from_utf8(output.stdout)
        .expect("python returned non-UTF-8 include directory");

    cc::Build::new()
        .file("py.c")
        .include(include_dir.trim())
        .compile("pyo3-msys2");
}
