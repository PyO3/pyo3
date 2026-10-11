import datetime as pdt
import struct
import sys

import pyo3_pytests.datetime as rdt
import pytest

hypothesis = pytest.importorskip("hypothesis")
st = pytest.importorskip("hypothesis.strategies")

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
    MIN_DATETIME = pdt.datetime(1970, 1, 1, 0, 0, 0)  # noqa: DTZ001
    if IS_32_BIT:
        MAX_DATETIME = pdt.datetime(2038, 1, 18, 23, 59, 59)  # noqa: DTZ001
    else:
        MAX_DATETIME = pdt.datetime(3000, 12, 31, 23, 59, 59)  # noqa: DTZ001
else:
    if IS_32_BIT:
        # TS ±2147483648 (2**31)
        MIN_DATETIME = pdt.datetime(1901, 12, 13, 20, 45, 52)  # noqa: DTZ001
        MAX_DATETIME = pdt.datetime(2038, 1, 19, 3, 14, 8)  # noqa: DTZ001
    else:
        MIN_DATETIME = pdt.datetime(1, 1, 2, 0, 0)  # noqa: DTZ001
        MAX_DATETIME = pdt.datetime(9999, 12, 31, 18, 59, 59)  # noqa: DTZ001


@hypothesis.given(d=st.dates())
def test_date_accessors(d):
    act = rdt.get_date_tuple(d)
    exp = (d.year, d.month, d.day)

    assert act == exp


@hypothesis.given(dt=st.datetimes(MIN_DATETIME, MAX_DATETIME))
def test_date_from_timestamp(dt):
    try:
        ts = pdt.datetime.timestamp(dt)
    except OverflowError:
        # out of range for timestamp
        return

    try:
        expected = pdt.date.fromtimestamp(ts)  # noqa: DTZ012
    except OverflowError as pdt_fail:
        # date from timestamp failed; expect the same from Rust binding
        with pytest.raises(type(pdt_fail)) as exc_info:
            rdt.date_from_timestamp(ts)
        assert str(exc_info.value) == str(pdt_fail)
    else:
        assert rdt.date_from_timestamp(ts) == expected


@hypothesis.given(t=st.times())
def test_time_hypothesis(t):
    act = rdt.get_time_tuple(t)
    exp = (t.hour, t.minute, t.second, t.microsecond)

    assert act == exp


@hypothesis.given(t=st.times())
def test_time_tuple_fold(t):
    t_nofold = t.replace(fold=0)
    t_fold = t.replace(fold=1)

    for t in (t_nofold, t_fold):  # noqa: PLR1704
        act = rdt.get_time_tuple_fold(t)
        exp = (t.hour, t.minute, t.second, t.microsecond, t.fold)

        assert act == exp


@hypothesis.given(dt=st.datetimes())
def test_datetime_tuple(dt):
    act = rdt.get_datetime_tuple(dt)
    exp = dt.timetuple()[0:6] + (dt.microsecond,)

    assert act == exp


@hypothesis.given(dt=st.datetimes())
def test_datetime_tuple_fold(dt):
    dt_fold = dt.replace(fold=1)
    dt_nofold = dt.replace(fold=0)

    for dt in (dt_fold, dt_nofold):  # noqa: PLR1704
        act = rdt.get_datetime_tuple_fold(dt)
        exp = dt.timetuple()[0:6] + (dt.microsecond, dt.fold)

        assert act == exp


@hypothesis.given(dt=st.datetimes(MIN_DATETIME, MAX_DATETIME))
@hypothesis.example(dt=pdt.datetime(1971, 1, 2, 0, 0))  # noqa: DTZ001
def test_datetime_from_timestamp(dt):
    try:
        ts = pdt.datetime.timestamp(dt)
    except OverflowError:
        # out of range for timestamp
        return

    try:
        expected = pdt.datetime.fromtimestamp(ts)  # noqa: DTZ006
    except OverflowError as pdt_fail:
        # datetime from timestamp failed; expect the same from Rust binding
        with pytest.raises(type(pdt_fail)) as exc_info:
            rdt.datetime_from_timestamp(ts)
        assert str(exc_info.value) == str(pdt_fail)
    else:
        assert rdt.datetime_from_timestamp(ts) == expected


@hypothesis.given(td=st.timedeltas())
def test_delta_accessors(td):
    act = rdt.get_delta_tuple(td)
    exp = (td.days, td.seconds, td.microseconds)

    assert act == exp
