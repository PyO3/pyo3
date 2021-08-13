from concurrent.futures import ThreadPoolExecutor

import pytest
import word_count


@pytest.fixture(scope="session")
def contents() -> str:
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
    return text * 1000


def test_word_count_rust_parallel(benchmark, contents):
    count = benchmark(word_count.search, contents, "is")
    assert count == 10000


def test_word_count_rust_sequential(benchmark, contents):
    count = benchmark(word_count.search_sequential, contents, "is")
    assert count == 10000


def test_word_count_python_sequential(benchmark, contents):
    count = benchmark(word_count.search_py, contents, "is")
    assert count == 10000


def run_rust_sequential_twice(
    executor: ThreadPoolExecutor, contents: str, needle: str
) -> int:
    future_1 = executor.submit(
        word_count.search_sequential_allow_threads, contents, needle
    )
    future_2 = executor.submit(
        word_count.search_sequential_allow_threads, contents, needle
    )
    result_1 = future_1.result()
    result_2 = future_2.result()
    return result_1 + result_2


def test_word_count_rust_sequential_twice_with_threads(benchmark, contents):
    executor = ThreadPoolExecutor(max_workers=2)
    count = benchmark(run_rust_sequential_twice, executor, contents, "is")
    assert count == 20000
