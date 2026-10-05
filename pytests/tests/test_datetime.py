import datetime as pdt
import re

import pyo3_pytests.datetime as rdt
import pytest


# Constants
def _get_utc():
    timezone = getattr(pdt, "timezone", None)
    if timezone:
        return timezone.utc
    else:

        class UTC(pdt.tzinfo):
            def utcoffset(self, dt):
                return pdt.timedelta(0)

            def dst(self, dt):
                return pdt.timedelta(0)

            def tzname(self, dt):
                return "UTC"

        return UTC()


UTC = _get_utc()

MAX_SECONDS = int(pdt.timedelta.max.total_seconds())
MIN_SECONDS = int(pdt.timedelta.min.total_seconds())

MAX_DAYS = pdt.timedelta.max // pdt.timedelta(days=1)
MIN_DAYS = pdt.timedelta.min // pdt.timedelta(days=1)

MAX_MICROSECONDS = int(pdt.timedelta.max.total_seconds() * 1e6)
MIN_MICROSECONDS = int(pdt.timedelta.min.total_seconds() * 1e6)


# Tests
def test_date():
    assert rdt.make_date(2017, 9, 1) == pdt.date(2017, 9, 1)


def test_invalid_date_fails():
    with pytest.raises(ValueError):
        rdt.make_date(2017, 2, 30)


@pytest.mark.parametrize(
    "args, kwargs",
    [
        ((0, 0, 0, 0, None), {}),
        ((1, 12, 14, 124731), {}),
        ((1, 12, 14, 124731), {"tzinfo": UTC}),
    ],
)
def test_time(args, kwargs):
    act = rdt.make_time(*args, **kwargs)
    exp = pdt.time(*args, **kwargs)

    assert act == exp
    assert act.tzinfo is exp.tzinfo
    assert rdt.get_time_tzinfo(act) == exp.tzinfo


@pytest.mark.parametrize("fold", [False, True])
def test_time_with_fold(fold):
    t = rdt.time_with_fold(0, 0, 0, 0, None, fold)
    assert t.fold == fold


@pytest.mark.parametrize(
    "args", [(-1, 0, 0, 0), (0, -1, 0, 0), (0, 0, -1, 0), (0, 0, 0, -1)]
)
def test_invalid_time_fails_overflow(args):
    with pytest.raises(OverflowError):
        rdt.make_time(*args)


@pytest.mark.parametrize(
    "args",
    [
        (24, 0, 0, 0),
        (25, 0, 0, 0),
        (0, 60, 0, 0),
        (0, 61, 0, 0),
        (0, 0, 60, 0),
        (0, 0, 61, 0),
        (0, 0, 0, 1000000),
    ],
)
def test_invalid_time_fails(args):
    with pytest.raises(ValueError):
        rdt.make_time(*args)


@pytest.mark.parametrize(
    "args",
    [
        ("0", 0, 0, 0),
        (0, "0", 0, 0),
        (0, 0, "0", 0),
        (0, 0, 0, "0"),
        (0, 0, 0, 0, "UTC"),
    ],
)
def test_time_typeerror(args):
    with pytest.raises(TypeError):
        rdt.make_time(*args)


@pytest.mark.parametrize(
    "args, kwargs",
    [((2017, 9, 1, 12, 45, 30, 0), {}), ((2017, 9, 1, 12, 45, 30, 0), {"tzinfo": UTC})],
)
def test_datetime(args, kwargs):
    act = rdt.make_datetime(*args, **kwargs)
    exp = pdt.datetime(*args, **kwargs)  # noqa: DTZ001

    assert act == exp
    assert act.tzinfo is exp.tzinfo
    assert rdt.get_datetime_tzinfo(act) == exp.tzinfo


def test_invalid_datetime_fails():
    with pytest.raises(ValueError):
        rdt.make_datetime(2011, 1, 42, 0, 0, 0, 0)


def test_datetime_typeerror():
    with pytest.raises(TypeError):
        rdt.make_datetime("2011", 1, 1, 0, 0, 0, 0)  # type: ignore[bad-argument-type]


def test_datetime_from_timestamp_tzinfo():
    d1 = rdt.datetime_from_timestamp(0, tz=UTC)
    d2 = rdt.datetime_from_timestamp(0, tz=UTC)

    assert d1 == d2
    assert d1.tzinfo is d2.tzinfo


@pytest.mark.parametrize(
    "args",
    [
        (0, 0, 0),
        (1, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (1, -1, 0),
        (-1, 1, 0),
        (0, 0, 123456),
        (0, 0, -123456),
    ],
)
def test_delta(args):
    act = pdt.timedelta(*args)
    exp = rdt.make_delta(*args)

    assert act == exp


@pytest.mark.parametrize(
    "args,err_type",
    [
        ((MAX_DAYS + 1, 0, 0), OverflowError),
        ((MIN_DAYS - 1, 0, 0), OverflowError),
        ((0, MAX_SECONDS + 1, 0), OverflowError),
        ((0, MIN_SECONDS - 1, 0), OverflowError),
        ((0, 0, MAX_MICROSECONDS + 1), OverflowError),
        ((0, 0, MIN_MICROSECONDS - 1), OverflowError),
        (("0", 0, 0), TypeError),
        ((0, "0", 0), TypeError),
        ((0, 0, "0"), TypeError),
    ],
)
def test_delta_err(args, err_type):
    with pytest.raises(err_type):
        rdt.make_delta(*args)


def test_tz_class():
    tzi = rdt.TzClass()

    dt = pdt.datetime(2018, 1, 1, tzinfo=tzi)

    assert dt.tzname() == "+01:00"
    assert dt.utcoffset() == pdt.timedelta(hours=1)
    assert dt.dst() is None


def test_tz_class_introspection():
    tzi = rdt.TzClass()

    assert tzi.__class__ == rdt.TzClass
    # PyPy generates <importlib.bootstrap.TzClass ...> for some reason.
    assert re.match(r"^<[\w\.]*TzClass object at", repr(tzi))
