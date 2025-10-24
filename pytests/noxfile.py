import nox
import sys
from nox.command import CommandFailed

nox.options.sessions = ["test"]


@nox.session
def test(session: nox.Session):
    session.env["MATURIN_PEP517_ARGS"] = "--profile=dev"
    session.install("-v", ".[dev]")

    def try_install_binary(package: str, constraint: str):
        try:
            session.install("--only-binary=:all:", f"{package}{constraint}")
        except CommandFailed:
            # No binary wheel available on this platform
            pass

    try_install_binary("numpy", ">=1.16")
    # https://github.com/zopefoundation/zope.interface/issues/316
    # - is a dependency of gevent
    try_install_binary("zope.interface", "<7")
    try_install_binary("gevent", ">=22.10.2")
    ignored_paths = []
    if sys.version_info < (3, 10):
        # Match syntax is only available in Python >= 3.10
        ignored_paths.append("tests/test_enums_match.py")
    ignore_args = [f"--ignore={path}" for path in ignored_paths]
    session.run("pytest", *ignore_args, *session.posargs)


@nox.session
def bench(session: nox.Session):
    session.install(".[dev]")
    session.run("pytest", "--benchmark-enable", "--benchmark-only", *session.posargs)


@nox.session
def build_guide(session: nox.Session):
    """Build the mdBook guide for all languages"""
    # Build main guide (English)
    session.run("mdbook", "build", "guide", external=True)
    # Build Chinese guide if it exists
    try:
        session.run("mdbook", "build", "guide/cn", external=True)
    except CommandFailed:
        print("Chinese guide build failed or doesn't exist, continuing...")


@nox.session
def check_guide(session: nox.Session):
    """Build and check links in the mdBook guide"""
    # Build all guides first
    session.run("mdbook", "build", "guide", external=True)
    try:
        session.run("mdbook", "build", "guide/cn", external=True)
    except CommandFailed:
        print("Chinese guide build failed or doesn't exist, continuing...")

    # Run lychee link checker on the built output
    session.run(
        "lychee",
        "--include-fragments",
        "target/guide/",
        "--remap",
        "file://target/guide/=https://pyo3.rs/",
        "--remap",
        "file://target/guide/cn/=https://pyo3.rs/cn/",
        "--accept=200,429",
        "--exclude-path",
        "target/guide/doc/",
        external=True,
    )


@nox.session
def ruff(session: nox.Session):
    """Check code formatting and linting with ruff"""
    session.install("ruff")

    # Run ruff format check
    session.run("ruff", "format", ".", "--check")

    # Run ruff linting
    session.run("ruff", "check", ".")
    """Build the complete Netlify site"""
    session.install("requests", "towncrier")

    # Run the netlify build script
    session.run("bash", ".netlify/build.sh", *session.posargs, external=True)
