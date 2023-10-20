import nox

nox.options.sessions = ["test"]


@nox.session
def test(session: nox.Session):
    session.env["MATURIN_PEP517_ARGS"] = "--profile=dev"
    session.install(".[dev]")
    session.run("pytest")


@nox.session
def bench(session: nox.Session):
    session.env["MATURIN_PEP517_ARGS"] = "--profile=dev"
    session.install(".[dev]")
    session.run("pytest", "--benchmark-enable")
