from decorator import Counter


@Counter
def say_hello():
    print("hello")


say_hello()
say_hello()
say_hello()
say_hello()

assert say_hello.count == 4
