import nox


@nox.session
def python(session: nox.Session):
    session.install("-rrequirements-dev.txt")
    session.run_always(
        "pip", "install", "-e", ".", "--no-build-isolation", env={"BUILD_DEBUG": "1"}
    )
    session.run("pytest")
