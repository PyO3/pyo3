import pyo3_pytests.misc


def test_issue_219():
    # Should not deadlock
    pyo3_pytests.misc.issue_219()
