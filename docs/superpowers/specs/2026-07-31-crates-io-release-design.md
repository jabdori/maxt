# crates.io initial release design

## Goal

Publish `maxt` as `0.1.0` on crates.io and make the installation and project
rationale in the public documentation match that release.

## Chosen approach

Use a verified manual release for the first version. A minimal upload would
leave stale installation instructions and CI assertions behind. A release
automation workflow would add machinery before the project has a second
release to automate.

The release uses:

- crate version `0.1.0`;
- Git tag `v0.1.0` on the published commit;
- the crates.io dependency requirement `maxt = "0.1"`;
- the existing MIT license and `rust-version = "1.85"` declaration.

## Documentation

Add the approved convenience-focused project rationale to `README.md` and
`README.ko.md`. It will explain using several exchanges through one client
contract without mentioning Youngcha, CCXT, or comparative performance.

Replace the Git dependency and unpublished-package wording in both READMEs and
both getting-started guides with the crates.io dependency. Add a concise
`CHANGELOG.md` entry for `0.1.0` covering the initial public adapters, common
contract, and public live-verification scope.

## Package metadata and contents

Keep the existing name, version, description, license, repository, keywords,
categories, and readme metadata. Add explicit homepage and docs.rs URLs.
Exclude `.github` and `docs/superpowers` from the published archive because
they are repository workflow files rather than crate documentation.

The package must contain no local environment files, credentials, private
keys, or absolute contributor paths.

## CI

Remove the check that forbids crates.io installation instructions. Keep the
format, lint, panic, test, doctest, example, Rustdoc, and absolute-path checks.

## Release sequence

1. Apply the documentation, manifest, changelog, and CI changes.
2. Run the full repository checks and `cargo publish --dry-run --locked` from a
   clean worktree.
3. Commit the release and create the annotated `v0.1.0` tag on that commit.
4. Push `main` and the tag to GitHub.
5. Run `cargo publish --registry crates-io --locked` without bypass flags.
6. Confirm `maxt 0.1.0` through the crates.io index and check the docs.rs build
   status.

Publishing requires a crates.io API token supplied through Cargo's credential
store or environment. The token must not enter the repository, command history
arguments, documentation, commit, or tag message.

## Failure handling

- Do not use `--allow-dirty` or `--no-verify`.
- If upload succeeds but Cargo times out while polling the index, query the
  registry before retrying because published versions cannot be overwritten.
- If publication fails before upload, fix the release commit with a new commit
  and move the local tag before it has been pushed.
- If a defect is found after publication, do not delete or overwrite `0.1.0`;
  publish a corrected version and yank only when consumers must stop selecting
  the affected release.

## Acceptance criteria

- English and Korean rationale and installation sections have the same
  structure and meaning.
- `cargo test --all-targets`, doctests, Clippy, formatting, and Rustdoc pass.
- `cargo package --list` contains only intended distributable files.
- `cargo publish --dry-run --locked` succeeds from the release commit.
- GitHub has `v0.1.0` on the published commit.
- crates.io resolves `maxt 0.1.0`; docs.rs is either built or visibly queued.
