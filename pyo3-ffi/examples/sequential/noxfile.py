import sys
import nox


@nox.session
def python(session):
    if sys.version_info < (3, 12):
        session.skip("Python 3.12+ is required")
    session.env["MATURIN_PEP517_ARGS"] = "--profile=dev"
    session.install(".[dev]")
    session.run("pytest")
