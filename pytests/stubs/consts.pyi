from typing import Final, final

ESCAPING: Final = "S\0\x01\t\n\r\"'\\"
PI: Final[float]

@final
class ClassWithConst:
    INSTANCE: Final[ClassWithConst]
