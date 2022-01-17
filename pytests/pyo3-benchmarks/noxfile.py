import nox


@nox.session
def python(session):
    session.install("-rrequirements-dev.txt")
    session.install("-e", ".", "--no-build-isolation")
    session.run("pytest", "--benchmark-sort=name")
