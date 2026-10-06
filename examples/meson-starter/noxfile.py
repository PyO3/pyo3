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

    features = args.features.split(",") if args.features else []
    # replace `pyo3/hashbrown` with `hashbrown` and similar for `parking_lot`;
    # meson doesn't support dependency features in the way that cargo does.
    features = [
        feature[len("pyo3/") :]
        for feature in features
        if feature in ("pyo3/hashbrown", "pyo3/parking_lot")
    ]
    features = ",".join(features)

    settings = ["-Csetup-args=-Dbuildtype=debug"]
    if features:
        settings.append(f"-Csetup-args=-Dfeatures={features}")
    session.install(".[dev]", *settings)
    session.run("pytest")
