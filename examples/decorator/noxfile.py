import argparse

import nox


@nox.session
def python(session: nox.Session):
    parser = argparse.ArgumentParser()
    parser.add_argument("--features")
    args = parser.parse_args(session.posargs)

    session.env["MATURIN_PEP517_ARGS"] = "--profile=dev"
    if args.features:
        session.env["MATURIN_PEP517_ARGS"] += f" --features={args.features}"

    session.run_install(
        "uv",
        "sync",
        "--group=dev",
        "--no-default-groups",
        env={
            "UV_PROJECT_ENVIRONMENT": session.virtualenv.location,
            "UV_PYTHON": session.virtualenv.location,
        },
    )

    session.run("pytest")
