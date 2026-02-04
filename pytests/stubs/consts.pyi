from typing import Final, final

ESCAPING: Final = "S\0\x01\t\n\r\"'\\"
"""
We experiment with "escaping"
"""

PI: Final[float]
"""
Exports PI constant as part of the module
"""

@final
class ClassWithConst:
    INSTANCE: Final[ClassWithConst]
    """
    A constant
    """
