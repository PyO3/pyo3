import nox


@nox.session
def python(session: nox.Session):
    session.env["SETUPTOOLS_RUST_CARGO_PROFILE"] = "dev"

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
