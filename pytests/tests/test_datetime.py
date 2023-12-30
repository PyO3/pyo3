import datetime as pdt
import platform
import re
import struct
import sys

import pyo3_pytests.datetime as rdt
import pytest
from hypothesis import example, given
from hypothesis import strategies as st


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

# The reason we don't use platform.architecture() here is that it's not
# reliable on macOS. See https://stackoverflow.com/a/1405971/823869. Similarly,
# sys.maxsize is not reliable on Windows. See
# https://stackoverflow.com/questions/1405913/how-do-i-determine-if-my-python-shell-is-executing-in-32bit-or-64bit-mode-on-os/1405971#comment6209952_1405971
# and https://stackoverflow.com/a/3411134/823869.
_pointer_size = struct.calcsize("P")
if _pointer_size == 8:
    IS_32_BIT = False
elif _pointer_size == 4:
    IS_32_BIT = True
else:
    raise RuntimeError("unexpected pointer size: " + repr(_pointer_size))
IS_WINDOWS = sys.platform == "win32"

if IS_WINDOWS:
    MIN_DATETIME = pdt.datetime(1971, 1, 2, 0, 0)
    if IS_32_BIT:
        MAX_DATETIME = pdt.datetime(3001, 1, 19, 4, 59, 59)
    else:
        MAX_DATETIME = pdt.datetime(3001, 1, 19, 7, 59, 59)
else:
    if IS_32_BIT:
        # TS ±2147483648 (2**31)
        MIN_DATETIME = pdt.datetime(1901, 12, 13, 20, 45, 52)
        MAX_DATETIME = pdt.datetime(2038, 1, 19, 3, 14, 8)
    else:
        MIN_DATETIME = pdt.datetime(1, 1, 2, 0, 0)
        MAX_DATETIME = pdt.datetime(9999, 12, 31, 18, 59, 59)

PYPY = platform.python_implementation() == "PyPy"


# Tests
def test_date():
    assert rdt.make_date(2017, 9, 1) == pdt.date(2017, 9, 1)


@given(d=st.dates())
def test_date_accessors(d):
    act = rdt.get_date_tuple(d)
    exp = (d.year, d.month, d.day)

    assert act == exp


def test_invalid_date_fails():
    with pytest.raises(ValueError):
        rdt.make_date(2017, 2, 30)


@given(d=st.dates(MIN_DATETIME.date(), MAX_DATETIME.date()))
def test_date_from_timestamp(d):
    if PYPY and d < pdt.date(1900, 1, 1):
        pytest.xfail("pdt.datetime.timestamp will raise on PyPy with dates before 1900")

    ts = pdt.datetime.timestamp(pdt.datetime.combine(d, pdt.time(0)))
    assert rdt.date_from_timestamp(int(ts)) == pdt.date.fromtimestamp(ts)


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


@given(t=st.times())
def test_time_hypothesis(t):
    act = rdt.get_time_tuple(t)
    exp = (t.hour, t.minute, t.second, t.microsecond)

    assert act == exp


@given(t=st.times())
def test_time_tuple_fold(t):
    t_nofold = t.replace(fold=0)
    t_fold = t.replace(fold=1)

    for t in (t_nofold, t_fold):
        act = rdt.get_time_tuple_fold(t)
        exp = (t.hour, t.minute, t.second, t.microsecond, t.fold)

        assert act == exp


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
    exp = pdt.datetime(*args, **kwargs)

    assert act == exp
    assert act.tzinfo is exp.tzinfo
    assert rdt.get_datetime_tzinfo(act) == exp.tzinfo


@given(dt=st.datetimes())
def test_datetime_tuple(dt):
    act = rdt.get_datetime_tuple(dt)
    exp = dt.timetuple()[0:6] + (dt.microsecond,)

    assert act == exp


@given(dt=st.datetimes())
def test_datetime_tuple_fold(dt):
    dt_fold = dt.replace(fold=1)
    dt_nofold = dt.replace(fold=0)

    for dt in (dt_fold, dt_nofold):
        act = rdt.get_datetime_tuple_fold(dt)
        exp = dt.timetuple()[0:6] + (dt.microsecond, dt.fold)

        assert act == exp


def test_invalid_datetime_fails():
    with pytest.raises(ValueError):
        rdt.make_datetime(2011, 1, 42, 0, 0, 0, 0)


def test_datetime_typeerror():
    with pytest.raises(TypeError):
        rdt.make_datetime("2011", 1, 1, 0, 0, 0, 0)


@given(dt=st.datetimes(MIN_DATETIME, MAX_DATETIME))
@example(dt=pdt.datetime(1971, 1, 2, 0, 0))
def test_datetime_from_timestamp(dt):
    if PYPY and dt < pdt.datetime(1900, 1, 1):
        pytest.xfail("pdt.datetime.timestamp will raise on PyPy with dates before 1900")

    ts = pdt.datetime.timestamp(dt)
    assert rdt.datetime_from_timestamp(ts) == pdt.datetime.fromtimestamp(ts)


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


@given(td=st.timedeltas())
def test_delta_accessors(td):
    act = rdt.get_delta_tuple(td)
    exp = (td.days, td.seconds, td.microseconds)

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
