import rustapi_module.datetime as rdt

import datetime as pdt

import pytest

# Constants
UTC = pdt.timezone.utc
MAX_DAYS = pdt.timedelta.max // pdt.timedelta(days=1)
MIN_DAYS = pdt.timedelta.min // pdt.timedelta(days=1)
MAX_SECONDS = int(pdt.timedelta.max.total_seconds())
MIN_SECONDS = int(pdt.timedelta.min.total_seconds())
MAX_MICROSECONDS = int(pdt.timedelta.max.total_seconds() * 1e6)
MIN_MICROSECONDS = int(pdt.timedelta.min.total_seconds() * 1e6)


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


@pytest.mark.parametrize('args', [
    (0, 0, 0),
    (1, 0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (1, -1, 0),
    (-1, 1, 0),
    (0, 0, 123456),
    (0, 0, -123456),
])
def test_delta(args):
    act = pdt.timedelta(*args)
    exp = rdt.make_delta(*args)

    assert act == exp


@pytest.mark.parametrize('args,err_type', [
    ((MAX_DAYS + 1, 0, 0), OverflowError),
    ((MIN_DAYS - 1, 0, 0), OverflowError),
    ((0, MAX_SECONDS + 1, 0), OverflowError),
    ((0, MIN_SECONDS - 1, 0), OverflowError),
    ((0, 0, MAX_MICROSECONDS + 1), OverflowError),
    ((0, 0, MIN_MICROSECONDS - 1), OverflowError),
    (('0', 0, 0), TypeError),
    ((0, '0', 0), TypeError),
    ((0, 0, '0'), TypeError),
])
def test_delta_err(args, err_type):
    with pytest.raises(err_type):
        rdt.make_delta(*args)
