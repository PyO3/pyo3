import argparse
import shutil
import sys
from collections.abc import Iterable
from pathlib import Path

import nox
from nox.command import CommandFailed

nox.options.sessions = ["test"]
nox.options.default_venv_backend = "uv"


def uv_sync(session: nox.Session, groups: Iterable[str] = []):
    cmd = ["uv", "sync", "--locked", "--no-default-groups"]
    cmd.extend([f"--group={g}" for g in groups])
    session.run_install(
        *cmd,
        env={
            "UV_PROJECT_ENVIRONMENT": session.virtualenv.location,
            "UV_PYTHON": session.virtualenv.location,
        },
    )


@nox.session
def test(session: nox.Session):
    parser = argparse.ArgumentParser()
    parser.add_argument("--features")
    (args, unhandled_posargs) = parser.parse_known_args(session.posargs)

    session.env["MATURIN_PEP517_ARGS"] = "--profile=dev"
    if args.features:
        session.env["MATURIN_PEP517_ARGS"] += f" --features={args.features}"

    uv_sync(session, ["dev"])

    def try_install_binary(package: str, constraint: str):
        try:
            session.install("--only-binary=:all:", f"{package}{constraint}")
        except CommandFailed:
            # No binary wheel available on this platform
            pass

    try_install_binary("numpy", ">=1.16")
    try_install_binary("gevent", ">=22.10.2")
    # hypothesis itself depends on PyO3 so newer Python versions may fail
    # to build
    try_install_binary("hypothesis", ">=6.156.1")
    # The stubs are generated from a build with `experimental-async,experimental-inspect`,
    # so only such a build exports the members they declare. The `test-introspection`
    # session runs this test against one.
    ignored_paths = ["tests/test_stubs.py"]
    if sys.version_info < (3, 10):
        # Match syntax is only available in Python >= 3.10
        ignored_paths.append("tests/test_enums_match.py")
    ignore_args = [f"--ignore={path}" for path in ignored_paths]

    extra_args = []
    # https://github.com/oracle/graalpython/issues/1211
    if sys.implementation.name == "graalpy":
        extra_args.append("--assert=plain")

    session.run("pytest", *extra_args, *ignore_args, *unhandled_posargs)


@nox.session
def bench(session: nox.Session):
    uv_sync(session, ["dev"])
    session.run("pytest", "--benchmark-enable", "--benchmark-only", *session.posargs)


@nox.session
def mypy(session: nox.Session):
    type_checker(
        session,
        ("mypy", "tests"),
    )
    # TODO: enable stubtest session.run_always("python", "-m", "mypy.stubtest", "pyo3_pytests")


@nox.session
def pyrefly(session: nox.Session):
    type_checker(
        session,
        ("pyrefly", "check", "--search-path", ".", "tests"),
    )


def type_checker(session: nox.Session, command: Iterable[str]):
    session.env["MATURIN_PEP517_ARGS"] = "--profile=dev"
    uv_sync(session, ["dev"])
    try:
        # We move the stubs where maturin is expecting them to be
        shutil.copytree("stubs", "pyo3_pytests")
        (Path("pyo3_pytests") / "py.typed").touch()

        session.run_always(*command)
    finally:
        shutil.rmtree("pyo3_pytests")
