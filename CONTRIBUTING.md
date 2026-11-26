# Contributing to Brio

Thank you for helping build Brio. This page explains how to propose changes and what every contribution needs.

## Ways to contribute

- **Report a bug or ask a question**: open a [GitHub issue](https://github.com/sounds-global/brio/issues).
- **Propose a language change**: open an issue describing the problem first. Language changes go through the public proposal process of the specification (Clause 29), and breaking changes only happen in new editions. See [GOVERNANCE.md](GOVERNANCE.md).
- **Fix a spec error**: typos, contradictions and unclear rules in `docs/` are welcome as pull requests. Cite the rule identifier, such as `[LENS-3]`.
- **Write code**: pick an issue labelled `ready-for-human` or `ready-for-agent`, or ask on an issue before starting something large.

Security problems are never reported in public issues. See [SECURITY.md](SECURITY.md).

## Developer Certificate of Origin

Every commit must be signed off under the [Developer Certificate of Origin](DCO) 1.1. By signing off, you certify that you wrote the change, or otherwise have the right to submit it under the project's licenses. Add the sign-off with `-s`:

```sh
git commit -s -m "lens: add stage record reader"
```

This adds a line with your real name and email:

```text
Signed-off-by: Jane Doe <jane@example.com>
```

A check on every pull request rejects commits without a sign-off. To fix earlier commits on your branch, run `git rebase --signoff main` and push again.

## Licenses of contributions

- Code is contributed under the Apache License 2.0 with LLVM Exceptions. Every source file starts with `// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception`.
- Changes to the specification and documentation in `docs/` are contributed under CC BY 4.0.
- Do not copy code from GPL, LGPL or AGPL projects, and list any new third-party dependency, with its license, in the pull request.

## Working on the code

The toolchain is a Rust workspace. You need Rust 1.96 or later.

```sh
cargo build
cargo test
cargo clippy --workspace
```

- `cli/` (`brio_cli`) builds the `brio` command. Everything under `crates/` is a library, at `crates/<name>` with the package name `brio_<name>`.
- The workspace is split into domain contexts. `GLOSSARY-MAP.md` lists them, and each context has its own `GLOSSARY.md` and decision records in `docs/adr/`. Use the glossary's terms in code, issues and pull requests.
- Every compiler phase emits Lens records in the same change that introduces it (specification, `[LENS-13]`).
- Workspace lints are strict (`missing_docs`, `unsafe_code = "forbid"`, and size limits in `clippy.toml`). `cargo clippy` must pass without warnings.

## Pull requests

1. Keep each pull request to one change, with tests.
2. Describe what changed and why, and link the issue it addresses.
3. Make sure `cargo test` and `cargo clippy --workspace` pass, and every commit is signed off.

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
