import time
from glob import glob

import nox

nox.options.sessions = ["test", "clippy", "fmt"]


@nox.session(venv_backend="none")
def test(session: nox.Session):
    test_rust(session)
    test_py(session)


@nox.session(name="test-rust", venv_backend="none")
def test_rust(session: nox.Session):
    session.run("cargo", "test", external=True)
    session.run("cargo", "test", "--features=abi3", external=True)
    session.run("cargo", "test", "--features=full", external=True)
    session.run("cargo", "test", "--features=abi3 full", external=True)


@nox.session(name="test-py", venv_backend="none")
def test_py(session):
    session.run("nox", "-f", "pytests/noxfile.py", external=True)
    for example in glob("examples/*/noxfile.py"):
        session.run("nox", "-f", example, external=True)


@nox.session
def fmt(session: nox.Session):
    fmt_rust(session)
    fmt_py(session)


@nox.session(name="fmt-rust", venv_backend="none")
def fmt_rust(session: nox.Session):
    session.run("cargo", "fmt", "--all", "--check", external=True)


@nox.session(name="fmt-py")
def fmt_py(session: nox.Session):
    session.install("black==22.3.0")
    session.run("black", ".", "--check")


@nox.session(venv_backend="none")
def clippy(session: nox.Session) -> None:
    for feature_set in ["full", "abi3 full"]:
        session.run(
            "cargo",
            "clippy",
            f"--features={feature_set}",
            "--all-targets",
            "--workspace",
            "--",
            "--deny=warnings",
            *session.posargs,
            external=True,
        )


@nox.session(venv_backend="none")
def publish(session: nox.Session) -> None:
    session.run(
        "cargo",
        "publish",
        "--manifest-path",
        "pyo3-build-config/Cargo.toml",
        external=True,
    )
    time.sleep(10)
    session.run(
        "cargo",
        "publish",
        "--manifest-path",
        "pyo3-macros-backend/Cargo.toml",
        external=True,
    )
    time.sleep(10)
    session.run(
        "cargo", "publish", "--manifest-path", "pyo3-macros/Cargo.toml", external=True
    )
    time.sleep(10)
    session.run(
        "cargo", "publish", "--manifest-path", "pyo3-ffi/Cargo.toml", external=True
    )
    time.sleep(10)
    session.run("cargo", "publish", external=True)


@nox.session(venv_backend="none")
def contributors(session: nox.Session) -> None:
    import requests

    if len(session.posargs) != 1:
        raise Exception("base commit positional argument missing")

    base = session.posargs[0]
    page = 1

    authors = set()

    while True:
        resp = requests.get(
            f"https://api.github.com/repos/PyO3/pyo3/compare/{base}...HEAD",
            params={"page": page, "per_page": 100},
        )

        body = resp.json()

        if resp.status_code != 200:
            raise Exception(
                f"failed to retrieve commits: {resp.status_code} {body['message']}"
            )

        for commit in body["commits"]:
            try:
                authors.add(commit["author"]["login"])
            except:
                continue

        if "next" in resp.links:
            page += 1
        else:
            break

    authors = sorted(list(authors))

    for author in authors:
        print(f"@{author}")
