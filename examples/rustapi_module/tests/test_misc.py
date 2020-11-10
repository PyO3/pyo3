import rustapi_module.misc


def test_issue_219():
    # Should not deadlock
    rustapi_module.misc.issue_219()
