# import the contents of the Rust library into the Python extension
from ._setuptools_rust_starter import *
from ._setuptools_rust_starter import __all__

# optional: include the documentation from the Rust module
from ._setuptools_rust_starter import __doc__  # noqa: F401

__all__ = __all__ + ["PythonClass"]


class PythonClass:
    def __init__(self, value: int) -> None:
        self.value = value
