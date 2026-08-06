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


def _module_for_stub(path: Path):
    root = pyo3_pytests.pyo3_pytests
    return root if path.name == "__init__.pyi" else getattr(root, path.stem)


@pytest.mark.parametrize(
    "stub_file", sorted(STUBS_DIR.glob("*.pyi")), ids=lambda path: path.name
)
def test_stub_dunder_all_matches_runtime(stub_file: Path):
    """The `__all__` in the stubs must name what the module exports at import time.

    We compare sorted lists rather than the lists themselves: the stubs list the members in
    the order they are declared in, which is not the order `PyModuleMethods::add` appends
    them in. `__all__` is only ever consumed as a set of names, so that difference is not
    observable.

    This needs `pyo3_pytests` built with the features the stubs were generated from, so it
    runs in the `test-introspection` nox session rather than in `pytests`' own one.
    """
    stub_all = _stub_dunder_all(stub_file)
    if stub_all is None:
        pytest.skip("incomplete modules get no `__all__`, see `guide/src/type-stub.md`")
    runtime_all = _module_for_stub(stub_file).__all__
    assert sorted(stub_all) == sorted(runtime_all)
    assert len(stub_all) == len(set(stub_all)), "`__all__` has duplicates"
