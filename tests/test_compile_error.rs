#![cfg(feature = "macros")]

#[cfg(not(target_arch = "wasm32"))] // Not possible to invoke compiler from wasm
#[test]
fn test_compile_errors() {
    // let t = trybuild::TestCases::new();

    // t.compile_fail("tests/ui/deprecated_pyfn.rs");
    // #[cfg(not(feature = "experimental-inspect"))]
    // t.compile_fail("tests/ui/invalid_property_args.rs");
    // t.compile_fail("tests/ui/invalid_proto_pymethods.rs");
    // #[cfg(not(all(Py_LIMITED_API, not(Py_3_10))))] // to avoid PyFunctionArgument for &str
    // t.compile_fail("tests/ui/invalid_pyclass_args.rs");
    // t.compile_fail("tests/ui/invalid_pyclass_doc.rs");
    // t.compile_fail("tests/ui/invalid_pyclass_enum.rs");
    // t.compile_fail("tests/ui/invalid_pyclass_init.rs");
    // t.compile_fail("tests/ui/invalid_pyclass_item.rs");
    // #[cfg(Py_3_9)]
    // t.compile_fail("tests/ui/invalid_pyclass_generic.rs");
    // #[cfg(Py_3_9)]
    // t.compile_fail("tests/ui/pyclass_generic_enum.rs");
    // #[cfg(not(feature = "experimental-inspect"))]
    // #[cfg(not(all(Py_LIMITED_API, not(Py_3_10))))] // to avoid PyFunctionArgument for &str
    // t.compile_fail("tests/ui/invalid_pyfunction_argument.rs");
    // t.compile_fail("tests/ui/invalid_pyfunction_definition.rs");
    // t.compile_fail("tests/ui/invalid_pyfunction_signatures.rs");
    // #[cfg(any(not(Py_LIMITED_API), Py_3_11))]
    // t.compile_fail("tests/ui/invalid_pymethods_buffer.rs");
    // // The output is not stable across abi3 / not abi3 and features
    // #[cfg(all(not(Py_LIMITED_API), feature = "full"))]
    // t.compile_fail("tests/ui/invalid_pymethods_duplicates.rs");
    // t.compile_fail("tests/ui/invalid_pymethod_enum.rs");
    // t.compile_fail("tests/ui/invalid_pymethod_names.rs");
    // t.compile_fail("tests/ui/invalid_pymodule_args.rs");
    // t.compile_fail("tests/ui/invalid_pycallargs.rs");
    // t.compile_fail("tests/ui/reject_generics.rs");
    // t.compile_fail("tests/ui/invalid_closure.rs");
    // t.compile_fail("tests/ui/pyclass_send.rs");
    // #[cfg(not(feature = "experimental-inspect"))]
    // t.compile_fail("tests/ui/invalid_annotation.rs");
    // #[cfg(not(feature = "experimental-inspect"))]
    // t.compile_fail("tests/ui/invalid_annotation_return.rs");
    // t.compile_fail("tests/ui/invalid_argument_attributes.rs");
    // t.compile_fail("tests/ui/invalid_intopy_derive.rs");
    // #[cfg(not(windows))]
    // t.compile_fail("tests/ui/invalid_intopy_with.rs");
    // t.compile_fail("tests/ui/invalid_frompy_derive.rs");
    // t.compile_fail("tests/ui/static_ref.rs");
    // t.compile_fail("tests/ui/wrong_aspyref_lifetimes.rs");
    // #[cfg(not(feature = "uuid"))]
    // t.compile_fail("tests/ui/invalid_pyfunctions.rs");
    // t.compile_fail("tests/ui/invalid_pymethods.rs");
    // // output changes with async feature
    // #[cfg(all(not(Py_3_12), Py_LIMITED_API, feature = "experimental-async"))]
    // t.compile_fail("tests/ui/abi3_nativetype_inheritance.rs");
    // #[cfg(not(feature = "experimental-async"))]
    // t.compile_fail("tests/ui/invalid_async.rs");
    // t.compile_fail("tests/ui/invalid_intern_arg.rs");
    // t.compile_fail("tests/ui/invalid_frozen_pyclass_borrow.rs");
    // #[cfg(not(any(feature = "hashbrown", feature = "indexmap")))]
    // t.compile_fail("tests/ui/invalid_pymethod_receiver.rs");
    // #[cfg(not(feature = "experimental-inspect"))]
    // t.compile_fail("tests/ui/missing_intopy.rs");
    // // adding extra error conversion impls changes the output
    // #[cfg(not(any(windows, feature = "eyre", feature = "anyhow", Py_LIMITED_API)))]
    // t.compile_fail("tests/ui/invalid_result_conversion.rs");
    // t.compile_fail("tests/ui/not_send.rs");
    // t.compile_fail("tests/ui/not_send2.rs");
    // t.compile_fail("tests/ui/get_set_all.rs");
    // t.compile_fail("tests/ui/traverse.rs");
    // t.compile_fail("tests/ui/invalid_pymodule_in_root.rs");
    // t.compile_fail("tests/ui/invalid_pymodule_glob.rs");
    // t.compile_fail("tests/ui/invalid_pymodule_trait.rs");
    // t.compile_fail("tests/ui/invalid_pymodule_two_pymodule_init.rs");
    // #[cfg(all(feature = "experimental-async", not(feature = "experimental-inspect")))]
    // #[cfg(any(not(Py_LIMITED_API), Py_3_10))] // to avoid PyFunctionArgument for &str
    // t.compile_fail("tests/ui/invalid_cancel_handle.rs");
    // t.pass("tests/ui/pymodule_missing_docs.rs");
    // #[cfg(not(any(Py_LIMITED_API, feature = "experimental-inspect")))]
    // t.pass("tests/ui/forbid_unsafe.rs");
    // #[cfg(all(Py_LIMITED_API, not(feature = "experimental-async")))]
    // // output changes with async feature
    // t.compile_fail("tests/ui/abi3_inheritance.rs");
    // #[cfg(all(Py_LIMITED_API, not(Py_3_9)))]
    // t.compile_fail("tests/ui/abi3_weakref.rs");
    // #[cfg(all(Py_LIMITED_API, not(Py_3_9)))]
    // t.compile_fail("tests/ui/abi3_dict.rs");
    // #[cfg(not(feature = "experimental-inspect"))]
    // t.compile_fail("tests/ui/duplicate_pymodule_submodule.rs");
    // #[cfg(all(not(Py_LIMITED_API), Py_3_11))]
    // t.compile_fail("tests/ui/invalid_base_class.rs");
    // #[cfg(any(not(Py_3_10), all(not(Py_3_14), Py_LIMITED_API)))]
    // t.compile_fail("tests/ui/immutable_type.rs");
    // t.pass("tests/ui/ambiguous_associated_items.rs");
    // t.pass("tests/ui/pyclass_probe.rs");
    // t.compile_fail("tests/ui/invalid_pyfunction_warn.rs");
    // t.compile_fail("tests/ui/invalid_pymethods_warn.rs");

    use std::path::PathBuf;

    use ui_test::{run_tests, Config};

    let mut config = Config::rustc("tests/ui");

    let deps_features = vec![
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

    config
        .comment_defaults
        .base()
        .compile_flags
        .push("--diagnostic-width=140".into());

    // not a test file, used to configure dependencies for the tests
    config.skip_files.push("base/src/lib.rs".into());

    // don't run abi3-only tests when not testing abi3 features
    #[cfg(not(Py_LIMITED_API))]
    config.skip_files.push("abi3".into());

    #[cfg(Py_LIMITED_API)]
    config.skip_files.push("forbid_unsafe.rs".into());

    #[cfg(all(Py_LIMITED_API, not(Py_3_11)))]
    config.skip_files.push("buffer".into());

    #[cfg(any(Py_3_14, all(Py_3_10, not(Py_LIMITED_API))))]
    config.skip_files.push("immutable_type.rs".into());

    #[cfg(not(Py_3_9))]
    config.skip_files.push("invalid_pyclass_generic.rs".into());

    config.output_conflict_handling = ui_test::bless_output_files;

    let abort_check = config.abort_check.clone();
    ctrlc::set_handler(move || abort_check.abort()).unwrap();

    run_tests(config).unwrap();
}
