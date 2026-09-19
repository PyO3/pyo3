use std::env;
use std::ffi::OsString;

fn main() {
    configure_wip_no_std();
    pyo3_build_config::use_pyo3_cfgs();
    pyo3_build_config::add_extension_module_link_args();
}

/// Enables a faux `std` feature by default.
///
/// Set env var `PYO3_WIP_NO_STD` to `1` to disable it.
// Has a matching function in pyo3's build.rs
fn configure_wip_no_std() {
    println!("cargo:rustc-check-cfg=cfg(wip_feature_std)");
    match env_var("PYO3_WIP_NO_STD").map(|s| s.into_string().unwrap()) {
        Some(no_std) if no_std.trim() == "1" || no_std.trim().eq_ignore_ascii_case("true") => (),
        _ => println!("cargo:rustc-cfg=wip_feature_std"),
    }
}

/// Gets an external environment variable, and registers the build script to rerun if
/// the variable changes.
fn env_var(var: &str) -> Option<OsString> {
    println!("cargo:rerun-if-env-changed={var}");
    env::var_os(var)
}
