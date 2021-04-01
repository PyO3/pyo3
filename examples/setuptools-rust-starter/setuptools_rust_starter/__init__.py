# import the contents of the Rust library into the Python extension
from ._setuptools_rust_starter import *


class PythonClass:
    def __init__(self, value: int) -> None:
        self.value = value
