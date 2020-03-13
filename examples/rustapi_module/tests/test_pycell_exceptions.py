from rustapi_module.cell import Mutable
from rustapi_module._pyo3_exceptions import PyBorrowError, PyBorrowMutError


def test_catch_borrrow():
    m = Mutable()
    try:
        m.invalid_borrow(m)
        assert False
    except PyBorrowError:
        pass
    except Exception as e:
        raise e


def test_catch_borrrowmut():
    m = Mutable()
    try:
        m.invalid_borrow_mut(m)
        assert False
    except PyBorrowMutError:
        pass
    except Exception as e:
        raise e
