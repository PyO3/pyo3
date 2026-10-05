import argparse
import sys

import nox


@nox.session
def python(session):
    if sys.version_info < (3, 10):
        session.skip("Meson 1.12 requires Python 3.10 or newer")

    parser = argparse.ArgumentParser()
    parser.add_argument("--features")
    args = parser.parse_args(session.posargs)

    settings = ["-Csetup-args=-Dbuildtype=debug"]
    if args.features:
        settings.append(f"-Csetup-args=-Dfeatures={args.features}")
    session.install(".[dev]", *settings)
    session.run("pytest")
