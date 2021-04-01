# import the contents of the Rust library into the Python extension
from .maturin_starter import *


class PythonClass:
    def __init__(self, value: int) -> None:
        self.value = value
