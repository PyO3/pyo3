#![cfg(feature = "macros")]

fn main() {
    if cfg!(target_arch = "wasm32") {
        // Not possible to invoke compiler from wasm
        return;
    }

    use std::{env::VarError, path::PathBuf};

    use regex::bytes::Regex;
    use ui_test::{run_tests, spanned::Span, Config, OptWithLine};

    let mut config = Config::rustc("tests/ui");

    // Various configurations of UI_TEST environment variable for different CI modes
    match std::env::var("UI_TEST").as_deref() {
        // Default is to run the test as normal, erroring if output is not as expected.
        Err(VarError::NotPresent) => {
            config.output_conflict_handling = error_on_output_conflict_normalized
        }
        // Used to update the output files to match expected output
        Ok("bless") => config.output_conflict_handling = bless_output_files_normalized,
        // This mode is useful for exercising coverage of the proc macros, e.g. on the
        // nightly compiler and MSRV, where the output may differ from expected.
        Ok("ignore") => {
            // Ignore mismatches on stderr / stdout files
            config.output_conflict_handling = ui_test::ignore_output_conflict;

            // This combination of settings helps ui test ignore the annotations on
            // the test files themselves:

            // The annotations by default start with //~, changing this to a pattern
            // which never appears in the files effectively means "ignore all annotations"
            config.comment_start = "/*DISABLED*/";
            // Don't error if there are no annotations
            config.comment_defaults.base().require_annotations =
                OptWithLine::new(false, Span::default());
            // Don't error if the test "passes" because there were no annotations
            config.comment_defaults.base().exit_status = OptWithLine::default();
        }
        // Completely running the tests, e.g. under `cargo careful` there is some issue which
        // doesn't seem worth understanding (we don't gain anything from extra assertions in
        // the proc-macro code, which is all quite pedestrian).
        Ok("skip") => return,
        Err(e) => panic!("error reading UI_TEST environment variable: {e}"),
        Ok(unknown) => panic!("invalid UI_TEST value: {unknown}"),
    }

    config.bless_command = Some("UI_TEST=bless cargo test --test test_compile_error".into());

    // There doesn't seem to be a good way to forward all these features automatically,
    // so have to just list the relevant ones here.
    let deps_features = [
        #[cfg(feature = "macros")]
        "pyo3/macros".to_string(),
        #[cfg(feature = "abi3")]
        "pyo3/abi3".to_string(),
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
        #[cfg(feature = "full")]
        "pyo3/full".to_string(),
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
        // similarly, just a component of `invalid_pymodule_in_root.rs`
        "empty.rs".into(),
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
        // an extra "note" is emitted on abi3
        #[cfg(any(not(Py_LIMITED_API), not(Py_3_12)))]
        "invalid_base_class.rs".into(),
        #[cfg(all(Py_LIMITED_API, not(Py_3_10)))]
        "invalid_pyfunction_argument.rs".into(),
        #[cfg(all(Py_LIMITED_API, not(Py_3_10)))]
        "invalid_pyclass_args.rs".into(),
        // tests that async functions are rejected without the feature
        #[cfg(feature = "experimental-async")]
        "invalid_async.rs".into(),
        #[cfg(any(
            // requires the async feature
            not(feature = "experimental-async"),
            // the `FromPyObject` argument for `&str` causes the output to differ
            all(Py_LIMITED_API, not(Py_3_10))
        ))]
        "invalid_cancel_handle.rs".into(),
    ]);

    config.comment_defaults.base().normalize_stderr.extend([
        // Normalize multiple trailing newlines to a single newline
        (Regex::new("\n\n$").unwrap().into(), vec![b'\n']),
        // Normalize counts of "and N others" in trait implementations
        (
            Regex::new(r"and \d+ others").unwrap().into(),
            b"and $$N others".to_vec(),
        ),
        // Some trait implementations which are only emitted with certain
        // features enabled
        (
            Regex::new(r"\n[ \t]*`i32` implements `From<deranged::RangedI32<MIN, MAX>>`")
                .unwrap()
                .into(),
            Vec::new(),
        ),
        (
            Regex::new(r"\n[ \t]*`String` implements `From<uuid::Uuid>`")
                .unwrap()
                .into(),
            Vec::new(),
        ),
    ]);

    config
        .custom_comments
        .insert("with-experimental-inspect", |parser, _args, span| {
            parser.set_custom_once(
                "with-experimental-inspect",
                SplitBuildOnExperimentalInpsect {
                    requires_inspect: true,
                },
                span,
            );
        });
    config
        .custom_comments
        .insert("without-experimental-inspect", |parser, _args, span| {
            parser.set_custom_once(
                "without-experimental-inspect",
                SplitBuildOnExperimentalInpsect {
                    requires_inspect: false,
                },
                span,
            );
        });

    // `ctrlc` doesn't build on wasm
    #[cfg(not(target_arch = "wasm32"))]
    {
        let abort_check = config.abort_check.clone();
        ctrlc::set_handler(move || abort_check.abort()).unwrap();
    }

    run_tests(config).unwrap();
}

/// Strips line:col information from src file references in error messages.
///
/// e.g. the following block:
///
/// ```
///    --> src/impl_/extract_argument.rs:226:8
///     |
/// 220 | pub fn extract_argument<'a, 'holder, 'py, T, const IMPLEMENTS_FROMPYOBJECT: bool>(
///     |        ---------------- required by a bound in this function
/// ...
/// 226 |     T: PyFunctionArgument<'a, 'holder, 'py, IMPLEMENTS_FROMPYOBJECT>,
///     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `extract_argument`
///     = note: required for `CancelHandle` to implement `FromPyObject<'_, '_>`
///     = note: required for `CancelHandle` to implement `pyo3::impl_::extract_argument::PyFunctionArgument<'_, '_, '_, true>`
/// ```
///
/// becomes:
///
/// ```
///  --> src/impl_/extract_argument.rs
///   |
///   | pub fn extract_argument<'a, 'holder, 'py, T, const IMPLEMENTS_FROMPYOBJECT: bool>(
///   |        ---------------- required by a bound in this function
/// ...
///   |     T: PyFunctionArgument<'a, 'holder, 'py, IMPLEMENTS_FROMPYOBJECT>,
///   |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `extract_argument`
///   = note: required for `CancelHandle` to implement `FromPyObject<'_, '_>`
///   = note: required for `CancelHandle` to implement `pyo3::impl_::extract_argument::PyFunctionArgument<'_, '_, '_, true>`
/// ```
///
/// Regex replacement via `ui_test`'s `normalize_stderr` can't express the transformation
/// we need here, so we write a custom wrapper which modifies the output before passing
/// to `ui_test`'s normal output handling machinery.
fn normalize_src_blocks(output: &[u8]) -> Vec<u8> {
    use std::sync::LazyLock;

    use regex::bytes::{Captures, Regex};

    // Matches the full block which we want to replace.
    //
    // The first line with the src path is captured, and then all following lines starting with either:
    // - a line number and `|`
    // - a line number and `=`
    // - a line number and `+` or `-` (suggested edit to fix the error)
    // - just `...`
    // are captured as the "listing".
    static SRC_BLOCK: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"\n[ \t]*--> (src/\S+?):\d+:\d+((?:\n[ \t]*\d*[ \t]*[|=+-][^\n]*|\n[ \t]*\.\.\.)+)",
        )
        .unwrap()
    });

    // Matches a gutter line in the listing (potentially with a line number)
    static GUTTER: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\n[ \t]*\d*[ \t]*([|=+-])").unwrap());

    SRC_BLOCK
        .replace_all(output, |captures: &Captures<'_>| {
            // always normalize gutter to two spaces, arrow to one space,
            // this leads to best stability
            let mut out = b"\n --> ".to_vec();
            out.extend_from_slice(&captures[1]);
            let listing = GUTTER.replace_all(&captures[2], b"\n  $1");
            out.extend_from_slice(&listing);
            out
        })
        .into_owned()
}

fn error_on_output_conflict_normalized(
    path: &std::path::Path,
    output: &[u8],
    errors: &mut Vec<ui_test::Error>,
    config: &ui_test::per_test_config::TestConfig,
) {
    ui_test::error_on_output_conflict(path, &normalize_src_blocks(output), errors, config);
}

fn bless_output_files_normalized(
    path: &std::path::Path,
    output: &[u8],
    errors: &mut Vec<ui_test::Error>,
    config: &ui_test::per_test_config::TestConfig,
) {
    ui_test::bless_output_files(path, &normalize_src_blocks(output), errors, config);
}

/// Some tests have different error messages when the `experimental-inspect` feature is
/// enabled.
#[derive(Clone, Debug)]
struct SplitBuildOnExperimentalInpsect {
    requires_inspect: bool,
}

impl ui_test::custom_flags::Flag for SplitBuildOnExperimentalInpsect {
    fn clone_inner(&self) -> Box<dyn ui_test::custom_flags::Flag> {
        Box::new(self.clone())
    }

    fn must_be_unique(&self) -> bool {
        true
    }

    fn test_condition(
        &self,
        _config: &ui_test::Config,
        _comments: &ui_test::Comments,
        _revision: &str,
    ) -> bool {
        // returning `true` skips the test, so return true when the feature doesn't
        // match the requirement of the test
        self.requires_inspect != cfg!(feature = "experimental-inspect")
    }
}
