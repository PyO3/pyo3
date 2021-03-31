import maturin_extension.misc


def test_issue_219():
    # Should not deadlock
    misc.issue_219()
