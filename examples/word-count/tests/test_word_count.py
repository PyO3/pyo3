# -*- coding: utf-8 -*-
import os

import pytest
import word_count

current_dir = os.path.abspath(os.path.dirname(__file__))
path = os.path.join(current_dir, "zen-of-python.txt")


@pytest.fixture(scope="session", autouse=True)
def textfile():
    text = """
The Zen of Python, by Tim Peters

Beautiful is better than ugly.
Explicit is better than implicit.
Simple is better than complex.
Complex is better than complicated.
Flat is better than nested.
Sparse is better than dense.
Readability counts.
Special cases aren't special enough to break the rules.
Although practicality beats purity.
Errors should never pass silently.
Unless explicitly silenced.
In the face of ambiguity, refuse the temptation to guess.
There should be one-- and preferably only one --obvious way to do it.
Although that way may not be obvious at first unless you're Dutch.
Now is better than never.
Although never is often better than *right* now.
If the implementation is hard to explain, it's a bad idea.
If the implementation is easy to explain, it may be a good idea.
Namespaces are one honking great idea -- let's do more of those!
"""

    with open(path, "w") as f:
        f.write(text * 1000)
    yield
    os.remove(path)


def test_word_count_rust_parallel(benchmark):
    count = benchmark(word_count.WordCounter(path).search, "is")
    assert count == 10000


def test_word_count_rust_sequential(benchmark):
    count = benchmark(word_count.WordCounter(path).search_sequential, "is")
    assert count == 10000


def test_word_count_python_sequential(benchmark):
    count = benchmark(word_count.search_py, path, "is")
    assert count == 10000
