from typing import Final, final

PI: Final[float]
SIMPLE: Final = "S\0\x01\t\n\r\"'\\"

@final
class ClassWithConst:
    INSTANCE: Final[ClassWithConst]
