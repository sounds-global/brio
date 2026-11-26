# Glossary Map

A context is a domain area, not a crate: one context may cover several crates. Its `GLOSSARY.md` and `docs/adr/` live in its anchor crate. Read the glossary of every context relevant to the topic.

## Contexts

- [Lens](./crates/lens/GLOSSARY.md): records what every phase and every run did, with provenance and reasons. Crates: `brio_lens` (anchor), later `brio_lens_view`.
- [Tooling](./cli/GLOSSARY.md): the `brio` command and the editor-facing tools. Crates: `brio_cli` (anchor).

## Relationships

- **Every phase → Lens**: each phase crate depends on `brio_lens` and defines its own record payloads; `brio_lens` depends on no other workspace crate.
- **Tooling → Lens**: `brio_cli` assembles the published record schema from every phase's payloads.

## Planned contexts

Add each one above when its first crate appears.

| Context | Covers |
| --- | --- |
| Syntax | tokens, error-recovering parser, syntax tree, grammar |
| Semantics | name resolution, types, ownership/borrow/origin, boundedness, capability flow |
| Verification | contracts, verification conditions, SMT, proof cache, proof checker |
| Comptime | comptime interpreter, embedded languages, tool extensions |
| Core | Brio Core elaboration, `brio interp` |
| Codegen | LLVM / Cranelift / TPDE back ends, linking, targets |
| Engine | query engine, early cutoff, build cache, `.brioh` interfaces |
| Interop | Rust bridge, C/C++ import, `brio bind`, Python |
| Testing | test runner, property tests, fuzzing, mutation, `sim`, `bench`, `ship` |
| Packages | resolver, registry, licensing, audit |
| Runtime | core/alloc/std, trap handler, confinement, telemetry |
