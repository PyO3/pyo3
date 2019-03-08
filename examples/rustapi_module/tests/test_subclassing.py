from rustapi_module.subclassing import Subclassable


class SomeSubClass(Subclassable):
    pass


a = SomeSubClass()
_b = str(a) + repr(a)
