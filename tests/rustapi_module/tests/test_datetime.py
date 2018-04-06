import rustapi_module.datetime as rdt

import datetime as pdt

import pytest


def test_date():
    assert rdt.make_date(2017, 9, 1) == pdt.date(2017, 9, 1)


def test_invalid_date_fails():
    with pytest.raises(ValueError):
        rdt.make_date(2017, 2, 30)


@pytest.mark.parametrize('args, kwargs', [
    ((2017, 9, 1, 12, 45, 30, 0), {}),
    ((2017, 9, 1, 12, 45, 30, 0), {'tzinfo': pdt.timezone.utc}),
])
def test_datetime(args, kwargs):
    act = rdt.make_datetime(*args, **kwargs)
    exp = pdt.datetime(*args, **kwargs)

    assert act == exp


def test_invalid_datetime_fails():
    with pytest.raises(ValueError):
        rdt.make_datetime(2011, 1, 42, 0, 0, 0, 0)


def test_datetime_typeerror():
    with pytest.raises(TypeError):
        rdt.make_datetime('2011', 1, 1, 0, 0, 0, 0)
