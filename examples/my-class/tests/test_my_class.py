# -*- coding: utf-8 -*-
from __future__ import absolute_import

import os

import pytest

from my_class import MyClass


def test_get_from_clone():
    c = MyClass("title")
    assert c.title == "title"

