import argparse
from collections.abc import Iterable

import nox

nox.options.sessions = ["test"]


def uv_sync(session: nox.Session, groups: Iterable[str] = ()):
    cmd = ["uv", "sync", "--locked", "--no-default-groups"]
    cmd.extend([f"--group={g}" for g in groups])
    session.run_install(
        *cmd,
        env={
            "UV_PROJECT_ENVIRONMENT": session.virtualenv.location,
            "UV_PYTHON": session.virtualenv.location,
        },
    )


@nox.session
def test(session: nox.Session):
    parser = argparse.ArgumentParser()
    parser.add_argument("--features")
    args = parser.parse_args(session.posargs)

    session.env["MATURIN_PEP517_ARGS"] = "--profile=dev"
    if args.features:
        session.env["MATURIN_PEP517_ARGS"] += f" --features={args.features}"

    uv_sync(session, groups=["dev"])
    session.run("pytest")


@nox.session
def bench(session: nox.Session):
    parser = argparse.ArgumentParser()
    parser.add_argument("--features")
    args = parser.parse_args(session.posargs)

    if args.features:
        session.env["MATURIN_PEP517_ARGS"] = f" --features={args.features}"

    uv_sync(session, groups=["dev"])
    session.run("pytest", "--benchmark-enable")
