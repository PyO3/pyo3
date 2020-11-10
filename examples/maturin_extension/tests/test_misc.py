import maturin_extension.misc


def test_issue_219():
    # Should not deadlock
    maturin_extension.misc.issue_219()
