import nox
import sys


@nox.session
def python(session: nox.Session):
    if sys.version_info < (3, 9):
        session.skip("Python 3.9 or later is required for setuptools-rust 1.11")
    session.env["SETUPTOOLS_RUST_CARGO_PROFILE"] = "dev"
    session.install(".[dev]")
    session.run("pytest")
