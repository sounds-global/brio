# Lens owns the record envelope; each phase owns its payloads

LENS-2 asks for one versioned, published schema for stage records. We do not put that whole schema in `brio_lens`. `brio_lens` owns only the envelope: stable identifiers, provenance, reasons, content-hash keys and versioning. Each phase crate (parser, type checker, interpreter, simulator, query engine, ...) defines its own payload types against that envelope. Execution records use the same envelope. `brio_cli`, the only crate that links every phase, assembles the published schema (`brio lens schema`): it chooses the set of payloads, and `brio_lens` combines that set with the envelope without knowing any payload type. Its version combines the envelope version with each phase's payload version.

## Considered Options

- **`brio_lens` owns every payload type.** Rejected. It would have to depend on, or duplicate, every phase's data types. That puts it at the top of the dependency graph, while LENS-13 needs every phase to depend on it.

## Consequences

- `brio_lens` must never depend on another workspace crate.
- A phase adds its payload types in the same change that adds the phase (LENS-13).
