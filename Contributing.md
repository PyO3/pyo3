# Contributing

Thank you for your interest in contributing to PyO3! All are welcome - please consider reading our [Code of Conduct](Code-of-Conduct.md) to keep our community positive and inclusive.

This guide contains suggestions on how to join in with the development of PyO3.

## Getting started contributing

All development activity is coordinated through the Github issue tracker. You may have already looked there searching for an item to work on.

You're welcome to volunteer on any part of PyO3 which strikes your interest. The following sections contain ideas on where to start.

### Help users identify bugs

The [PyO3 Gitter channel](https://gitter.im/PyO3/Lobby) is very active with users who are new to PyO3, and often completely new to Rust. Helping them debug their solutions is a great way to get to the nuts and bolts of PyO3.

Often this activity leads to discoveries of bugs, documentation weaknesses, and missing APIs. It's good practice to open Github issues for these discoveries immediately so the resolution can be planned and implemented!

### Dive straight into the implementation

We use the [needs-implementer](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Aneeds-implemeter) label to mark issues where the solution is reasonably clear and nobody has yet claimed responsibility for its enaction.

Feel free to introduce yourself directly on any issue which strikes your interest and volunteer to take it on!

Don't be afraid if the full solution is not clear to you! The core PyO3 contributors will be happy to mentor you through any questions you have to help you write the solution.

### Help write great docs

Perhaps even more important than implementing features is ensuring users know how to leverage them!

PyO3 aims to have an extensive user guide as well as exhaustive API docs. Pull requests to fix typos, change wording, add examples, or any other improvement you spot are always welcome.

There's a few particular initiatives you can look for when helping with documentation:
- Areas where we know the documentation could be improved are tracked with the [documentation](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Adocumentation) label.
- Not all APIs had docs or examples written when they were originally created. We'd like to one day have docstrings on all functions ([#306](https://github.com/PyO3/pyo3/issues/306)). If you see a function lacking a good doc, please write one and open a PR!
- Not all `unsafe` APIs had safety notes written when they were originally created. We'd like to one day ensure all `unsafe` usage is carefully guided ([#698](https://github.com/PyO3/pyo3/issues/698)). If you see an `unsafe` function missing safety notes, please write one and open a PR!

### Help design the future PyO3

For issues which don't yet have enough detail for a complete solution, we use the [needs-design](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Aneeds-design) label.

If any of these issues interest you, please join in with the conversation on the issue! All opinions are valuable, and if you're interested in going further with e.g. draft PRs to experiment with API designs, even better!

### Review pull requests

The job doesn't stop with the first commit! All are welcome to submit comments on open PRs. Help us make sure that APIs we merge are safe, performant, tidy, and easy to use!

## Implementation notes

Here are a few things to bear in mind when you are writing PRs.

### CI

We use a mixture of Github actions and Travis CI to verify pull requests. PRs are blocked from merging if they do not pass our CI infrastructure.

In particular, keep in mind that formatting, linting and tests are checked for all Rust and Python code, for a number of Rust and Python versions.

### Minimum supported Rust version

PyO3 aims to make use of up-to-date Rust language features to keep the implementation as efficient as possible.

That said, the aim is to always have support for at least a few of the most recent Rust compiler versions, so that users have time to update.

If your PR needs to bump the minimum supported Rust version, this is acceptable, with two caveats:
- Any changes which require a more recent version than what is [currently available on stable Red Hat Enterprise Linux](https://access.redhat.com/documentation/en-us/red_hat_developer_tools/1/) will be postponed. (This is a rough measure to allow package managers to update support for newer `rustc` versions; RHEL was arbitrarily picked because their update policy is clear.)
- You might be asked to do extra work to tidy up other parts of the PyO3 codebase which could benefit from the compiler version bump :)
