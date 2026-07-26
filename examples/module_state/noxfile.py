import nox


@nox.session
def test(session):
    """Run tests for module state example (once implemented)."""
    session.install("pytest")
    session.run("pytest", "tests/", "-v")


@nox.session
def build(session):
    """Build the module."""
    session.install("maturin")
    session.run("maturin", "develop")


@nox.session
def dev(session):
    """Development session with build and test."""
    session.run("maturin", "develop", external=True)
    session.install("pytest")
    session.run("pytest", "tests/", "-v")
