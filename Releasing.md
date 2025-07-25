# Releasing

This is notes for the current process of releasing a new PyO3 version. Replace `<version>` in all instructions below with the new version.

## 1. Prepare the release commit

Follow the process below to update all required pieces to bump the version. All these changes are done in a single commit because it makes it clear to git readers what happened to bump the version. It also makes it easy to cherry-pick the version bump onto the `main` branch when tidying up branch history at the end of the release process.

1. Replace all instances of the PyO3 current version and the with the new version to be released. Places to check:
   - `Cargo.toml` for all PyO3 crates in the repository.
   - Examples in `README.md`
   - PyO3 version embedded into documentation like the README.
   - `pre-script.rhai` templates for the examples.
   - `[towncrier]` section in `pyproject.toml`.

   Some of the above locations may already have the new version with a `-dev` suffix, which needs to be removed.

   **Make sure not to modify the CHANGELOG during this step!**

2. Run `towncrier build` to generate the CHANGELOG. The version used by `towncrier` should automatically be correct because of the update to `pyproject.toml` in step 1.

3. Manually edit the CHANGELOG for final notes. Steps to do:
   - Adjust wording of any release lines to make them clearer for users / fix typos.
   - Add a new link at the bottom for the new version, and update the `Unreleased` link.

4. Create the commit containing all the above changes, with a message of `release: <version>`. Push to `release-<BRANCH_VER>` branch on the main PyO3 repository, where `<BRANCH_VER>` depends on whether this is a major or minor release:
   - for O.X.0 minor releases, just use `0.X`, e.g. `release-0.17`. This will become the maintenance branch after release.
   - for 0.X.Y patch releases, use the full `0.X.Y`, e.g. `release-0.17.1`. This will be deleted after merge.

## 2. Create the release PR and draft release notes

Open a PR for the branch, and confirm that it passes CI. For `0.X.0` minor releases, the PR should be merging into `main`, for `0.X.Y` patch releases, the PR should be merging the `release-0.X` maintenance branch.

On https://github.com/PyO3/pyo3/releases, click "Draft a new release". The tag will be a new tag of `v<version>` (note preceding `v`) and target should be the `release-<BRANCH_VER>` branch you just pushed.

Write release notes which match the style of previous releases. You can get the list of contributors by running `nox -s contributors -- v<prev-version> release-<BRANCH_VER>` to get contributors from the previous version tag through to the branch tip you just pushed. (This uses the GitHub API, so you'll need to push the branch first.)

Save as a draft and wait for now.

## 3. Leave for a cooling off period

Wait a couple of days in case anyone wants to hold up the release to add bugfixes etc.

## 4. Put live

To put live:
- 1. merge the release PR
- 2. publish a release on GitHub targeting the release branch

CI will automatically push to `crates.io`.

## 5. Tidy the main branch

If the release PR targeted a branch other than main, you will need to cherry-pick the version bumps, CHANGELOG modifications and removal of towncrier `newsfragments` and open another PR to land these on main.

## 6. Delete the release branch (patch releases only)

For 0.X.Y patch releases, the release branch is no longer needed, so it should be deleted.
