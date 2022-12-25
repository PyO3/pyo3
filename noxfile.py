import json
import os
import re
import subprocess
import sys
import tempfile
import time
from glob import glob
from pathlib import Path
from typing import Any, Dict, List, Optional

import nox

nox.options.sessions = ["test", "clippy", "fmt"]


PYO3_DIR = Path(__file__).parent


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
    _run_cargo_test(session, features="abi3")
    if not "skip-full" in session.posargs:
        _run_cargo_test(session, features="full")
        _run_cargo_test(session, features="abi3 full")


@nox.session(name="test-py", venv_backend="none")
def test_py(session: nox.Session) -> None:
    _run(session, "nox", "-f", "pytests/noxfile.py", external=True)
    for example in glob("examples/*/noxfile.py"):
        _run(session, "nox", "-f", example, external=True)


@nox.session(venv_backend="none")
def coverage(session: nox.Session) -> None:
    session.env.update(_get_coverage_env())
    _run(session, "cargo", "llvm-cov", "clean", "--workspace", external=True)
    test(session)
    _run(
        session,
        "cargo",
        "llvm-cov",
        "--package=pyo3",
        "--package=pyo3-build-config",
        "--package=pyo3-macros-backend",
        "--package=pyo3-macros",
        "--package=pyo3-ffi",
        "report",
        "--lcov",
        "--output-path",
        "coverage.lcov",
        external=True,
    )


@nox.session
def fmt(session: nox.Session):
    fmt_rust(session)
    fmt_py(session)


@nox.session(name="fmt-rust", venv_backend="none")
def fmt_rust(session: nox.Session):
    _run(session, "cargo", "fmt", "--all", "--check", external=True)


@nox.session(name="fmt-py")
def fmt_py(session: nox.Session):
    session.install("black==22.3.0")
    _run(session, "black", ".", "--check")


@nox.session(venv_backend="none")
def clippy(session: nox.Session) -> None:
    for feature_set in ["full", "abi3 full"]:
        _run(
            session,
            "cargo",
            "clippy",
            f"--features={feature_set}",
            "--all-targets",
            "--workspace",
            "--",
            "--deny=warnings",
            external=True,
        )


@nox.session(venv_backend="none")
def publish(session: nox.Session) -> None:
    _run_cargo_publish(session, package="pyo3-build-config")
    time.sleep(10)
    _run_cargo_publish(session, package="pyo3-macros-backend")
    time.sleep(10)
    _run_cargo_publish(session, package="pyo3-macros")
    time.sleep(10)
    _run_cargo_publish(session, package="pyo3-ffi")
    time.sleep(10)
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
            except:
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


@nox.session(name="build-guide", venv_backend="none")
def build_guide(session: nox.Session):
    _run(session, "mdbook", "build", "-d", "../target/guide", "guide", *session.posargs)


@nox.session(name="format-guide", venv_backend="none")
def format_guide(session: nox.Session):
    fence_line = "//! ```\n"

    for path in Path("guide").glob("**/*.md"):
        session.log("Working on %s", path)
        content = path.read_text()

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
    _run(
        session,
        "cargo",
        "+nightly",
        "test",
        "--release",
        "-Zbuild-std",
        f"--target={_get_rust_target()}",
        "--",
        "--test-threads=1",
        env={
            "RUSTFLAGS": "-Zsanitizer=address",
            "RUSTDOCFLAGS": "-Zsanitizer=address",
            "ASAN_OPTIONS": "detect_leaks=0",
        },
        external=True,
    )


@nox.session(name="check-changelog")
def check_changelog(session: nox.Session):
    event_path = os.environ.get("GITHUB_EVENT_PATH")
    if event_path is None:
        session.error("Can only check changelog on github actions")

    with open(event_path) as event_file:
        event = json.load(event_file)

    if event["pull_request"]["title"].startswith("release:"):
        session.skip("PR title starts with release")

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
            "Changelog entry not found, please add one (or more) to `newsfragments` directory. For more information see https://github.com/PyO3/pyo3/blob/main/Contributing.md#documenting-changes"
        )

    print("Found newsfragments:")
    for fragment in fragments:
        print(fragment.name)


@nox.session(name="set-minimal-package-versions")
def set_minimal_package_versions(session: nox.Session):
    # run cargo update first to ensure that everything is at highest
    # possible version, so that this matches what CI will resolve to.
    _run(session, "cargo", "update", external=True)

    _run_cargo_set_package_version(session, "indexmap", "1.6.2")
    _run_cargo_set_package_version(session, "hashbrown:0.12.3", "0.9.1")
    _run_cargo_set_package_version(session, "plotters", "0.3.1")
    _run_cargo_set_package_version(session, "plotters-svg", "0.3.1")
    _run_cargo_set_package_version(session, "plotters-backend", "0.3.2")
    _run_cargo_set_package_version(session, "bumpalo", "3.10.0")
    _run_cargo_set_package_version(session, "once_cell", "1.14.0")
    _run_cargo_set_package_version(session, "rayon", "1.5.3")
    _run_cargo_set_package_version(session, "rayon-core", "1.9.3")

    # 1.15.0 depends on hermit-abi 0.2.6 which has edition 2021 and breaks 1.48.0
    _run_cargo_set_package_version(session, "num_cpus", "1.14.0")
    _run_cargo_set_package_version(
        session, "num_cpus", "1.14.0", project="examples/word-count"
    )

    projects = (
        None,
        "examples/decorator",
        "examples/maturin-starter",
        "examples/setuptools-rust-starter",
        "examples/word-count",
    )
    for project in projects:
        _run_cargo_set_package_version(
            session, "parking_lot", "0.11.0", project=project
        )
        _run_cargo_set_package_version(session, "once_cell", "1.14.0", project=project)

    _run_cargo_set_package_version(
        session, "rayon", "1.5.3", project="examples/word-count"
    )
    _run_cargo_set_package_version(
        session, "rayon-core", "1.9.3", project="examples/word-count"
    )

    # As a smoke test, cargo metadata solves all dependencies, so
    # will break if any crates rely on cargo features not
    # supported on MSRV
    _run(session, "cargo", "metadata", silent=True, external=True)


def _get_rust_target() -> str:
    output = _get_output("rustc", "-vV")

    for line in output.splitlines():
        if line.startswith(_HOST_LINE_START):
            return line[len(_HOST_LINE_START) :].strip()


_HOST_LINE_START = "host: "


def _get_coverage_env() -> Dict[str, str]:
    env = {}
    output = _get_output("cargo", "llvm-cov", "show-env")

    for line in output.strip().splitlines():
        (key, value) = line.split("=", maxsplit=1)
        env[key] = value.strip('"')

    # Ensure that examples/ and pytests/ all build to the correct target directory to collect
    # coverage artifacts.
    env["CARGO_TARGET_DIR"] = env["CARGO_LLVM_COV_TARGET_DIR"]

    return env


def _run(session: nox.Session, *args: str, **kwargs: Any) -> None:
    """Wrapper for _run(session, which creates nice groups on GitHub Actions."""
    if "GITHUB_ACTIONS" in os.environ:
        # Insert ::group:: at the start of nox's command line output
        print("::group::", end="", flush=True, file=sys.stderr)
    session.run(*args, **kwargs)
    if "GITHUB_ACTIONS" in os.environ:
        print("::endgroup::", file=sys.stderr)


def _run_cargo_test(
    session: nox.Session,
    *,
    package: Optional[str] = None,
    features: Optional[str] = None,
) -> None:
    command = ["cargo"]
    if "careful" in session.posargs:
        command.append("careful")
    command.append("test")
    if "release" in session.posargs:
        command.append("--release")
    if package:
        command.append(f"--package={package}")
    if features:
        command.append(f"--features={features}")

    _run(session, *command, external=True)


def _run_cargo_publish(session: nox.Session, *, package: str) -> None:
    _run(session, "cargo", "publish", f"--package={package}", external=True)


def _run_cargo_set_package_version(
    session: nox.Session,
    package: str,
    version: str,
    *,
    project: Optional[str] = None,
) -> None:
    command = ["cargo", "update", "-p", package, "--precise", version]
    if project:
        command.append(f"--manifest-path={project}/Cargo.toml")
    _run(session, *command, external=True)


def _get_output(*args: str) -> str:
    return subprocess.run(args, capture_output=True, text=True, check=True).stdout
