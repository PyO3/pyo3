from contextlib import contextmanager
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from functools import lru_cache
from glob import glob
from pathlib import Path
from typing import Any, Callable, Dict, Iterator, List, Optional, Tuple

import nox
import nox.command

try:
    import tomllib as toml
except ImportError:
    try:
        import toml
    except ImportError:
        toml = None

nox.options.sessions = ["test", "clippy", "rustfmt", "ruff", "docs"]


PYO3_DIR = Path(__file__).parent
PYO3_TARGET = Path(os.environ.get("CARGO_TARGET_DIR", PYO3_DIR / "target")).absolute()
PYO3_GUIDE_SRC = PYO3_DIR / "guide" / "src"
PYO3_GUIDE_TARGET = PYO3_TARGET / "guide"
PYO3_DOCS_TARGET = PYO3_TARGET / "doc"
PY_VERSIONS = ("3.7", "3.8", "3.9", "3.10", "3.11", "3.12", "3.13")
PYPY_VERSIONS = ("3.7", "3.8", "3.9", "3.10")


@nox.session(venv_backend="none")
def test(session: nox.Session) -> None:
    test_rust(session)
    test_py(session)


@nox.session(name="test-rust", venv_backend="none")
def test_rust(session: nox.Session):
    _run_cargo_test(session, package="pyo3-build-config")
    _run_cargo_test(session, package="pyo3-macros-backend")
    _run_cargo_test(session, package="pyo3-macros")
    _run_cargo_test(session, package="pyo3-ffi")

    _run_cargo_test(session)
    _run_cargo_test(session, features="unlimited-api")
    if "skip-full" not in session.posargs:
        _run_cargo_test(session, features="full")
        _run_cargo_test(session, features="full gil-refs")


@nox.session(name="test-py", venv_backend="none")
def test_py(session: nox.Session) -> None:
    _run(session, "nox", "-f", "pytests/noxfile.py", external=True)
    for example in glob("examples/*/noxfile.py"):
        _run(session, "nox", "-f", example, external=True)


@nox.session(venv_backend="none")
def coverage(session: nox.Session) -> None:
    session.env.update(_get_coverage_env())
    _run_cargo(session, "llvm-cov", "clean", "--workspace")
    test(session)

    cov_format = "codecov"
    output_file = "coverage.json"

    if "lcov" in session.posargs:
        cov_format = "lcov"
        output_file = "lcov.info"

    _run_cargo(
        session,
        "llvm-cov",
        "--package=pyo3",
        "--package=pyo3-build-config",
        "--package=pyo3-macros-backend",
        "--package=pyo3-macros",
        "--package=pyo3-ffi",
        "report",
        f"--{cov_format}",
        "--output-path",
        output_file,
    )


@nox.session(venv_backend="none")
def rustfmt(session: nox.Session):
    _run_cargo(session, "fmt", "--all", "--check")
    _run_cargo(session, "fmt", _FFI_CHECK, "--all", "--check")


@nox.session(name="ruff")
def ruff(session: nox.Session):
    session.install("ruff")
    _run(session, "ruff", "format", ".", "--check")
    _run(session, "ruff", "check", ".")


@nox.session(name="clippy", venv_backend="none")
def clippy(session: nox.Session) -> bool:
    if not _clippy(session) and _clippy_additional_workspaces(session):
        session.error("one or more jobs failed")


def _clippy(session: nox.Session, *, env: Dict[str, str] = None) -> bool:
    success = True
    env = env or os.environ
    for feature_set in _get_feature_sets():
        try:
            _run_cargo(
                session,
                "clippy",
                *feature_set,
                "--all-targets",
                "--workspace",
                "--",
                "--deny=warnings",
                env=env,
            )
        except nox.command.CommandFailed:
            success = False
    return success


def _clippy_additional_workspaces(session: nox.Session) -> bool:
    # pyo3-benches and pyo3-ffi-check are in isolated workspaces so that their
    # dependencies do not interact with MSRV

    success = True
    try:
        _run_cargo(session, "clippy", _BENCHES)
    except Exception:
        success = False

    # Run pyo3-ffi-check only on when not cross-compiling, because it needs to
    # have Python headers to feed to bindgen which gets messy when cross-compiling.
    target = os.environ.get("CARGO_BUILD_TARGET")
    if target is None or _get_rust_default_target() == target:
        try:
            _build_docs_for_ffi_check(session)
            _run_cargo(session, "clippy", _FFI_CHECK, "--workspace", "--all-targets")
        except Exception:
            success = False
    return success


@nox.session(venv_backend="none")
def bench(session: nox.Session) -> bool:
    _run_cargo(session, "bench", _BENCHES, *session.posargs)


@nox.session()
def codspeed(session: nox.Session) -> bool:
    # rust benchmarks
    os.chdir(PYO3_DIR / "pyo3-benches")
    _run_cargo(session, "codspeed", "build")
    _run_cargo(session, "codspeed", "run")
    # python benchmarks
    os.chdir(PYO3_DIR / "pytests")
    session.install(".[dev]", "pytest-codspeed")
    _run(session, "pytest", "--codspeed", external=True)


@nox.session(name="clippy-all", venv_backend="none")
def clippy_all(session: nox.Session) -> None:
    success = True

    def _clippy_with_config(env: Dict[str, str]) -> None:
        nonlocal success
        success &= _clippy(session, env=env)

    _for_all_version_configs(session, _clippy_with_config)
    success &= _clippy_additional_workspaces(session)

    if not success:
        session.error("one or more jobs failed")


@nox.session(name="check-all", venv_backend="none")
def check_all(session: nox.Session) -> None:
    success = True

    def _check(env: Dict[str, str]) -> None:
        nonlocal success
        for feature_set in _get_feature_sets():
            try:
                _run_cargo(
                    session,
                    "check",
                    *feature_set,
                    "--all-targets",
                    "--workspace",
                    env=env,
                )
            except Exception:
                success = False

    _for_all_version_configs(session, _check)

    if not success:
        session.error("one or more jobs failed")


@nox.session(venv_backend="none")
def publish(session: nox.Session) -> None:
    _run_cargo_publish(session, package="pyo3-build-config")
    _run_cargo_publish(session, package="pyo3-macros-backend")
    _run_cargo_publish(session, package="pyo3-macros")
    _run_cargo_publish(session, package="pyo3-ffi")
    _run_cargo_publish(session, package="pyo3")


@nox.session(venv_backend="none")
def contributors(session: nox.Session) -> None:
    import requests

    if len(session.posargs) < 1:
        raise Exception("base commit positional argument missing")

    base = session.posargs[0]
    page = 1

    head = "HEAD"
    if len(session.posargs) == 2:
        head = session.posargs[1]

    if len(session.posargs) > 2:
        raise Exception("too many arguments")

    authors = set()

    while True:
        resp = requests.get(
            f"https://api.github.com/repos/PyO3/pyo3/compare/{base}...{head}",
            params={"page": page, "per_page": 100},
        )

        body = resp.json()

        if resp.status_code != 200:
            raise Exception(
                f"failed to retrieve commits: {resp.status_code} {body['message']}"
            )

        for commit in body["commits"]:
            try:
                authors.add(commit["author"]["login"])
            except Exception:
                continue

        if "next" in resp.links:
            page += 1
        else:
            break

    authors = sorted(list(authors), key=lambda author: author.lower())

    for author in authors:
        print(f"@{author}")


class EmscriptenInfo:
    def __init__(self):
        self.emscripten_dir = PYO3_DIR / "emscripten"
        self.builddir = PYO3_DIR / ".nox/emscripten"
        self.builddir.mkdir(exist_ok=True, parents=True)

        self.pyversion = sys.version.split()[0]
        self.pymajor, self.pyminor, self.pymicro = self.pyversion.split(".")
        self.pymicro, self.pydev = re.match(
            "([0-9]*)([^0-9].*)?", self.pymicro
        ).groups()
        if self.pydev is None:
            self.pydev = ""

        self.pymajorminor = f"{self.pymajor}.{self.pyminor}"
        self.pymajorminormicro = f"{self.pymajorminor}.{self.pymicro}"


@nox.session(name="build-emscripten", venv_backend="none")
def build_emscripten(session: nox.Session):
    info = EmscriptenInfo()
    _run(
        session,
        "make",
        "-C",
        str(info.emscripten_dir),
        f"PYTHON={sys.executable}",
        f"BUILDROOT={info.builddir}",
        f"PYMAJORMINORMICRO={info.pymajorminormicro}",
        f"PYPRERELEASE={info.pydev}",
        external=True,
    )


@nox.session(name="test-emscripten", venv_backend="none")
def test_emscripten(session: nox.Session):
    info = EmscriptenInfo()

    libdir = info.builddir / f"install/Python-{info.pyversion}/lib"
    pythonlibdir = libdir / f"python{info.pymajorminor}"

    target = "wasm32-unknown-emscripten"

    session.env["CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_RUNNER"] = "python " + str(
        info.emscripten_dir / "runner.py"
    )
    session.env["RUSTFLAGS"] = " ".join(
        [
            f"-L native={libdir}",
            "-C link-arg=--preload-file",
            f"-C link-arg={pythonlibdir}@/lib/python{info.pymajorminor}",
            f"-C link-arg=-lpython{info.pymajorminor}",
            "-C link-arg=-lexpat",
            "-C link-arg=-lmpdec",
            "-C link-arg=-lz",
            "-C link-arg=-lbz2",
            "-C link-arg=-sALLOW_MEMORY_GROWTH=1",
        ]
    )
    session.env["CARGO_BUILD_TARGET"] = target
    session.env["PYO3_CROSS_LIB_DIR"] = pythonlibdir
    _run(session, "rustup", "target", "add", target, "--toolchain", "stable")
    _run(
        session,
        "bash",
        "-c",
        f"source {info.builddir/'emsdk/emsdk_env.sh'} && cargo test",
    )


@nox.session(venv_backend="none")
def docs(session: nox.Session) -> None:
    rustdoc_flags = ["-Dwarnings"]
    toolchain_flags = []
    cargo_flags = []

    if "open" in session.posargs:
        cargo_flags.append("--open")

    if "nightly" in session.posargs:
        rustdoc_flags.append("--cfg docsrs")
        toolchain_flags.append("+nightly")
        cargo_flags.extend(["-Z", "unstable-options", "-Z", "rustdoc-scrape-examples"])

    if "nightly" in session.posargs and "internal" in session.posargs:
        rustdoc_flags.append("--Z unstable-options")
        rustdoc_flags.append("--document-hidden-items")
        rustdoc_flags.extend(("--html-after-content", ".netlify/internal_banner.html"))
        cargo_flags.append("--document-private-items")
    else:
        cargo_flags.extend(["--exclude=pyo3-macros", "--exclude=pyo3-macros-backend"])

    rustdoc_flags.append(session.env.get("RUSTDOCFLAGS", ""))
    session.env["RUSTDOCFLAGS"] = " ".join(rustdoc_flags)

    shutil.rmtree(PYO3_DOCS_TARGET, ignore_errors=True)
    _run_cargo(
        session,
        *toolchain_flags,
        "doc",
        "--lib",
        "--no-default-features",
        "--features=full",
        "--no-deps",
        "--workspace",
        *cargo_flags,
    )


@nox.session(name="build-guide", venv_backend="none")
def build_guide(session: nox.Session):
    shutil.rmtree(PYO3_GUIDE_TARGET, ignore_errors=True)
    _run(session, "mdbook", "build", "-d", PYO3_GUIDE_TARGET, "guide", *session.posargs)
    for license in ("LICENSE-APACHE", "LICENSE-MIT"):
        target_file = PYO3_GUIDE_TARGET / license
        target_file.unlink(missing_ok=True)
        shutil.copy(PYO3_DIR / license, target_file)


@nox.session(name="check-guide", venv_backend="none")
def check_guide(session: nox.Session):
    # reuse other sessions, but with default args
    posargs = [*session.posargs]
    del session.posargs[:]
    build_guide(session)
    docs(session)
    session.posargs.extend(posargs)

    remaps = {
        f"file://{PYO3_GUIDE_SRC}/([^/]*/)*?%7B%7B#PYO3_DOCS_URL}}}}": f"file://{PYO3_DOCS_TARGET}",
        "%7B%7B#PYO3_DOCS_VERSION}}": "latest",
    }
    remap_args = []
    for key, value in remaps.items():
        remap_args.extend(("--remap", f"{key} {value}"))
    # check all links in the guide
    _run(
        session,
        "lychee",
        "--include-fragments",
        str(PYO3_GUIDE_SRC),
        *remap_args,
        *session.posargs,
    )
    # check external links in the docs
    # (intra-doc links are checked by rustdoc)
    _run(
        session,
        "lychee",
        str(PYO3_DOCS_TARGET),
        f"--remap=https://pyo3.rs/main/ file://{PYO3_GUIDE_TARGET}/",
        f"--remap=https://pyo3.rs/latest/ file://{PYO3_GUIDE_TARGET}/",
        f"--exclude=file://{PYO3_DOCS_TARGET}",
        "--exclude=http://www.adobe.com/",
        *session.posargs,
    )


@nox.session(name="format-guide", venv_backend="none")
def format_guide(session: nox.Session):
    fence_line = "//! ```\n"

    for path in Path("guide").glob("**/*.md"):
        session.log("Working on %s", path)
        lines = iter(path.read_text().splitlines(True))
        new_lines = []

        for line in lines:
            new_lines.append(line)
            if not re.search("```rust(,.*)?$", line):
                continue

            # Found a code block fence, gobble up its lines and write to temp. file
            prefix = line[: line.index("```")]
            with tempfile.NamedTemporaryFile("w", delete=False) as file:
                tempname = file.name
                file.write(fence_line)
                for line in lines:
                    if line == prefix + "```\n":
                        break
                    file.write(("//! " + line[len(prefix) :]).rstrip() + "\n")
                file.write(fence_line)

            # Format it (needs nightly rustfmt for `format_code_in_doc_comments`)
            _run(
                session,
                "rustfmt",
                "+nightly",
                "--config",
                "format_code_in_doc_comments=true",
                "--config",
                "reorder_imports=false",
                tempname,
            )

            # Re-read the formatted file, add its lines, and delete it
            with open(tempname, "r") as file:
                for line in file:
                    if line == fence_line:
                        continue
                    new_lines.append((prefix + line[4:]).rstrip() + "\n")
            os.unlink(tempname)

            new_lines.append(prefix + "```\n")

        path.write_text("".join(new_lines))


@nox.session(name="address-sanitizer", venv_backend="none")
def address_sanitizer(session: nox.Session):
    _run_cargo(
        session,
        "+nightly",
        "test",
        "--release",
        "-Zbuild-std",
        f"--target={_get_rust_default_target()}",
        "--",
        "--test-threads=1",
        env={
            "RUSTFLAGS": "-Zsanitizer=address",
            "RUSTDOCFLAGS": "-Zsanitizer=address",
            "ASAN_OPTIONS": "detect_leaks=0",
        },
    )


_IGNORE_CHANGELOG_PR_CATEGORIES = (
    "release",
    "docs",
)


@nox.session(name="check-changelog")
def check_changelog(session: nox.Session):
    if not _is_github_actions():
        session.error("Can only check changelog on github actions")

    event_path = os.environ["GITHUB_EVENT_PATH"]

    with open(event_path) as event_file:
        event = json.load(event_file)

    for category in _IGNORE_CHANGELOG_PR_CATEGORIES:
        if event["pull_request"]["title"].startswith(f"{category}:"):
            session.skip(f"PR title starts with {category}")

    for label in event["pull_request"]["labels"]:
        if label["name"] == "CI-skip-changelog":
            session.skip("CI-skip-changelog label applied")

    issue_number = event["pull_request"]["number"]

    newsfragments = PYO3_DIR / "newsfragments"

    fragments = tuple(
        filter(
            Path.exists,
            (
                newsfragments / f"{issue_number}.{change_type}.md"
                for change_type in ("packaging", "added", "changed", "removed", "fixed")
            ),
        )
    )

    if not fragments:
        session.error(
            "Changelog entry not found, please add one (or more) to the `newsfragments` directory.\n"
            "Alternatively, start the PR title with `docs:` if this PR is a docs-only PR.\n"
            "See https://github.com/PyO3/pyo3/blob/main/Contributing.md#documenting-changes for more information."
        )

    print("Found newsfragments:")
    for fragment in fragments:
        print(fragment.name)


@nox.session(name="set-minimal-package-versions", venv_backend="none")
def set_minimal_package_versions(session: nox.Session):
    from collections import defaultdict

    if toml is None:
        session.error("requires Python 3.11 or `toml` to be installed")

    projects = (
        None,
        "examples/decorator",
        "examples/maturin-starter",
        "examples/setuptools-rust-starter",
        "examples/word-count",
    )
    min_pkg_versions = {
        "regex": "1.9.6",
        "proptest": "1.2.0",
        "trybuild": "1.0.89",
        "eyre": "0.6.8",
        "allocator-api2": "0.2.10",
    }

    # run cargo update first to ensure that everything is at highest
    # possible version, so that this matches what CI will resolve to.
    for project in projects:
        if project is None:
            _run_cargo(session, "update")
        else:
            _run_cargo(session, "update", f"--manifest-path={project}/Cargo.toml")

    for project in projects:
        lock_file = Path(project or "") / "Cargo.lock"

        def load_pkg_versions():
            cargo_lock = toml.loads(lock_file.read_text())
            # Cargo allows to depends on multiple versions of the same package
            pkg_versions = defaultdict(list)
            for pkg in cargo_lock["package"]:
                name = pkg["name"]
                if name not in min_pkg_versions:
                    continue
                pkg_versions[name].append(pkg["version"])
            return pkg_versions

        pkg_versions = load_pkg_versions()
        for pkg_name, min_version in min_pkg_versions.items():
            versions = pkg_versions.get(pkg_name, [])
            for version in versions:
                if version != min_version:
                    pkg_id = pkg_name + ":" + version
                    _run_cargo_set_package_version(
                        session, pkg_id, min_version, project=project
                    )
                    # assume `_run_cargo_set_package_version` has changed something
                    # and re-read `Cargo.lock`
                    pkg_versions = load_pkg_versions()

    # As a smoke test, cargo metadata solves all dependencies, so
    # will break if any crates rely on cargo features not
    # supported on MSRV
    for project in projects:
        if project is None:
            _run_cargo(session, "metadata", silent=True)
        else:
            _run_cargo(
                session,
                "metadata",
                f"--manifest-path={project}/Cargo.toml",
                silent=True,
            )


@nox.session(name="ffi-check")
def ffi_check(session: nox.Session):
    _build_docs_for_ffi_check(session)
    _run_cargo(session, "run", _FFI_CHECK)


@nox.session(name="test-version-limits")
def test_version_limits(session: nox.Session):
    env = os.environ.copy()
    with _config_file() as config_file:
        env["PYO3_CONFIG_FILE"] = config_file.name

        assert "3.6" not in PY_VERSIONS
        config_file.set("CPython", "3.6")
        _run_cargo(session, "check", env=env, expect_error=True)

        assert "3.14" not in PY_VERSIONS
        config_file.set("CPython", "3.14")
        _run_cargo(session, "check", env=env, expect_error=True)

        # 3.14 CPython should build with forward compatibility
        env["PYO3_USE_ABI3_FORWARD_COMPATIBILITY"] = "1"
        _run_cargo(session, "check", env=env)

        assert "3.6" not in PYPY_VERSIONS
        config_file.set("PyPy", "3.6")
        _run_cargo(session, "check", env=env, expect_error=True)

        assert "3.11" not in PYPY_VERSIONS
        config_file.set("PyPy", "3.11")
        _run_cargo(session, "check", env=env, expect_error=True)


@nox.session(name="check-feature-powerset", venv_backend="none")
def check_feature_powerset(session: nox.Session):
    if toml is None:
        session.error("requires Python 3.11 or `toml` to be installed")

    cargo_toml = toml.loads((PYO3_DIR / "Cargo.toml").read_text())

    EXCLUDED_FROM_FULL = {
        "nightly",
        "gil-refs",
        "extension-module",
        "full",
        "default",
        "auto-initialize",
        "generate-import-lib",
        "multiple-pymethods",  # Because it's not supported on wasm
    }

    features = cargo_toml["features"]

    full_feature = set(features["full"])
    abi3_features = {feature for feature in features if feature.startswith("abi3")}
    abi3_version_features = abi3_features - {"abi3"}

    expected_full_feature = features.keys() - EXCLUDED_FROM_FULL - abi3_features

    uncovered_features = expected_full_feature - full_feature
    if uncovered_features:
        session.error(
            f"some features missing from `full` meta feature: {uncovered_features}"
        )

    experimental_features = {
        feature for feature in features if feature.startswith("experimental-")
    }
    full_without_experimental = full_feature - experimental_features

    if len(experimental_features) >= 2:
        # justification: we always assume that feature within these groups are
        # mutually exclusive to simplify CI
        features_to_group = [
            full_without_experimental,
            experimental_features,
        ]
    elif len(experimental_features) == 1:
        # no need to make an experimental features group
        features_to_group = [full_without_experimental]
    else:
        session.error("no experimental features exist; please simplify the noxfile")

    features_to_skip = [
        *(EXCLUDED_FROM_FULL - {"gil-refs"}),
        *abi3_version_features,
    ]

    # deny warnings
    env = os.environ.copy()
    rust_flags = env.get("RUSTFLAGS", "")
    env["RUSTFLAGS"] = f"{rust_flags} -Dwarnings"

    comma_join = ",".join
    _run_cargo(
        session,
        "hack",
        "--feature-powerset",
        '--optional-deps=""',
        f'--skip="{comma_join(features_to_skip)}"',
        *(f"--group-features={comma_join(group)}" for group in features_to_group),
        "check",
        "--all-targets",
        env=env,
    )


@nox.session(name="update-ui-tests", venv_backend="none")
def update_ui_tests(session: nox.Session):
    env = os.environ.copy()
    env["TRYBUILD"] = "overwrite"
    command = ["test", "--test", "test_compile_error"]
    _run_cargo(session, *command, env=env)
    _run_cargo(session, *command, "--features=full", env=env)


def _build_docs_for_ffi_check(session: nox.Session) -> None:
    # pyo3-ffi-check needs to scrape docs of pyo3-ffi
    env = os.environ.copy()
    env["PYO3_PYTHON"] = sys.executable
    _run_cargo(session, "doc", _FFI_CHECK, "-p", "pyo3-ffi", "--no-deps", env=env)


@lru_cache()
def _get_rust_info() -> Tuple[str, ...]:
    output = _get_output("rustc", "-vV")

    return tuple(output.splitlines())


def _get_rust_version() -> Tuple[int, int, int, List[str]]:
    for line in _get_rust_info():
        if line.startswith(_RELEASE_LINE_START):
            version = line[len(_RELEASE_LINE_START) :].strip()
            # e.g. 1.67.0-beta.2
            (version_number, *extra) = version.split("-", maxsplit=1)
            return (*map(int, version_number.split(".")), extra)


def _get_rust_default_target() -> str:
    for line in _get_rust_info():
        if line.startswith(_HOST_LINE_START):
            return line[len(_HOST_LINE_START) :].strip()


@lru_cache()
def _get_feature_sets() -> Tuple[Tuple[str, ...], ...]:
    """Returns feature sets to use for clippy job"""
    cargo_target = os.getenv("CARGO_BUILD_TARGET", "")
    if "wasm32-wasi" not in cargo_target:
        # multiple-pymethods not supported on wasm
        return (
            ("--no-default-features",),
            (
                "--no-default-features",
                "--features=unlimited-api",
            ),
            ("--features=full gil-refs multiple-pymethods",),
            ("--features=full gil-refs multiple-pymethods unlimited-api",),
        )
    else:
        return (
            ("--no-default-features",),
            (
                "--no-default-features",
                "--features=unlimited-api",
            ),
            ("--features=full gil-refs",),
            ("--features=full gil-refs unlimited-api",),
        )


_RELEASE_LINE_START = "release: "
_HOST_LINE_START = "host: "


def _get_coverage_env() -> Dict[str, str]:
    env = {}
    output = _get_output("cargo", "llvm-cov", "show-env")

    for line in output.strip().splitlines():
        (key, value) = line.split("=", maxsplit=1)
        # Strip single or double quotes from the variable value
        # - quote used by llvm-cov differs between Windows and Linux
        if value and value[0] in ("'", '"'):
            value = value[1:-1]
        env[key] = value

    # Ensure that examples/ and pytests/ all build to the correct target directory to collect
    # coverage artifacts.
    env["CARGO_TARGET_DIR"] = env["CARGO_LLVM_COV_TARGET_DIR"]

    return env


def _run(session: nox.Session, *args: str, **kwargs: Any) -> None:
    """Wrapper for _run(session, which creates nice groups on GitHub Actions."""
    is_github_actions = _is_github_actions()
    failed = False
    if is_github_actions:
        # Insert ::group:: at the start of nox's command line output
        print("::group::", end="", flush=True, file=sys.stderr)
    try:
        session.run(*args, **kwargs)
    except nox.command.CommandFailed:
        failed = True
        raise
    finally:
        if is_github_actions:
            print("::endgroup::", file=sys.stderr)
            # Defer the error message until after the group to make them easier
            # to find in the log
            if failed:
                command = " ".join(args)
                print(f"::error::`{command}` failed", file=sys.stderr)


def _run_cargo(
    session: nox.Session, *args: str, expect_error: bool = False, **kwargs: Any
) -> None:
    if expect_error:
        if "success_codes" in kwargs:
            raise ValueError("expect_error overrides success_codes")
        kwargs["success_codes"] = [101]
    _run(session, "cargo", *args, **kwargs, external=True)


def _run_cargo_test(
    session: nox.Session,
    *,
    package: Optional[str] = None,
    features: Optional[str] = None,
) -> None:
    command = ["cargo"]
    if "careful" in session.posargs:
        command.append("careful")
    command.extend(("test", "--no-fail-fast"))
    if "release" in session.posargs:
        command.append("--release")
    if package:
        command.append(f"--package={package}")
    if features:
        command.append(f"--features={features}")

    _run(session, *command, external=True)


def _run_cargo_publish(session: nox.Session, *, package: str) -> None:
    _run_cargo(session, "publish", f"--package={package}")


def _run_cargo_set_package_version(
    session: nox.Session,
    pkg_id: str,
    version: str,
    *,
    project: Optional[str] = None,
) -> None:
    command = ["cargo", "update", "-p", pkg_id, "--precise", version, "--workspace"]
    if project:
        command.append(f"--manifest-path={project}/Cargo.toml")
    _run(session, *command, external=True)


def _get_output(*args: str) -> str:
    return subprocess.run(args, capture_output=True, text=True, check=True).stdout


def _for_all_version_configs(
    session: nox.Session, job: Callable[[Dict[str, str]], None]
) -> None:
    env = os.environ.copy()
    with _config_file() as config_file:
        env["PYO3_CONFIG_FILE"] = config_file.name

        def _job_with_config(implementation, version):
            session.log(f"{implementation} {version}")
            config_file.set(implementation, version)
            job(env)

        for version in PY_VERSIONS:
            _job_with_config("CPython", version)

        for version in PYPY_VERSIONS:
            _job_with_config("PyPy", version)


class _ConfigFile:
    def __init__(self, config_file) -> None:
        self._config_file = config_file

    def set(self, implementation: str, version: str) -> None:
        """Set the contents of this config file to the given implementation and version."""
        self._config_file.seek(0)
        self._config_file.truncate(0)
        self._config_file.write(
            f"""\
implementation={implementation}
version={version}
suppress_build_script_link_lines=true
"""
        )
        self._config_file.flush()

    @property
    def name(self) -> str:
        return self._config_file.name


@contextmanager
def _config_file() -> Iterator[_ConfigFile]:
    """Creates a temporary config file which can be repeatedly set to different values."""
    with tempfile.NamedTemporaryFile("r+") as config:
        yield _ConfigFile(config)


def _is_github_actions() -> bool:
    return "GITHUB_ACTIONS" in os.environ


_BENCHES = "--manifest-path=pyo3-benches/Cargo.toml"
_FFI_CHECK = "--manifest-path=pyo3-ffi-check/Cargo.toml"
