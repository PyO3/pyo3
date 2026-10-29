import argparse

import nox
import nox_uv

nox.options.sessions = ["test"]


@nox_uv.session(venv_backend="uv", uv_extras=["dev"], uv_no_install_project=True)
def test(session: nox.Session):
    parser = argparse.ArgumentParser()
    parser.add_argument("--features")
    args = parser.parse_args(session.posargs)

    session.env["MATURIN_PEP517_ARGS"] = "--profile=dev"
    if args.features:
        session.env["MATURIN_PEP517_ARGS"] += f" --features={args.features}"
    session.install(".")
    session.run("pytest")


@nox_uv.session(venv_backend="uv", uv_extras=["dev"], uv_no_install_project=True)
def bench(session: nox.Session):
    parser = argparse.ArgumentParser()
    parser.add_argument("--features")
    args = parser.parse_args(session.posargs)

    if args.features:
        session.env["MATURIN_PEP517_ARGS"] = f" --features={args.features}"
    session.install(".")
    session.run("pytest", "--benchmark-enable")
