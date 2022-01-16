import nox


@nox.session
def python(session):
    session.install("-rrequirements-dev.txt")
    session.install("-e", ".")
    session.run("pytest", "--benchmark-sort=name")
