import nox


@nox.session
@nox.parametrize("cargo_features", ['""', "pep489"])
def python(session, cargo_features):
    session.install("-rrequirements-dev.txt")
    session.install("maturin")
    session.run_always(
        "maturin", "develop", f"--cargo-extra-args=--features {cargo_features}"
    )
    session.run("pytest")
