import time
from glob import glob

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
    session.install("black==21.12b0")
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
