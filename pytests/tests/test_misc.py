import pyo3_pytests.misc
import pytest


def test_issue_219():
    # Should not deadlock
    pyo3_pytests.misc.issue_219()


def test_capsule_send_destructor():
    with pytest.warns(
        RuntimeWarning,
        match="capsule destructor called in thread other than the one the capsule was created in",
    ):
        pyo3_pytests.misc.capsule_send_destructor()
