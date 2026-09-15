from .pyclasses import EmptyClass

__all__ = ["cross_module_imports", "with_custom_type_annotations"]

def cross_module_imports(_a: EmptyClass) -> None: ...
def with_custom_type_annotations(
    a: "list[int]", *_args: "str", _b: "int | None" = None, **_kwargs: "bool"
) -> "int": ...
