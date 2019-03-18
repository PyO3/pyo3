import sys
import platform

from setuptools import setup
from setuptools.command.test import test as TestCommand
from setuptools_rust import RustExtension


class PyTest(TestCommand):
    user_options = []

    def run(self):
        self.run_command("test_rust")

        import subprocess

        errno = subprocess.call(["pytest", "tests"])
        raise SystemExit(errno)


def get_py_version_cfgs():
    # For now each Cfg Py_3_X flag is interpreted as "at least 3.X"
    version = sys.version_info[0:2]

    if version[0] == 2:
        return ["--cfg=Py_2"]

    py3_min = 5
    out_cfg = []
    for minor in range(py3_min, version[1] + 1):
        out_cfg.append("--cfg=Py_3_%d" % minor)

    if platform.python_implementation() == "PyPy":
        out_cfg.append("--cfg=PyPy")

    return out_cfg


install_requires = []
tests_require = install_requires + ["pytest", "pytest-benchmark"]

setup(
    name="rustapi-module",
    version="0.1.0",
    classifiers=[
        "License :: OSI Approved :: MIT License",
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "Programming Language :: Python",
        "Programming Language :: Rust",
        "Operating System :: POSIX",
        "Operating System :: MacOS :: MacOS X",
    ],
    packages=["rustapi_module"],
    rust_extensions=[
        RustExtension(
            "rustapi_module.othermod", "Cargo.toml", rustc_flags=get_py_version_cfgs()
        ),
        RustExtension(
            "rustapi_module.datetime", "Cargo.toml", rustc_flags=get_py_version_cfgs()
        ),
        RustExtension(
            "rustapi_module.subclassing",
            "Cargo.toml",
            rustc_flags=get_py_version_cfgs(),
        ),
        RustExtension(
            "rustapi_module.test_dict",
            "Cargo.toml",
            rustc_flags=get_py_version_cfgs(),
        ),
    ],
    install_requires=install_requires,
    tests_require=tests_require,
    include_package_data=True,
    zip_safe=False,
    cmdclass=dict(test=PyTest),
)
