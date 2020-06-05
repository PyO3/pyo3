from .word_count import count_line, search, search_sequential

__all__ = ["count_line", "search_py", "search", "search_sequential"]


def search_py(contents, needle):
    total = 0
    for line in contents.split():
        words = line.split(" ")
        for word in words:
            if word == needle:
                total += 1
    return total
