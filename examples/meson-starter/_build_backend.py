"""Custom build-backend to workaround in-tree PyO3 dependency.

Issues were observed with the PyO3 crates existing in a path dependency
but a separate cargo workspace.

The "solution" for now is to use `cargo package` to manually place the PyO3
crates into meson's `subprojects` directory.

FIXME: investigate and remove the need for this wrapper.
"""

import json
import os
import shutil
import subprocess
import tempfile
from contextlib import contextmanager
from pathlib import Path

import mesonpy

try:
    import tomllib
except ImportError:
    import tomli as tomllib

get_requires_for_build_wheel = mesonpy.get_requires_for_build_wheel

MESON_STARTER_DIR = Path(__file__).resolve().parent
PYO3_DIR = MESON_STARTER_DIR.parent.parent
CRATES = (
    "pyo3",
    "pyo3-ffi",
    "pyo3-macros",
    "pyo3-macros-backend",
)


# See top-level doc; this function stages the PyO3 crates into the `subprojects` directory
# so that meson can build them as part of the example.
#
# A real-world project would not need to do this, as it would depend on PyO3 from crates.io.
def _stage_crates():
    manifest = PYO3_DIR / "Cargo.toml"

    subprojects = MESON_STARTER_DIR / "subprojects"
    subprojects.mkdir(exist_ok=True)

    # Clean up existing staged crates
    for path in subprojects.glob("pyo3-*"):
        if path.is_dir() and not path.is_symlink():
            shutil.rmtree(path)
        else:
            path.unlink()

    # Generate `Cargo.lock` from the repository and copy it to the example directory;
    # meson needs a lockfile to know what transitive dependencies to download and build.
    metadata = json.loads(
        subprocess.check_output(
            [
                "cargo",
                "metadata",
                "--manifest-path",
                str(manifest),
                "--format-version=1",
            ],
            cwd=PYO3_DIR,
        )
    )
    shutil.copyfile(PYO3_DIR / "Cargo.lock", MESON_STARTER_DIR / "Cargo.lock")

    packages = {
        package["name"]: package
        for package in metadata["packages"]
        if package["source"] is None and package["name"] in CRATES
    }

    # Stage all the PyO3 crates into the `subprojects` directory so that meson can build them.
    with tempfile.TemporaryDirectory(prefix="pyo3-meson-") as directory:
        target = Path(directory)
        subprocess.run(
            [
                "cargo",
                "package",
                "--manifest-path",
                str(manifest),
                "--no-verify",
                "--allow-dirty",
                "--exclude-lockfile",
                "--target-dir",
                str(target),
                *(argument for name in CRATES for argument in ("-p", name)),
            ],
            cwd=PYO3_DIR,
            check=True,
        )

        for name in CRATES:
            version = packages[name]["version"]
            major, minor, *_ = version.split(".")
            api = major if major != "0" else f"0.{minor}"
            dependency = f"{name}-{api}-rs"
            crate = f"{name}-{version}"
            shutil.unpack_archive(
                target / "package" / f"{crate}.crate",
                subprojects,
                format="gztar",
                filter="data",
            )
            (subprojects / f"{dependency}.wrap").write_text(
                f"[wrap-file]\ndirectory = {crate}\nmethod = cargo\n"
                f"\n[provide]\ndependency_names = {dependency}\n"
            )


# FIXME: this is a workaround for meson not currently setting
# `CARGO_PKG_VERSION`, which PyO3's crates use via `env!` in several places.
#
# This workaround just sets a single env var globally for the compile, which
# is technically wrong but works enough for testing for now.
#
# https://github.com/mesonbuild/meson/pull/15837 should hopefully remove the
# need for this hack.
@contextmanager
def _cargo_environment():
    lockfile = tomllib.loads((MESON_STARTER_DIR / "Cargo.lock").read_text())
    version = next(
        package["version"]
        for package in lockfile["package"]
        if package["name"] == "pyo3" and "source" not in package
    )

    # Until mesonbuild/meson#15837 supplies these variables per crate, this is
    # sufficient for this example: only its PyO3 crates use CARGO_PKG_VERSION.
    previous = os.environ.get("CARGO_PKG_VERSION")
    os.environ["CARGO_PKG_VERSION"] = version
    try:
        yield
    finally:
        if previous is None:
            os.environ.pop("CARGO_PKG_VERSION", None)
        else:
            os.environ["CARGO_PKG_VERSION"] = previous


def build_wheel(wheel_directory, config_settings=None, metadata_directory=None):
    _stage_crates()
    with _cargo_environment():
        return mesonpy.build_wheel(wheel_directory, config_settings, metadata_directory)
