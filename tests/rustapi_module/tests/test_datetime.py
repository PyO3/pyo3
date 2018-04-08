import rustapi_module.datetime as rdt

import datetime as pdt

import pytest

UTC = pdt.timezone.utc


def test_date():
    assert rdt.make_date(2017, 9, 1) == pdt.date(2017, 9, 1)


def test_invalid_date_fails():
    with pytest.raises(ValueError):
        rdt.make_date(2017, 2, 30)


@pytest.mark.parametrize('args, kwargs', [
    ((0, 0, 0, 0, None), {}),
    ((1, 12, 14, 124731), {}),
    ((1, 12, 14, 124731), {'tzinfo': UTC}),
])
def test_time(args, kwargs):
    act = rdt.make_time(*args, **kwargs)
    exp = pdt.time(*args, **kwargs)

    assert act == exp
    assert act.tzinfo is exp.tzinfo


@pytest.mark.xfail
@pytest.mark.parametrize('args', [
    (-1, 0, 0, 0),
    (0, -1, 0, 0),
    (0, 0, -1, 0),
    (0, 0, 0, -1),
])
def test_invalid_time_fails_xfail(args):
    with pytest.raises(ValueError):
        rdt.make_time(*args)


@pytest.mark.parametrize('args', [
    (24, 0, 0, 0),
    (25, 0, 0, 0),
    (0, 60, 0, 0),
    (0, 61, 0, 0),
    (0, 0, 60, 0),
    (0, 0, 61, 0),
    (0, 0, 0, 1000000)
])
def test_invalid_time_fails(args):
    with pytest.raises(ValueError):
        rdt.make_time(*args)


@pytest.mark.parametrize('args', [
    ('0', 0, 0, 0),
    (0, '0', 0, 0),
    (0, 0, '0', 0),
    (0, 0, 0, '0'),
    (0, 0, 0, 0, 'UTC')
])
def test_time_typeerror(args):
    with pytest.raises(TypeError):
        rdt.make_time(*args)


@pytest.mark.parametrize('args, kwargs', [
    ((2017, 9, 1, 12, 45, 30, 0), {}),
    ((2017, 9, 1, 12, 45, 30, 0), {'tzinfo': UTC}),
])
def test_datetime(args, kwargs):
    act = rdt.make_datetime(*args, **kwargs)
    exp = pdt.datetime(*args, **kwargs)

    assert act == exp
    assert act.tzinfo is exp.tzinfo


def test_invalid_datetime_fails():
    with pytest.raises(ValueError):
        rdt.make_datetime(2011, 1, 42, 0, 0, 0, 0)


def test_datetime_typeerror():
    with pytest.raises(TypeError):
        rdt.make_datetime('2011', 1, 1, 0, 0, 0, 0)
