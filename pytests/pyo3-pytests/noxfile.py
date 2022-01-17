import nox


@nox.session
def python(session):
    session.install("-rrequirements-dev.txt")
    session.install(".", "--no-build-isolation")
    session.run("pytest")
