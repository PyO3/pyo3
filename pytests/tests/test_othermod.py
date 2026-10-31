from pyo3_pytests import othermod


def test_modclass():
    # Test that the repr of the class itself doesn't crash anything
    repr(othermod.ModClass)

    assert isinstance(othermod.ModClass, type)


def test_modclass_instance():
    mi = othermod.ModClass()

    repr(mi)
    repr(mi.__class__)

    assert isinstance(mi, othermod.ModClass)
    assert isinstance(mi, object)
