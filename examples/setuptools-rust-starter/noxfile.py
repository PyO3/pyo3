import nox


@nox.session
def python(session: nox.Session):
    session.env["SETUPTOOL_RUST_CARGO_PROFILE"] = "dev"
    session.install(".[dev]")
    session.run("pytest")
