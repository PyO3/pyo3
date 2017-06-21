extern crate compiletest_rs as compiletest;

use std::path::PathBuf;
use std::env::var;

fn run_mode(mode: &'static str) {
    let mut config = compiletest::default_config();
    let cfg_mode = mode.parse().ok().expect("Invalid mode");

    config.target_rustcflags = Some("-L target/debug/deps/".to_string());
    config.mode = cfg_mode;
    if let Ok(name) = var::<&str>("TESTNAME") {
        let s : String = name.to_string();
        config.filter = Some(s)
    }
    config.src_base = PathBuf::from(format!("tests/{}", mode));

    compiletest::run_tests(&config);
}

#[test]
fn compile_tests() {
    // run_mode("compile-fail");
    // run_mode("run-pass");
}
