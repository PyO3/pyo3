import nox

nox.options.sessions = ["test"]


@nox.session
@nox.parametrize("cargo_features", ['""', "pep489"])
def test(session, cargo_features):
    session.install("-rrequirements-dev.txt")
    session.install("maturin")
    session.run_always(
        "maturin", "develop", f"--cargo-extra-args=--features {cargo_features}"
    )
    session.run("pytest")


@nox.session
def bench(session):
    session.install("-rrequirements-dev.txt")
    session.install(".")
    session.run("pytest", "--benchmark-enable")
