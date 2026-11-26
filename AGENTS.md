# brio

## Workspace

- The root is a virtual workspace. `cli/` (`brio_cli`) is the only binary and builds the `brio` command; everything under `crates/` is a library.
- Crates live at `crates/<name>` with package name `brio_<name>` (underscores, not hyphens).
- Every compiler phase emits Lens records through `brio_lens`; see `crates/lens/docs/adr/`.

## Agent skills

### Issue tracker

Issues live in GitHub Issues for sounds-global/brio (`gh` CLI). See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary: needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix. See `docs/agents/triage-labels.md`.

### Domain docs

Multi-context: each crate is its own context; `GLOSSARY-MAP.md` at the root points to them. See `docs/agents/domain.md`.
