# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- **`GLOSSARY.md`** at the repo root, or
- **`GLOSSARY-MAP.md`** at the repo root if it exists: it points at one `GLOSSARY.md` per context. Read each one relevant to the topic.
- **`docs/adr/`**: read ADRs that touch the area you're about to work in. In multi-context repos, also check `cli/docs/adr/` and `crates/<crate>/docs/adr/` for context-scoped decisions.

If any of these files don't exist, **proceed silently**. Don't flag their absence; don't suggest creating them upfront. The `/domain-modeling` skill (reached via `/grill-with-docs` and `/improve-codebase-architecture`) creates them lazily when terms or decisions actually get resolved.

## File structure

Single-context repo (most repos):

```
/
├── GLOSSARY.md
├── docs/adr/
│   ├── 0001-event-sourced-orders.md
│   └── 0002-postgres-for-write-model.md
└── src/
```

Multi-context repo (presence of `GLOSSARY-MAP.md` at the root). This workspace uses this layout. A context is a domain area that may cover several crates; its `GLOSSARY.md` and `docs/adr/` live in its anchor crate, and `GLOSSARY-MAP.md` lists the crates each context covers:

```
/
├── GLOSSARY-MAP.md            ← lists every context and its GLOSSARY.md
├── docs/adr/                  ← workspace-wide decisions
├── cli/                       ← anchor of Tooling
│   ├── GLOSSARY.md
│   └── docs/adr/
└── crates/
    └── <crate>/               ← anchor of a context, e.g. lens → Lens
        ├── GLOSSARY.md
        └── docs/adr/          ← crate-specific decisions
```

## Spec contradictions

The specs in `docs/` sometimes disagree. Never pick a winner silently: put each contradiction to the developer, with a recommended answer.

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in `GLOSSARY.md`. Don't drift to synonyms the glossary explicitly avoids.

If the concept you need isn't in the glossary yet, that's a signal: either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/domain-modeling`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding:

> _Contradicts ADR-0007 (event-sourced orders), but worth reopening because…_
