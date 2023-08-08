import pathlib

import pytest

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


def test_take_pathlike():
    assert rpath.take_pathbuf(PathLike("/root")) == "/root"


def test_take_invalid_pathlike():
    with pytest.raises(TypeError):
        assert rpath.take_pathbuf(PathLike(1))


def test_take_invalid():
    with pytest.raises(TypeError):
        assert rpath.take_pathbuf(3)


class PathLike:
    def __init__(self, path):
        self._path = path

    def __fspath__(self):
        return self._path
