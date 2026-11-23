import argparse
import struct
import sys

import nox


@nox.session
def python(session):
    if sys.version_info < (3, 10):
        session.skip("Meson 1.12 requires Python 3.10 or newer")

    if sys.platform == "win32" and struct.calcsize("P") == 4:
        # TODO: could implement this with some effort
        session.skip("Windows x86 requires Meson cross-compilation configuration")

    # FIXME: this can be dropped if meson supports path dependencies properly
    # (along with _build_backend.py)
    rustc_version = session.run("rustc", "--version", silent=True).split()[1]
    if rustc_version < "1.87.0":
        session.skip(
            "_build_backend.py requires Rust 1.87 or newer for `cargo package --exclude-lockfile`"
        )

    parser = argparse.ArgumentParser()
    parser.add_argument("--features")
    args = parser.parse_args(session.posargs)

    features = args.features.split(",") if args.features else []
    # replace `pyo3/hashbrown` with `hashbrown` and similar for `parking_lot`;
    # meson doesn't support dependency features in the way that cargo does.
    features = [
        feature[len("pyo3/") :]
        if feature in ("pyo3/hashbrown", "pyo3/parking_lot")
        else feature
        for feature in features
    ]
    features = ",".join(features)

    settings = ["-Csetup-args=-Dbuildtype=debug"]
    if features:
        settings.append(f"-Csetup-args=-Dfeatures={features}")
    session.install(".[dev]", *settings)
    session.run("pytest")
