import setuptools_rust_extension.misc


def test_issue_219():
    # Should not deadlock
    setuptools_rust_extension.misc.issue_219()
