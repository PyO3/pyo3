import ast
from pathlib import Path

import pyo3_pytests
import pytest

STUBS_DIR = Path(__file__).parent.parent / "stubs"


def _stub_dunder_all(path: Path):
    """The `__all__` a stub file declares, or `None` if it declares none."""
    for node in ast.parse(path.read_text()).body:
        if isinstance(node, ast.Assign) and any(
            isinstance(target, ast.Name) and target.id == "__all__"
            for target in node.targets
        ):
            return ast.literal_eval(node.value)
    return None


@pytest.mark.parametrize(
    "stub_file", sorted(STUBS_DIR.glob("*.pyi")), ids=lambda path: path.name
)
def test_stub_dunder_all_matches_runtime(stub_file: Path):
    """The `__all__` in the stubs must name what the module exports at import time.

    Sorted, because the stubs declare the members in a different order than
    `PyModuleMethods::add` appends them in, and `__all__` is only consumed as a set of names.
    """
    stub_all = _stub_dunder_all(stub_file)
    if stub_all is None:
        pytest.skip("incomplete modules get no `__all__`, see `guide/src/type-stub.md`")
    root = pyo3_pytests.pyo3_pytests
    module = root if stub_file.name == "__init__.pyi" else getattr(root, stub_file.stem)
    runtime_all = module.__all__
    assert sorted(stub_all) == sorted(runtime_all)
    assert len(stub_all) == len(set(stub_all)), "`__all__` has duplicates"
