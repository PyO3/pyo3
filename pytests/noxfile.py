import nox
import sys
from nox.command import CommandFailed

nox.options.sessions = ["test"]


@nox.session
def test(session: nox.Session):
    session.env["MATURIN_PEP517_ARGS"] = "--profile=dev"
    session.run_always("python", "-m", "pip", "install", "-v", ".[dev]")

    def try_install_binary(package: str, constraint: str):
        try:
            session.install(f"--only-binary={package}", f"{package}{constraint}")
        except CommandFailed:
            # No binary wheel available on this platform
            pass

    try_install_binary("numpy", ">=1.16")
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
