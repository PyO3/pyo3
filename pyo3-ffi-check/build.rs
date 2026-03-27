fn main() {
    let status = std::process::Command::new("cargo")
        .args(["doc", "-p", "pyo3-ffi-check-definitions", "--no-deps"])
        .env("CARGO_TARGET_DIR", std::env::var("OUT_DIR").unwrap())
        .env("CARGO_BUILD_TARGET", std::env::var("TARGET").unwrap())
        .status()
        .expect("failed to build definitions");

    status
        .success()
        .then_some(())
        .expect("failed to build definitions, see above for details");
}
