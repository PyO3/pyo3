import rustapi_module.dunder


def test_add():
    assert rustapi_module.dunder.Number(10) + 20 == 30
