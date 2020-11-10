import os
import sys
import importlib
import importlib.abc

# Import the local extension
from .maturin_extension import __file__ as rust_extension_path


class SubmoduleFinder(importlib.abc.MetaPathFinder):
    def find_module(self, fullname, path):
        if fullname.startswith("maturin_extension."):
            return importlib.machinery.ExtensionFileLoader(
                fullname, rust_extension_path
            )


def _register_submodules():
    """Inject custom finder into sys.meta_path"""
    sys.meta_path.append(SubmoduleFinder())
