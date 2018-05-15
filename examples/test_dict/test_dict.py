from _test_dict import DictSize


def test_size(size):
    print ("-------------------")
    print ("size:{}",size)
    d = {}
    for i in range(0,size):
        d[i]=i
    
    print("Python says");
    print(d)
    print("Rust says");
    DictSize(len(d)).iter_dict(d)
    print("Look ma, no segfault!")

#to find the sweet spot on your system, change the range
for size in range(127,150):
    test_size(size)


