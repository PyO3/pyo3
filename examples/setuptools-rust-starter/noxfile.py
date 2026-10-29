import nox
import nox_uv


@nox_uv.session(uv_groups=["dev"])
def python(session: nox.Session):
    session.env["SETUPTOOLS_RUST_CARGO_PROFILE"] = "dev"
    session.run("pytest")
