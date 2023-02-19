import nox


@nox.session
def python(session):
    session.install("-rrequirements-dev.txt")
    session.install("maturin")
    session.run_always("maturin", "develop", "--features", "native-module")
    session.run("pytest")
