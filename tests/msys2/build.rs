use std::{env, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=py.c");
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=PYO3_PYTHON");

    let python = env::var("PYO3_PYTHON").unwrap_or_else(|_| "python".to_owned());
    let include_dir = String::from_utf8(
        Command::new(&python)
            .args(["-c", "import sysconfig; print(sysconfig.get_path('include'))"])
            .output()
            .unwrap_or_else(|err| panic!("failed to run {python}: {err}"))
            .stdout,
    )
    .expect("python include path is not valid UTF-8")
    .trim()
    .to_owned();

    let obj_file = env::temp_dir().join("pyo3-msys2.o");
    let output = Command::new(env::var("CC").unwrap_or_else(|_| "gcc".to_owned()))
        .args([
            format!("-I{include_dir}"),
            "-c".to_owned(),
            "py.c".to_owned(),
            "-o".to_owned(),
            obj_file.display().to_string(),
        ])
        .output()
        .unwrap_or_else(|err| panic!("failed to run gcc: {err}"));

    assert!(
        output.status.success(),
        "gcc failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    println!("cargo:rustc-link-arg={}", obj_file.display());
}
