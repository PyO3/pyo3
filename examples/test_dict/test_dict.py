from _test_dict import DictSize

class Foo:

    def __init__(self, x):
        self.x = x
        print("Foo:", self.x)

    def __repr__(self):
        return "Foo({})".format(self.x)

    def __del__(self):
        print("~Foo:", self.x)


def fake(to_call):
    def monkey(x):
        print(">>Monkey:", x)
        try:
            breakpoint()
        except NameError:
            print("No breakpoint method")
        to_call(x)
        print("<<Monkey:", x)

    return monkey




def test_size(size):
    print ("-------------------")
    print ("size:{}",size)
    d = {}

    for i in range(0,size):
        d[i]=Foo(i)

    print("Injecting destructor to call breakpoint when the dict is dropped")
    Foo.__del__=fake(Foo.__del__)

    print("Pass to rust");
    DictSize(len(d)).iter_dict(d)
    print("Look ma, no segfault!")

#to find the sweet spot on your system, change the range
for size in range(127,150):
    test_size(size)


