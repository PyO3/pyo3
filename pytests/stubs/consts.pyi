import consts
import typing

PI: typing.Final[float]
SIMPLE: typing.Final = "SIMPLE"

class ClassWithConst:
    INSTANCE: typing.Final[consts.ClassWithConst]
