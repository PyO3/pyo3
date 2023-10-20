import nox


@nox.session
def python(session):
    session.env["MATURIN_PEP517_ARGS"] = "--profile=dev"
    session.install(".[dev]")
    session.run("pytest")
