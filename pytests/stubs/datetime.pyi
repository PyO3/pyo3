from datetime import date, datetime, time, timedelta, tzinfo
from typing import SupportsFloat, SupportsIndex, final

__all__ = [
    "TzClass",
    "date_from_timestamp",
    "datetime_from_timestamp",
    "get_date_tuple",
    "get_datetime_tuple",
    "get_datetime_tuple_fold",
    "get_datetime_tzinfo",
    "get_delta_tuple",
    "get_time_tuple",
    "get_time_tuple_fold",
    "get_time_tzinfo",
    "make_date",
    "make_datetime",
    "make_delta",
    "make_time",
    "time_with_fold",
]

@final
class TzClass(tzinfo):
    def __new__(cls, /) -> TzClass: ...
    def dst(self, _dt: datetime | None, /) -> timedelta | None: ...
    def tzname(self, _dt: datetime | None, /) -> str: ...
    def utcoffset(self, _dt: datetime | None, /) -> timedelta: ...

def date_from_timestamp(timestamp: SupportsFloat | SupportsIndex) -> date: ...
def datetime_from_timestamp(
    ts: SupportsFloat | SupportsIndex, tz: tzinfo | None = None
) -> datetime: ...
def get_date_tuple(d: date) -> tuple: ...
def get_datetime_tuple(dt: datetime) -> tuple: ...
def get_datetime_tuple_fold(dt: datetime) -> tuple: ...
def get_datetime_tzinfo(dt: datetime) -> tzinfo | None: ...
def get_delta_tuple(delta: timedelta) -> tuple: ...
def get_time_tuple(dt: time) -> tuple: ...
def get_time_tuple_fold(dt: time) -> tuple: ...
def get_time_tzinfo(dt: time) -> tzinfo | None: ...
def make_date(
    year: SupportsIndex, month: SupportsIndex, day: SupportsIndex
) -> date: ...
def make_datetime(
    year: SupportsIndex,
    month: SupportsIndex,
    day: SupportsIndex,
    hour: SupportsIndex,
    minute: SupportsIndex,
    second: SupportsIndex,
    microsecond: SupportsIndex,
    tzinfo: tzinfo | None = None,
) -> datetime: ...
def make_delta(
    days: SupportsIndex, seconds: SupportsIndex, microseconds: SupportsIndex
) -> timedelta: ...
def make_time(
    hour: SupportsIndex,
    minute: SupportsIndex,
    second: SupportsIndex,
    microsecond: SupportsIndex,
    tzinfo: tzinfo | None = None,
) -> time: ...
def time_with_fold(
    hour: SupportsIndex,
    minute: SupportsIndex,
    second: SupportsIndex,
    microsecond: SupportsIndex,
    tzinfo: tzinfo | None,
    fold: bool,
) -> time: ...
