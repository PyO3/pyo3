from pyo3_pytests import exceptions
import pytest


def test_exceptions():
    assert exceptions.do_something("success") is None

    with pytest.raises(exceptions.CustomException) as exc_info:
        exceptions.do_something("fail")

    assert exc_info.value.args == ("unknown op `fail`",)

    with pytest.raises(exceptions.ExceptionSubclassA) as exc_info:
        exceptions.do_something("subclass_a")

    assert exc_info.value.args == ("subclass_a",)

    with pytest.raises(exceptions.ExceptionSubclassAChild) as exc_info:
        exceptions.do_something("subclass_a_child")

    assert exc_info.value.args == ("subclass_a_child",)

    with pytest.raises(exceptions.ExceptionSubclassB) as exc_info:
        exceptions.do_something("subclass_b")

    assert exc_info.value.args == ("subclass_b",)
