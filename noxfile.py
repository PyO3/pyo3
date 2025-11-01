import io
import json
import os
import re
import shutil
import subprocess
import sys
import sysconfig
import tarfile
import tempfile
from contextlib import ExitStack, contextmanager
from functools import lru_cache
from glob import glob
from pathlib import Path
from typing import (
    Any,
    Callable,
    Dict,
    Iterable,
    Iterator,
    List,
    Optional,
    Tuple,
)

import nox.command

try:
    import tomllib as toml
except ImportError:
    try:
        import toml
    except ImportError:
        toml = None

try:
    import requests
except ImportError:
    requests = None

nox.options.sessions = ["test", "clippy", "rustfmt", "ruff", "rumdl", "docs"]

PYO3_DIR = Path(__file__).parent
PYO3_TARGET = Path(os.environ.get("CARGO_TARGET_DIR", PYO3_DIR / "target")).absolute()
PYO3_GUIDE_SRC = PYO3_DIR / "guide" / "src"
PYO3_GUIDE_TARGET = PYO3_TARGET / "guide"
PYO3_DOCS_TARGET = PYO3_TARGET / "doc"
FREE_THREADED_BUILD = bool(sysconfig.get_config_var("Py_GIL_DISABLED"))


def _get_output(*args: str) -> str:
    return subprocess.run(args, capture_output=True, text=True, check=True).stdout


def _parse_supported_interpreter_version(
    python_impl: str,  # Literal["cpython", "pypy"], TODO update after 3.7 dropped
) -> Tuple[str, str]:
    output = _get_output("cargo", "metadata", "--format-version=1", "--no-deps")
    cargo_packages = json.loads(output)["packages"]
    # Check Python interpreter version support in package metadata
    package = "pyo3-ffi"
    metadata = next(pkg["metadata"] for pkg in cargo_packages if pkg["name"] == package)
    version_info = metadata[python_impl]
    assert "min-version" in version_info, f"missing min-version for {python_impl}"
    assert "max-version" in version_info, f"missing max-version for {python_impl}"
    return version_info["min-version"], version_info["max-version"]


def _supported_interpreter_versions(
    python_impl: str,  # Literal["cpython", "pypy"], TODO update after 3.7 dropped
) -> List[str]:
    min_version, max_version = _parse_supported_interpreter_version(python_impl)
    major = int(min_version.split(".")[0])
    assert major == 3, f"unsupported Python major version {major}"
    min_minor = int(min_version.split(".")[1])
    max_minor = int(max_version.split(".")[1])
    versions = [f"{major}.{minor}" for minor in range(min_minor, max_minor + 1)]
    # Add free-threaded builds for 3.13+
    if python_impl == "cpython":
        versions += [f"{major}.{minor}t" for minor in range(13, max_minor + 1)]
    return versions


PY_VERSIONS = _supported_interpreter_versions("cpython")
PYPY_VERSIONS = _supported_interpreter_versions("pypy")


@nox.session(venv_backend="none")
def test(session: nox.Session) -> None:
    test_rust(session)
    test_py(session)


@nox.session(name="test-rust", venv_backend="none")
def test_rust(session: nox.Session):
    _run_cargo_test(session, package="pyo3-build-config")
    _run_cargo_test(session, package="pyo3-macros-backend")
    _run_cargo_test(session, package="pyo3-macros")

    extra_flags = []
    # pypy and graalpy don't have Py_Initialize APIs, so we can only
    # build the main tests, not run them
    if sys.implementation.name in ("pypy", "graalpy"):
        extra_flags.append("--no-run")

    _run_cargo_test(session, package="pyo3-ffi", extra_flags=extra_flags)

    extra_flags.append("--no-default-features")

    for feature_set in _get_feature_sets():
        flags = extra_flags.copy()

        if feature_set is None or "full" not in feature_set:
            # doctests require at least the macros feature, which is
            # activated by the full feature set
            #
            # using `--all-targets` makes cargo run everything except doctests
            flags.append("--all-targets")

        # We need to pass the feature set to the test command
        # so that it can be used in the test code
        # (e.g. for `#[cfg(feature = "abi3-py37")]`)
        if feature_set and "abi3" in feature_set and FREE_THREADED_BUILD:
            # free-threaded builds don't support abi3 yet
            continue

        _run_cargo_test(session, features=feature_set, extra_flags=flags)

        if (
            feature_set
            and "abi3" in feature_set
            and "full" in feature_set
            and sys.version_info >= (3, 7)
        ):
            # run abi3-py37 tests to check abi3 forward compatibility
            _run_cargo_test(
                session,
                features=feature_set.replace("abi3", "abi3-py37"),
                extra_flags=flags,
            )


@nox.session(name="test-py", venv_backend="none")
def test_py(session: nox.Session) -> None:
    _run(session, "nox", "-f", "pytests/noxfile.py", external=True)
    for example in glob("examples/*/noxfile.py"):
        _run(session, "nox", "-f", example, external=True)
    for example in glob("pyo3-ffi/examples/*/noxfile.py"):
        _run(session, "nox", "-f", example, external=True)


@nox.session(venv_backend="none")
def coverage(session: nox.Session) -> None:
    session.env.update(_get_coverage_env())
    _run_cargo(session, "llvm-cov", "clean", "--workspace")
    test(session)
    generate_coverage_report(session)


@nox.session(name="set-coverage-env", venv_backend="none")
def set_coverage_env(session: nox.Session) -> None:
    """For use in GitHub Actions to set coverage environment variables."""
    with open(os.environ["GITHUB_ENV"], "a") as env_file:
        for k, v in _get_coverage_env().items():
            print(f"{k}={v}", file=env_file)


@nox.session(name="generate-coverage-report", venv_backend="none")
def generate_coverage_report(session: nox.Session) -> None:
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


@nox.session(name="rumdl", venv_backend="none")
def rumdl(session: nox.Session):
    """Run rumdl to check markdown formatting in the guide.

    Can also run with uv directly, e.g. `uv run rumdl check guide`.
    """
    _run(
        session, "uv", "run", "rumdl", "check", "guide", *session.posargs, external=True
    )


@nox.session(name="clippy", venv_backend="none")
def clippy(session: nox.Session) -> bool:
    if not (_clippy(session) and _clippy_additional_workspaces(session)):
        session.error("one or more jobs failed")


def _clippy(session: nox.Session, *, env: Dict[str, str] = None) -> bool:
    success = True
    env = env or os.environ
    for feature_set in _get_feature_sets():
        try:
            _run_cargo(
                session,
                "clippy",
                "--no-default-features",
                *((f"--features={feature_set}",) if feature_set else ()),
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
                    "--no-default-features",
                    *((f"--features={feature_set}",) if feature_set else ()),
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
    _run_cargo_publish(session, package="pyo3-introspection")


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
            "-C link-arg=-lsqlite3",
            "-C link-arg=-lz",
            "-C link-arg=-lbz2",
            "-C link-arg=-sALLOW_MEMORY_GROWTH=1",
        ]
    )
    session.env["RUSTDOCFLAGS"] = session.env["RUSTFLAGS"]
    session.env["CARGO_BUILD_TARGET"] = target
    session.env["PYO3_CROSS_LIB_DIR"] = pythonlibdir
    _run(session, "rustup", "target", "add", target, "--toolchain", "stable")
    _run(
        session,
        "bash",
        "-c",
        f"source {info.builddir / 'emsdk/emsdk_env.sh'} && cargo test",
    )


@nox.session(name="test-cross-compilation-windows")
def test_cross_compilation_windows(session: nox.Session):
    session.install("cargo-xwin")

    env = os.environ.copy()
    env["XWIN_ARCH"] = "x86_64"

    # abi3
    _run_cargo(
        session,
        "build",
        "--manifest-path",
        "examples/maturin-starter/Cargo.toml",
        "--features",
        "abi3",
        "--target",
        "x86_64-pc-windows-gnu",
        env=env,
    )
    _run_cargo(
        session,
        "xwin",
        "build",
        "--cross-compiler",
        "clang",
        "--manifest-path",
        "examples/maturin-starter/Cargo.toml",
        "--features",
        "abi3",
        "--target",
        "x86_64-pc-windows-msvc",
        env=env,
    )

    # non-abi3
    env["PYO3_CROSS_PYTHON_VERSION"] = "3.13"
    _run_cargo(
        session,
        "build",
        "--manifest-path",
        "examples/maturin-starter/Cargo.toml",
        "--features",
        "generate-import-lib",
        "--target",
        "x86_64-pc-windows-gnu",
        env=env,
    )
    _run_cargo(
        session,
        "xwin",
        "build",
        "--cross-compiler",
        "clang",
        "--manifest-path",
        "examples/maturin-starter/Cargo.toml",
        "--features",
        "generate-import-lib",
        "--target",
        "x86_64-pc-windows-msvc",
        env=env,
    )


@nox.session(venv_backend="none")
def docs(session: nox.Session, nightly: bool = False, internal: bool = False) -> None:
    rustdoc_flags = ["-Dwarnings"]
    toolchain_flags = []
    cargo_flags = []

    nightly = nightly or ("nightly" in session.posargs)
    internal = internal or ("internal" in session.posargs)

    if "open" in session.posargs:
        cargo_flags.append("--open")

    if nightly:
        rustdoc_flags.append("--cfg docsrs")
        toolchain_flags.append("+nightly")
        cargo_flags.extend(["-Z", "unstable-options", "-Z", "rustdoc-scrape-examples"])

    if internal:
        rustdoc_flags.append("--Z unstable-options")
        rustdoc_flags.append("--document-hidden-items")
        rustdoc_flags.extend(("--html-after-content", ".netlify/internal_banner.html"))
        cargo_flags.append("--document-private-items")
    else:
        cargo_flags.extend(["--exclude=pyo3-macros", "--exclude=pyo3-macros-backend"])

    rustdoc_flags.append(session.env.get("RUSTDOCFLAGS", ""))
    session.env["RUSTDOCFLAGS"] = " ".join(rustdoc_flags)

    features = "full"

    shutil.rmtree(PYO3_DOCS_TARGET, ignore_errors=True)
    _run_cargo(
        session,
        *toolchain_flags,
        "doc",
        "--lib",
        "--no-default-features",
        f"--features={features}",
        "--no-deps",
        "--workspace",
        *cargo_flags,
    )


@nox.session(name="build-guide", venv_backend="none")
def build_guide(session: nox.Session):
    shutil.rmtree(PYO3_GUIDE_TARGET, ignore_errors=True)
    _run(
        session,
        "mdbook",
        "build",
        "-d",
        str(PYO3_GUIDE_TARGET),
        "guide",
        *session.posargs,
        external=True,
    )
    for license in ("LICENSE-APACHE", "LICENSE-MIT"):
        target_file = PYO3_GUIDE_TARGET / license
        target_file.unlink(missing_ok=True)
        shutil.copy(PYO3_DIR / license, target_file)


@nox.session(name="build-netlify-site")
def build_netlify_site(session: nox.Session):
    # Remove netlify_build directory if it exists
    netlify_build = Path("netlify_build")
    if netlify_build.exists():
        shutil.rmtree(netlify_build)

    url = "https://github.com/PyO3/pyo3/archive/gh-pages.tar.gz"
    response = requests.get(url, stream=True)
    response.raise_for_status()
    with tarfile.open(fileobj=io.BytesIO(response.content), mode="r:gz") as tar:
        tar.extractall()
    shutil.move("pyo3-gh-pages", "netlify_build")

    preview = "--preview" in session.posargs
    if preview:
        session.posargs.remove("--preview")

    _build_netlify_redirects(preview)

    session.install("towncrier")
    # Save a copy of the changelog to restore later
    changelog = (PYO3_DIR / "CHANGELOG.md").read_text()

    # Build the changelog
    session.run(
        "towncrier", "build", "--keep", "--version", "Unreleased", "--date", "TBC"
    )

    # Build the guide
    build_guide(session)
    PYO3_GUIDE_TARGET.rename("netlify_build/main")

    # Restore the original changelog
    (PYO3_DIR / "CHANGELOG.md").write_text(changelog)
    session.run("git", "restore", "--staged", "CHANGELOG.md", external=True)

    # Build the main branch docs
    docs(session)
    PYO3_DOCS_TARGET.rename("netlify_build/main/doc")

    Path("netlify_build/main/doc/index.html").write_text(
        "<meta http-equiv=refresh content=0;url=pyo3/>"
    )

    # Build the internal docs
    docs(session, nightly=True, internal=True)
    PYO3_DOCS_TARGET.rename("netlify_build/internal")


def _build_netlify_redirects(preview: bool) -> None:
    current_version = os.environ.get("PYO3_VERSION")

    with ExitStack() as stack:
        redirects_file = stack.enter_context(open("netlify_build/_redirects", "w"))
        headers_file = stack.enter_context(open("netlify_build/_headers", "w"))
        for d in glob("netlify_build/v*"):
            version = d.removeprefix("netlify_build/v")
            full_directory = d + "/"
            redirects_file.write(
                f"/v{version}/doc/* https://docs.rs/pyo3/{version}/:splat\n"
            )
            if version != current_version:
                # for old versions, mark the files in the latest version as the canonical URL
                for file in glob(f"{d}/**", recursive=True):
                    file_path = file.removeprefix(full_directory)
                    # remove index.html and/or .html suffix to match the page URL on the
                    # final netlfiy site
                    url_path = file_path
                    if file_path == "index.html":
                        url_path = ""

                    url_path = url_path.removesuffix(".html")

                    # if the file exists in the latest version, add a canonical
                    # URL as a header
                    for url in (
                        f"/v{version}/{url_path}",
                        *(
                            (f"/v{version}/{file_path}",)
                            if file_path != url_path
                            else ()
                        ),
                    ):
                        headers_file.write(url + "\n")
                        if os.path.exists(
                            f"netlify_build/v{current_version}/{file_path}"
                        ):
                            headers_file.write(
                                f'  Link: <https://pyo3.rs/v{current_version}/{url_path}>; rel="canonical"\n'
                            )
                        else:
                            # this file doesn't exist in the latest guide, don't
                            # index it
                            headers_file.write("  X-Robots-Tag: noindex\n")

        # Add latest redirect
        if current_version is not None:
            redirects_file.write(f"/latest/* /v{current_version}/:splat 302\n")

        # some backwards compatbiility redirects
        redirects_file.write(
            """\
/latest/building_and_distribution/* /latest/building-and-distribution/:splat 302
/latest/building_and_distribution/multiple_python_versions/* /latest/building-and-distribution/multiple-python-versions:splat 302
/latest/function/error_handling/* /latest/function/error-handling/:splat 302
/latest/getting_started/* /latest/getting-started/:splat 302
/latest/python_from_rust/* /latest/python-from-rust/:splat 302
/latest/python_typing_hints/* /latest/python-typing-hints/:splat 302
/latest/trait_bounds/* /latest/trait-bounds/:splat 302
"""
        )

        # Add landing page redirect
        if preview:
            redirects_file.write("/ /main/ 302\n")
        else:
            redirects_file.write(f"/ /v{current_version}/ 302\n")


@nox.session(name="check-guide")
def check_guide(session: nox.Session):
    # reuse other sessions, but with default args
    posargs = [*session.posargs]
    del session.posargs[:]
    build_guide(session)
    docs(session)
    session.posargs.extend(posargs)

    if toml is None:
        session.error("requires Python 3.11 or `toml` to be installed")
    pyo3_version = toml.loads((PYO3_DIR / "Cargo.toml").read_text())["package"][
        "version"
    ]

    remaps = {
        f"file://{PYO3_GUIDE_SRC}/([^/]*/)*?%7B%7B#PYO3_DOCS_URL}}}}": f"file://{PYO3_DOCS_TARGET}",
        f"https://pyo3.rs/v{pyo3_version}": f"file://{PYO3_GUIDE_TARGET}",
        "https://pyo3.rs/main/": f"file://{PYO3_GUIDE_TARGET}/",
        "https://pyo3.rs/latest/": f"file://{PYO3_GUIDE_TARGET}/",
        "%7B%7B#PYO3_DOCS_VERSION}}": "latest",
        # bypass fragments for edge cases
        # blob links
        "(https://github.com/[^/]+/[^/]+/blob/[^#]+)#[a-zA-Z0-9._-]*": "$1",
        # issue comments
        "(https://github.com/[^/]+/[^/]+/issues/[0-9]+)#issuecomment-[0-9]*": "$1",
        # rust docs
        "(https://docs.rs/[^#]+)#[a-zA-Z0-9._-]*": "$1",
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
        "--accept=200,429",
        *session.posargs,
        external=True,
    )
    # check external links in the docs
    # (intra-doc links are checked by rustdoc)
    _run(
        session,
        "lychee",
        str(PYO3_DOCS_TARGET),
        *remap_args,
        f"--exclude=file://{PYO3_DOCS_TARGET}",
        # exclude some old http links from copyright notices, known to fail
        "--exclude=http://www.adobe.com/",
        "--exclude=http://www.nhncorp.com/",
        "--accept=200,429",
        # reduce the concurrency to avoid rate-limit from `pyo3.rs`
        "--max-concurrency=32",
        *session.posargs,
        external=True,
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
    "ci",
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


@nox.session(name="set-msrv-package-versions", venv_backend="none")
def set_msrv_package_versions(session: nox.Session):
    from collections import defaultdict

    projects = (
        PYO3_DIR,
        *(Path(p).parent for p in glob("examples/*/Cargo.toml")),
        *(Path(p).parent for p in glob("pyo3-ffi/examples/*/Cargo.toml")),
    )
    min_pkg_versions = {}

    # run cargo update first to ensure that everything is at highest
    # possible version, so that this matches what CI will resolve to.
    for project in projects:
        _run_cargo(
            session,
            "+stable",
            "update",
            f"--manifest-path={project}/Cargo.toml",
            env=os.environ | {"CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS": "fallback"},
        )

        lock_file = project / "Cargo.lock"

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

        assert "3.16" not in PY_VERSIONS
        config_file.set("CPython", "3.16")
        _run_cargo(session, "check", env=env, expect_error=True)

        # 3.15 CPython should build if abi3 is explicitly requested
        _run_cargo(session, "check", "--features=pyo3/abi3", env=env)

        # 3.15 CPython should build with forward compatibility
        env["PYO3_USE_ABI3_FORWARD_COMPATIBILITY"] = "1"
        _run_cargo(session, "check", env=env)

        assert "3.10" not in PYPY_VERSIONS
        config_file.set("PyPy", "3.10")
        _run_cargo(session, "check", env=env, expect_error=True)

    # attempt to build with latest version and check that abi3 version
    # configured matches the feature
    max_minor_version = max(int(v.split(".")[1]) for v in PY_VERSIONS if "t" not in v)
    with tempfile.TemporaryFile() as stderr:
        env = os.environ.copy()
        env["PYO3_PRINT_CONFIG"] = "1"  # get diagnostics from the build
        env["PYO3_NO_PYTHON"] = "1"  # isolate the build from local Python
        _run_cargo(
            session,
            "check",
            f"--features=pyo3/abi3-py3{max_minor_version}",
            env=env,
            stderr=stderr,
            expect_error=True,
        )
        stderr.seek(0)
        stderr = stderr.read().decode()
    # NB if this assertion fails with something like
    # "An abi3-py3* feature must be specified when compiling without a Python
    # interpreter."
    #
    # then `ABI3_MAX_MINOR` in `pyo3-build-config/src/impl_.rs` is probably outdated.
    assert f"version=3.{max_minor_version}" in stderr, (
        f"Expected to see version=3.{max_minor_version}, got: \n\n{stderr}"
    )


@nox.session(name="check-feature-powerset", venv_backend="none")
def check_feature_powerset(session: nox.Session):
    if toml is None:
        session.error("requires Python 3.11 or `toml` to be installed")

    cargo_toml = toml.loads((PYO3_DIR / "Cargo.toml").read_text())

    # free-threaded builds do not support ABI3 (yet)
    EXPECTED_ABI3_FEATURES = {
        f"abi3-py3{ver.split('.')[1]}" for ver in PY_VERSIONS if not ver.endswith("t")
    }

    EXCLUDED_FROM_FULL = {
        "nightly",
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

    unexpected_abi3_features = abi3_version_features - EXPECTED_ABI3_FEATURES
    if unexpected_abi3_features:
        session.error(
            f"unexpected `abi3` features found in Cargo.toml: {unexpected_abi3_features}"
        )

    missing_abi3_features = EXPECTED_ABI3_FEATURES - abi3_version_features
    if missing_abi3_features:
        session.error(f"missing `abi3` features in Cargo.toml: {missing_abi3_features}")

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
        *(EXCLUDED_FROM_FULL),
        *abi3_version_features,
    ]

    # deny warnings
    env = os.environ.copy()
    rust_flags = env.get("RUSTFLAGS", "")
    env["RUSTFLAGS"] = f"{rust_flags} -Dwarnings"

    subcommand = "hack"
    if "minimal-versions" in session.posargs:
        subcommand = "minimal-versions"

    comma_join = ",".join
    _run_cargo(
        session,
        subcommand,
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
    _run_cargo(session, *command, "--features=abi3,full", env=env)


@nox.session(name="test-introspection")
def test_introspection(session: nox.Session):
    session.install("maturin")
    session.install("ruff")
    options = []
    target = os.environ.get("CARGO_BUILD_TARGET")
    if target is not None:
        options += ("--target", target)
    profile = os.environ.get("CARGO_BUILD_PROFILE")
    if profile == "release":
        options.append("--release")
    session.run_always(
        "maturin",
        "develop",
        "-m",
        "./pytests/Cargo.toml",
        "--features",
        "experimental-inspect",
        *options,
    )
    # We look for the built library
    lib_file = None
    for file in Path(session.virtualenv.location).rglob("pyo3_pytests.*"):
        if file.is_file():
            lib_file = str(file.resolve())
    _run_cargo_test(
        session,
        package="pyo3-introspection",
        env={"PYO3_PYTEST_LIB_PATH": lib_file},
    )


def _build_docs_for_ffi_check(session: nox.Session) -> None:
    # pyo3-ffi-check needs to scrape docs of pyo3-ffi
    env = os.environ.copy()
    env["PYO3_PYTHON"] = sys.executable
    _run_cargo(session, "doc", _FFI_CHECK, "-p", "pyo3-ffi", "--no-deps", env=env)


@lru_cache()
def _get_rust_info() -> Tuple[str, ...]:
    output = _get_output("rustc", "-vV")

    return tuple(output.splitlines())


def get_rust_version() -> Tuple[int, int, int, List[str]]:
    for line in _get_rust_info():
        if line.startswith(_RELEASE_LINE_START):
            version = line[len(_RELEASE_LINE_START) :].strip()
            # e.g. 1.67.0-beta.2
            (version_number, *extra) = version.split("-", maxsplit=1)
            return (*map(int, version_number.split(".")), extra)


def is_rust_nightly() -> bool:
    for line in _get_rust_info():
        if line.startswith(_RELEASE_LINE_START):
            return line.strip().endswith("-nightly")
    return False


def _get_rust_default_target() -> str:
    for line in _get_rust_info():
        if line.startswith(_HOST_LINE_START):
            return line[len(_HOST_LINE_START) :].strip()


@lru_cache()
def _get_feature_sets() -> Tuple[Optional[str], ...]:
    """Returns feature sets to use for Rust jobs"""
    cargo_target = os.getenv("CARGO_BUILD_TARGET", "")

    features = "full"

    if "wasm32-wasip1" not in cargo_target:
        # multiple-pymethods not supported on wasm
        features += ",multiple-pymethods"

    if is_rust_nightly():
        features += ",nightly"

    return (None, "abi3", features, f"abi3,{features}")


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
    env: Optional[Dict[str, str]] = None,
    extra_flags: Optional[List[str]] = None,
) -> None:
    command = ["cargo"]
    if "careful" in session.posargs:
        # do explicit setup so failures in setup can be seen
        _run_cargo(session, "careful", "setup")
        command.append("careful")

    command.extend(("test", "--no-fail-fast"))

    if "release" in session.posargs:
        command.append("--release")
    if package:
        command.append(f"--package={package}")
    if features:
        command.append(f"--features={features}")
    if extra_flags:
        command.extend(extra_flags)

    _run(session, *command, external=True, env=env or {})


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

    def set(
        self, implementation: str, version: str, build_flags: Iterable[str] = ()
    ) -> None:
        """Set the contents of this config file to the given implementation and version."""
        if version.endswith("t"):
            # Free threaded versions pass the support in config file through a flag
            version = version[:-1]
            build_flags = (*build_flags, "Py_GIL_DISABLED")

        self._config_file.seek(0)
        self._config_file.truncate(0)
        self._config_file.write(
            f"""\
implementation={implementation}
version={version}
build_flags={",".join(build_flags)}
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
