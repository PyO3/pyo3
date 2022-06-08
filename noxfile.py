import re
import sys
import time
from glob import glob
from pathlib import Path

import nox

nox.options.sessions = ["test", "clippy", "fmt"]


@nox.session(venv_backend="none")
def test(session: nox.Session):
    test_rust(session)
    test_py(session)


@nox.session(name="test-rust", venv_backend="none")
def test_rust(session: nox.Session):
    session.run("cargo", "test", external=True)
    session.run("cargo", "test", "--features=abi3", external=True)
    session.run("cargo", "test", "--features=full", external=True)
    session.run("cargo", "test", "--features=abi3 full", external=True)


@nox.session(name="test-py", venv_backend="none")
def test_py(session):
    session.run("nox", "-f", "pytests/noxfile.py", external=True)
    for example in glob("examples/*/noxfile.py"):
        session.run("nox", "-f", example, external=True)


@nox.session
def fmt(session: nox.Session):
    fmt_rust(session)
    fmt_py(session)


@nox.session(name="fmt-rust", venv_backend="none")
def fmt_rust(session: nox.Session):
    session.run("cargo", "fmt", "--all", "--check", external=True)


@nox.session(name="fmt-py")
def fmt_py(session: nox.Session):
    session.install("black==22.3.0")
    session.run("black", ".", "--check")


@nox.session(venv_backend="none")
def clippy(session: nox.Session) -> None:
    for feature_set in ["full", "abi3 full"]:
        session.run(
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
    session.run(
        "cargo",
        "publish",
        "--manifest-path",
        "pyo3-build-config/Cargo.toml",
        external=True,
    )
    time.sleep(10)
    session.run(
        "cargo",
        "publish",
        "--manifest-path",
        "pyo3-macros-backend/Cargo.toml",
        external=True,
    )
    time.sleep(10)
    session.run(
        "cargo", "publish", "--manifest-path", "pyo3-macros/Cargo.toml", external=True
    )
    time.sleep(10)
    session.run(
        "cargo", "publish", "--manifest-path", "pyo3-ffi/Cargo.toml", external=True
    )
    time.sleep(10)
    session.run("cargo", "publish", external=True)


@nox.session(venv_backend="none")
def contributors(session: nox.Session) -> None:
    import requests

    if len(session.posargs) != 1:
        raise Exception("base commit positional argument missing")

    base = session.posargs[0]
    page = 1

    authors = set()

    while True:
        resp = requests.get(
            f"https://api.github.com/repos/PyO3/pyo3/compare/{base}...HEAD",
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
        rootdir = Path(__file__).parent
        self.emscripten_dir = rootdir / "emscripten"
        self.builddir = rootdir / ".nox/emscripten"
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
    session.run(
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
        ]
    )
    session.env["CARGO_BUILD_TARGET"] = target
    session.env["PYO3_CROSS_LIB_DIR"] = pythonlibdir
    session.run("rustup", "target", "add", target, "--toolchain", "stable")
    session.run(
        "bash", "-c", f"source {info.builddir/'emsdk/emsdk_env.sh'} && cargo test"
    )
