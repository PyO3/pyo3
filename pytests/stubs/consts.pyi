from typing import Final, final

PI: Final[float]
SIMPLE: Final = "SIMPLE"

@final
class ClassWithConst:
    INSTANCE: Final[ClassWithConst]
