import argparse

import nox
import nox_uv


@nox_uv.session(uv_groups=["dev"])
def python(session: nox.Session):
    parser = argparse.ArgumentParser()
    parser.add_argument("--features")
    args = parser.parse_args(session.posargs)

    session.env["MATURIN_PEP517_ARGS"] = "--profile=dev"
    if args.features:
        session.env["MATURIN_PEP517_ARGS"] += f" --features={args.features}"
    session.run("pytest")
