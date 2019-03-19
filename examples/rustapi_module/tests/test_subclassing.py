from rustapi_module.subclassing import Subclassable


# should not raise
def test_subclassing_works():
    class SomeSubClass(Subclassable):
        pass

    a = SomeSubClass()
    _b = str(a) + repr(a)
