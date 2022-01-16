import nox


@nox.session
def python(session):
    session.install("-rrequirements-dev.txt")
    session.install(".")
    session.run("pytest")
