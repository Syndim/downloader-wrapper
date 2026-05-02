---
name: cargo-minor-release
description: Bump the current Rust/Cargo project's minor package version, commit the version change, and create a local git tag for the release commit. Use this skill whenever the user asks to bump, upgrade, release, or tag a Cargo/Rust project version, especially phrases like "upgrade minor version", "bump Cargo.toml version", "release this crate", "commit and tag a release", or "tag the version commit". This skill is for local release preparation only and must not push to remotes.
---

# Cargo Minor Release

Prepare a local Rust/Cargo release by bumping the package minor version, committing the version files, and adding a version tag to that commit.

## Release workflow

1. **Inspect the repository**
   - Run `git status --short`.
   - Run `git --no-pager diff -- Cargo.toml Cargo.lock`.
   - If there are unrelated uncommitted changes, do not stage or overwrite them. Continue only if the release can be made by editing/staging the version files without touching unrelated work.

2. **Read the current version**
   - Read the root `Cargo.toml`.
   - Find `[package].version`.
   - Parse it as semantic version `MAJOR.MINOR.PATCH`.
   - Compute the new minor version as `MAJOR.(MINOR + 1).0`.
   - If the user explicitly provides a target version, use it only if it is a valid semantic version and represents a version increase.

3. **Update version files**
   - Update `Cargo.toml` package `version` to the new version.
   - Run a Cargo command that already exists for the project and updates metadata safely, such as `cargo check` or the repository's normal validation command. This ensures `Cargo.lock` reflects the package version when applicable.
   - Do not manually edit dependency versions unless the user explicitly asks.

4. **Validate**
   - Run the repository's existing validation command when practical, typically `cargo test` for Rust projects.
   - If validation fails, stop and report the failure instead of committing.

5. **Commit**
   - Review the final diff with `git --no-pager diff -- Cargo.toml Cargo.lock`.
   - Stage only release version files: `Cargo.toml` and `Cargo.lock` if changed.
   - Commit with:
     ```bash
     git commit -m "chore(release): bump version to X.Y.Z"
     ```
   - Replace `X.Y.Z` with the new version.

6. **Tag**
   - Create a local annotated tag named `vX.Y.Z` on the release commit:
     ```bash
     git tag -a "vX.Y.Z" -m "Release X.Y.Z"
     ```
   - If the tag already exists, stop and report it instead of overwriting or moving it.

7. **Report outcome**
   - Include the new version, commit SHA, and tag name.
   - Mention that no push was performed.

## Safety rules

- Do not run `git push`.
- Do not amend, rebase, reset, or force-update tags.
- Do not stage unrelated files.
- Do not create a tag if the version commit was not created successfully.
- Stop and ask the user if the repository is not a Cargo project, has no root `Cargo.toml`, has an unparsable version, or has conflicting uncommitted release-file changes.

## Examples

**User:** "Bump the minor version and tag it."

**Expected behavior:** Change `Cargo.toml` from `0.3.5` to `0.4.0`, update `Cargo.lock` if needed, commit as `chore(release): bump version to 0.4.0`, then create annotated tag `v0.4.0`.

**User:** "Release version 1.8.0 for this crate."

**Expected behavior:** Verify `1.8.0` is a valid increase, update Cargo version files, commit as `chore(release): bump version to 1.8.0`, then create annotated tag `v1.8.0`.
