from maturin_starter.submodule import SubmoduleClass


def test_submodule_class() -> None:
    submodule_class = SubmoduleClass()
    assert submodule_class.greeting() == "Hello, world!"
