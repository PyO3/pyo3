from datetime import date, datetime as dt, time
from uuid import UUID

def with_built_in_type_annotations(
    _date_time: dt, _time: time, _date: date
) -> None: ...
def with_custom_type_annotations(
    a: "dt | time | UUID", *_args: "str", _b: "int | None" = None, **_kwargs: "bool"
) -> "int": ...
