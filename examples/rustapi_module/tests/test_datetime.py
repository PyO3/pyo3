import datetime as pdt
import sys
import platform

import pytest
import rustapi_module.datetime as rdt
from hypothesis import given
from hypothesis import strategies as st
from hypothesis.strategies import dates, datetimes


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

try:
    MAX_DAYS = pdt.timedelta.max // pdt.timedelta(days=1)
    MIN_DAYS = pdt.timedelta.min // pdt.timedelta(days=1)
except Exception:
    # Python 2 compatibility
    MAX_DAYS = MAX_SECONDS // pdt.timedelta(days=1).total_seconds()
    MIN_DAYS = MIN_SECONDS // pdt.timedelta(days=1).total_seconds()

MAX_MICROSECONDS = int(pdt.timedelta.max.total_seconds() * 1e6)
MIN_MICROSECONDS = int(pdt.timedelta.min.total_seconds() * 1e6)

IS_X86 = platform.architecture()[0] == '32bit'
IS_WINDOWS = sys.platform == 'win32'
if IS_WINDOWS:
    if IS_X86:
        MIN_DATETIME_FROM_TIMESTAMP = pdt.datetime.fromtimestamp(86400)
        MAX_DATETIME_FROM_TIMESTAMP = pdt.datetime.fromtimestamp(32536789199)
    else:
        MIN_DATETIME_FROM_TIMESTAMP = pdt.datetime.fromtimestamp(0)
        MAX_DATETIME_FROM_TIMESTAMP = pdt.datetime.fromtimestamp(32536799999)
else:
    if IS_X86:
        MIN_DATETIME_FROM_TIMESTAMP = pdt.datetime.fromtimestamp(-2147483648)
        MAX_DATETIME_FROM_TIMESTAMP = pdt.datetime.fromtimestamp(2147483647)
    else:
        MIN_DATETIME_FROM_TIMESTAMP = pdt.datetime.fromtimestamp(-62135510400)
        MAX_DATETIME_FROM_TIMESTAMP = pdt.datetime.fromtimestamp(253402300799)

PYPY = platform.python_implementation() == "PyPy"
HAS_FOLD = getattr(pdt.datetime, "fold", False)

# Helper functions
get_timestamp = getattr(pdt.datetime, "timestamp", None)
if get_timestamp is None:

    def get_timestamp(dt):
        # Python 2 compatibility
        return (dt - pdt.datetime(1970, 1, 1)).total_seconds()


xfail_date_bounds = pytest.mark.xfail(
    sys.version_info < (3, 6),
    reason="Date bounds were not checked in the C constructor prior to version 3.6",
)


# Tests
def test_date():
    assert rdt.make_date(2017, 9, 1) == pdt.date(2017, 9, 1)


@given(d=st.dates())
def test_date_accessors(d):
    act = rdt.get_date_tuple(d)
    exp = (d.year, d.month, d.day)

    assert act == exp


@xfail_date_bounds
def test_invalid_date_fails():
    with pytest.raises(ValueError):
        rdt.make_date(2017, 2, 30)


@given(d=st.dates(MIN_DATETIME_FROM_TIMESTAMP.date(),
                  MAX_DATETIME_FROM_TIMESTAMP.date()))
def test_date_from_timestamp(d):
    if PYPY and d < pdt.date(1900, 1, 1):
        pytest.xfail("get_timestamp will raise on PyPy with dates before 1900")

    ts = get_timestamp(pdt.datetime.combine(d, pdt.time(0)))
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


@given(t=st.times())
def test_time(t):
    act = rdt.get_time_tuple(t)
    exp = (t.hour, t.minute, t.second, t.microsecond)

    assert act == exp


@pytest.mark.skipif(not HAS_FOLD, reason="Feature not available before 3.6")
@given(t=st.times())
def test_time_fold(t):
    t_nofold = t.replace(fold=0)
    t_fold = t.replace(fold=1)

    for t in (t_nofold, t_fold):
        act = rdt.get_time_tuple_fold(t)
        exp = (t.hour, t.minute, t.second, t.microsecond, t.fold)

        assert act == exp


@pytest.mark.skipif(not HAS_FOLD, reason="Feature not available before 3.6")
@pytest.mark.parametrize("fold", [False, True])
def test_time_fold(fold):
    t = rdt.time_with_fold(0, 0, 0, 0, None, fold)
    assert t.fold == fold


@pytest.mark.xfail
@pytest.mark.parametrize(
    "args", [(-1, 0, 0, 0), (0, -1, 0, 0), (0, 0, -1, 0), (0, 0, 0, -1)]
)
def test_invalid_time_fails_xfail(args):
    with pytest.raises(ValueError):
        rdt.make_time(*args)


@xfail_date_bounds
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


@given(dt=st.datetimes())
def test_datetime_tuple(dt):
    act = rdt.get_datetime_tuple(dt)
    exp = dt.timetuple()[0:6] + (dt.microsecond,)

    assert act == exp


@pytest.mark.skipif(not HAS_FOLD, reason="Feature not available before 3.6")
@given(dt=st.datetimes())
def test_datetime_tuple_fold(dt):
    dt_fold = dt.replace(fold=1)
    dt_nofold = dt.replace(fold=0)

    for dt in (dt_fold, dt_nofold):
        act = rdt.get_datetime_tuple_fold(dt)
        exp = dt.timetuple()[0:6] + (dt.microsecond, dt.fold)

        assert act == exp


@xfail_date_bounds
def test_invalid_datetime_fails():
    with pytest.raises(ValueError):
        rdt.make_datetime(2011, 1, 42, 0, 0, 0, 0)


def test_datetime_typeerror():
    with pytest.raises(TypeError):
        rdt.make_datetime("2011", 1, 1, 0, 0, 0, 0)


@given(dt=st.datetimes(MIN_DATETIME_FROM_TIMESTAMP,
                       MAX_DATETIME_FROM_TIMESTAMP))
def test_datetime_from_timestamp(dt):
    if PYPY and dt < pdt.datetime(1900, 1, 1):
        pytest.xfail("get_timestamp will raise on PyPy with dates before 1900")

    ts = get_timestamp(dt)
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


def test_issue_219():
    rdt.issue_219()


def test_tz_class():
    tzi = rdt.TzClass()

    dt = pdt.datetime(2018, 1, 1, tzinfo=tzi)

    assert dt.tzname() == "+01:00"
    assert dt.utcoffset() == pdt.timedelta(hours=1)
    assert dt.dst() is None


def test_tz_class_introspection():
    tzi = rdt.TzClass()

    assert tzi.__class__ == rdt.TzClass
    assert repr(tzi).startswith("<TzClass object at")
