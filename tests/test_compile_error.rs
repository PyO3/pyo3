#![cfg(feature = "macros")]

#[cfg(not(target_arch = "wasm32"))] // Not possible to invoke compiler from wasm
#[test]
fn test_compile_errors() {
    use std::path::PathBuf;

    use ui_test::{run_tests, Config};

    let mut config = Config::rustc("tests/ui");

    let deps_features = [
        #[cfg(feature = "macros")]
        "pyo3/macros".to_string(),
        #[cfg(feature = "abi3")]
        "pyo3/abi3".to_string(),
        #[cfg(feature = "abi3-py37")]
        "pyo3/abi3-py37".to_string(),
        #[cfg(feature = "abi3-py38")]
        "pyo3/abi3-py38".to_string(),
        #[cfg(feature = "abi3-py39")]
        "pyo3/abi3-py39".to_string(),
        #[cfg(feature = "abi3-py310")]
        "pyo3/abi3-py310".to_string(),
        #[cfg(feature = "abi3-py311")]
        "pyo3/abi3-py311".to_string(),
        #[cfg(feature = "abi3-py312")]
        "pyo3/abi3-py312".to_string(),
        #[cfg(feature = "abi3-py313")]
        "pyo3/abi3-py313".to_string(),
        #[cfg(feature = "abi3-py314")]
        "pyo3/abi3-py314".to_string(),
    ];

    let mut deps_cargo = ui_test::CommandBuilder::cargo();
    deps_cargo.args.push("--features".into());
    deps_cargo.args.push(deps_features.join(",").into());

    config.comment_defaults.base().set_custom(
        "dependencies",
        ui_test::dependencies::DependencyBuilder {
            crate_manifest_path: PathBuf::from(
                env!("CARGO_MANIFEST_DIR").to_owned() + "/tests/ui/base/Cargo.toml",
            ),
            program: deps_cargo,
            ..Default::default()
        },
    );

    if let Ok(target) = std::env::var("CARGO_BUILD_TARGET") {
        config.target = Some(target);
    }

    config
        .comment_defaults
        .base()
        .compile_flags
        .push("--diagnostic-width=140".into());

    config.skip_files.extend([
        // not a test file, used to configure dependencies for the tests
        "base/src/lib.rs".into(),
        // abi3-only tests only need to check when the feature is unsupported
        #[cfg(any(not(Py_LIMITED_API), Py_3_9))]
        "abi3_dict".into(),
        #[cfg(any(not(Py_LIMITED_API), Py_3_9))]
        "abi3_weakref".into(),
        #[cfg(any(not(Py_LIMITED_API), Py_3_12))]
        "abi3_nativetype_inheritance".into(),
        #[cfg(any(not(Py_LIMITED_API), Py_3_12))]
        "abi3_inheritance".into(),
        // this test doesn't work properly without the full API available
        #[cfg(Py_LIMITED_API)]
        "forbid_unsafe.rs".into(),
        // buffer protocol only supported on 3.11+ with abi3
        #[cfg(all(Py_LIMITED_API, not(Py_3_11)))]
        "buffer".into(),
        // only needs to run on versions where `#[pyclass(immutable_type)]` is unsupported
        #[cfg(any(Py_3_14, all(Py_3_10, not(Py_LIMITED_API))))]
        "immutable_type.rs".into(),
        // generic pyclasses only supported on 3.9+, doesn't fail gracefully on older versions
        #[cfg(not(Py_3_9))]
        "invalid_pyclass_generic.rs".into(),
    ]);

    config.output_conflict_handling = ui_test::bless_output_files;

    #[cfg(not(target_arch = "wasm32"))] // doesn't work on wasm
    {
        let abort_check = config.abort_check.clone();
        ctrlc::set_handler(move || abort_check.abort()).unwrap();
    }

    run_tests(config).unwrap();
}
