fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let target = std::env::var("TARGET").unwrap();

    let doc_dir = std::path::Path::new(&out_dir)
        .join(&target)
        .join("doc")
        .join("pyo3_ffi_check_definitions");

    // write docs into the build script output directory, they will be read
    // by the proc macro to resolve what definitions exist
    let status = std::process::Command::new("cargo")
        .args(["doc", "-p", "pyo3-ffi-check-definitions", "--no-deps"])
        .env("CARGO_TARGET_DIR", out_dir)
        // forward target to the doc buid to ensure `--target` is honored
        .env("CARGO_BUILD_TARGET", target)
        .status()
        .expect("failed to build definitions");

    // macro will use this env var to locate the docs
    println!(
        "cargo:rustc-env=PYO3_FFI_CHECK_DOC_DIR={}",
        doc_dir.to_str().unwrap()
    );

    status
        .success()
        .then_some(())
        .expect("failed to build definitions, see above for details");

    // Because `pyo3-ffi` is a dependency, libpython is linked, this ensures `main.rs` can run.
    // Slightly needless (no symbols from libpython are actually called), but simple to do.
    pyo3_build_config::add_libpython_rpath_link_args();
}
