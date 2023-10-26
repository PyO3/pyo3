from decorator import Counter


def test_no_args():
    @Counter
    def say_hello():
        print("hello")

    say_hello()
    say_hello()
    say_hello()
    say_hello()

    assert say_hello.count == 4


def test_arg():
    @Counter
    def say_hello(name):
        print(f"hello {name}")

    say_hello("a")
    say_hello("b")
    say_hello("c")
    say_hello("d")

    assert say_hello.count == 4


def test_default_arg():
    @Counter
    def say_hello(name="default"):
        print(f"hello {name}")

    say_hello("a")
    say_hello()
    say_hello("c")
    say_hello()

    assert say_hello.count == 4


# https://github.com/PyO3/pyo3/discussions/2598
def test_discussion_2598():
    @Counter
    def say_hello():
        if say_hello.count < 2:
            print("hello from decorator")

    say_hello()
    say_hello()
