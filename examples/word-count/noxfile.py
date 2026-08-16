import argparse

import nox

nox.options.sessions = ["test"]


@nox.session
def test(session: nox.Session):
    parser = argparse.ArgumentParser()
    parser.add_argument("--features")
    args = parser.parse_args(session.posargs)

    session.env["MATURIN_PEP517_ARGS"] = "--profile=dev"
    if args.features:
        session.env["MATURIN_PEP517_ARGS"] += f" --features={args.features}"
    session.install(".[dev]")
    session.run("pytest")


@nox.session
def bench(session: nox.Session):
    parser = argparse.ArgumentParser()
    parser.add_argument("--features")
    args = parser.parse_args(session.posargs)

    if args.features:
        session.env["MATURIN_PEP517_ARGS"] = f" --features={args.features}"
    session.install(".[dev]")
    session.run("pytest", "--benchmark-enable")
