from .word_count import search, search_sequential, search_sequential_allow_threads

__all__ = [
    "search_py",
    "search",
    "search_sequential",
    "search_sequential_allow_threads",
]


def search_py(contents: str, needle: str) -> int:
    total = 0
    for line in contents.splitlines():
        for word in line.split(" "):
            if word == needle:
                total += 1
    return total
