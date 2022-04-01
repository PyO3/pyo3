import pathlib

import pyo3_pytests.path as rpath


def test_make_path():
    p = rpath.make_path()
    assert p == "/root"


def test_take_pathbuf():
    p = "/root"
    assert rpath.take_pathbuf(p) == p


def test_take_pathlib():
    p = pathlib.Path("/root")
    assert rpath.take_pathbuf(p) == str(p)
