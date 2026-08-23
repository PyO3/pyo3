#![cfg(feature = "macros")]

use std::{
    env,
    panic::{RefUnwindSafe, UnwindSafe},
};

use ui_test::{spanned::Spanned, CommentParser, Revisioned};

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
        #[cfg(not(wip_feature_std))]
        "pyo3/hashbrown".to_string(),
        #[cfg(not(wip_feature_std))]
        "pyo3/parking_lot".to_string(),
        #[cfg(feature = "macros")]
        "pyo3/macros".to_string(),
        #[cfg(feature = "abi3")]
        "pyo3/abi3".to_string(),
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
    #[cfg(not(wip_feature_std))]
    deps_cargo
        .envs
        .push(("PYO3_WIP_NO_STD".into(), Some("1".into())));

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
        // Normalize paths into the Rust toolchain sources
        (
            Regex::new(r"[^\s]*?/rustlib/src/rust").unwrap().into(),
            b"$$RUST_SRC".to_vec(),
        ),
        // Some trait implementations which are only emitted with certain
        // features enabled
        (
            Regex::new(r"\n[ \t]*From<deranged::RangedI32<MIN, MAX>>")
                .unwrap()
                .into(),
            Vec::new(),
        ),
        (
            Regex::new(r"\n[ \t]*From<uuid::Uuid>").unwrap().into(),
            Vec::new(),
        ),
    ]);

    /// Generic function to configure a revision to require a given feature to
    /// be enabled or disabled (`custom_comments` requires function pointers).
    fn require_feature_enabled<F: Feature, const ENABLED: bool>(
        parser: &mut CommentParser<&mut Revisioned>,
        _args: Spanned<&str>,
        span: Span,
    ) {
        parser.set_custom_once(
            F::ENABLED_FLAG,
            SplitBuildOnFeature::<F>::new(ENABLED),
            span,
        );
    }

    config.custom_comments.insert(
        ExperimentalInspect::ENABLED_FLAG,
        require_feature_enabled::<ExperimentalInspect, true>,
    );
    config.custom_comments.insert(
        ExperimentalInspect::DISABLED_FLAG,
        require_feature_enabled::<ExperimentalInspect, false>,
    );
    config
        .custom_comments
        .insert(Std::ENABLED_FLAG, require_feature_enabled::<Std, true>);
    config
        .custom_comments
        .insert(Std::DISABLED_FLAG, require_feature_enabled::<Std, false>);

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
            r"\n[ \t]*--> (src(?:/|\\)\S+?):\d+:\d+((?:\n[ \t]*\d*[ \t]*[|=+-][^\n]*|\n[ \t]*\.\.\.)+)",
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

fn check_rust_src_paths(output: &[u8], errors: &mut Vec<ui_test::Error>) -> bool {
    use std::sync::LazyLock;

    use regex::bytes::Regex;

    static REMAPPED_RUST_SRC: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"/rustc/[0-9a-f]{40}/library/").unwrap());

    if REMAPPED_RUST_SRC.is_match(output) {
        // This causes `ui_test` to emit:
        //
        // ```
        // error: a bug in `ui_test` occurred
        // rust-src is required for UI tests; install it with `rustup component add rust-src`
        // ```
        errors.push(ui_test::Error::Bug(
            "rust-src is required for UI tests; install it with `rustup component add rust-src`"
                .into(),
        ));
        false
    } else {
        true
    }
}

fn error_on_output_conflict_normalized(
    path: &std::path::Path,
    output: &[u8],
    errors: &mut Vec<ui_test::Error>,
    config: &ui_test::per_test_config::TestConfig,
) {
    if check_rust_src_paths(output, errors) {
        ui_test::error_on_output_conflict(path, &normalize_src_blocks(output), errors, config);
    }
}

fn bless_output_files_normalized(
    path: &std::path::Path,
    output: &[u8],
    errors: &mut Vec<ui_test::Error>,
    config: &ui_test::per_test_config::TestConfig,
) {
    if check_rust_src_paths(output, errors) {
        ui_test::bless_output_files(path, &normalize_src_blocks(output), errors, config);
    }
}

/// Trait naming a feature which may be enabled or disabled in a given build.
///
/// The trait bounds are useful because `ui_test`'s `Flag` trait requires all these.
trait Feature: Send + Sync + UnwindSafe + RefUnwindSafe + 'static {
    const ENABLED: bool;
    const ENABLED_FLAG: &'static str;
    const DISABLED_FLAG: &'static str;
}

struct ExperimentalInspect;

impl Feature for ExperimentalInspect {
    const ENABLED: bool = cfg!(feature = "experimental-inspect");
    const ENABLED_FLAG: &'static str = "with-experimental-inspect";
    const DISABLED_FLAG: &'static str = "no-experimental-inspect";
}

struct Std;

impl Feature for Std {
    const ENABLED: bool = cfg!(wip_feature_std);
    const ENABLED_FLAG: &'static str = "with-std";
    const DISABLED_FLAG: &'static str = "no-std";
}
/// Some tests have different error messages when a given feature is
/// enabled.
struct SplitBuildOnFeature<F: Feature> {
    /// Whether the revision requires the feature to be enabled.
    feature_required: bool,
    phantom: std::marker::PhantomData<F>,
}

// Avoid `#[derive(Clone)]` because `F` may not be `Clone`.
impl<F: Feature> Clone for SplitBuildOnFeature<F> {
    fn clone(&self) -> Self {
        Self {
            feature_required: self.feature_required,
            phantom: std::marker::PhantomData,
        }
    }
}

// Debug the revision as the feature flag which it requires
impl<F: Feature> std::fmt::Debug for SplitBuildOnFeature<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SplitBuildOnFeature")
            .field(if self.feature_required {
                &F::ENABLED_FLAG
            } else {
                &F::DISABLED_FLAG
            })
            .finish()
    }
}

impl<F: Feature> SplitBuildOnFeature<F> {
    fn new(feature_required: bool) -> Self {
        Self {
            feature_required,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<F: Feature> ui_test::custom_flags::Flag for SplitBuildOnFeature<F> {
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
        self.feature_required != F::ENABLED
    }
}
