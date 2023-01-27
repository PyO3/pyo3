import pytest


@pytest.fixture
def gadget():
    import plugin_api as pa

    g = pa.Gadget()
    return g


def test_creation(gadget):
    pass


def test_property(gadget):
    gadget.prop = 42
    assert gadget.prop == 42


def test_push(gadget):
    gadget.push(42)
