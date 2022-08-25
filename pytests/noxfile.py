import nox
import platform

nox.options.sessions = ["test"]


@nox.session
def test(session):
    session.install("-rrequirements-dev.txt")
    if platform.system() == "Linux" and platform.python_implementation() == "CPython":
        session.install("numpy>=1.16")
    session.install("maturin")
    session.run_always("maturin", "develop")
    session.run("pytest", *session.posargs)


@nox.session
def bench(session):
    session.install("-rrequirements-dev.txt")
    session.install(".")
    session.run("pytest", "--benchmark-enable", "--benchmark-only", *session.posargs)
