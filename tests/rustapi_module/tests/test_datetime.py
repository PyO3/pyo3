import rustapi_module.datetime as rdt

import datetime as pdt

import pytest


def test_date():
    assert rdt.make_date(2017, 9, 1) == pdt.date(2017, 9, 1)


def test_date_fails():
    with pytest.raises(ValueError):
        rdt.make_date(2017, 2, 30)
