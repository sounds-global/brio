# Lens

Brio Lens records everything the toolchain does to a program, at every phase and during execution, so any result can be traced to its source and explained.

## Language

**Lens**:
The toolchain's recorder and viewer of **Stage records** and **Execution records**. Means only this: never an editor display.
_Avoid_: ownership lens, code lens (as a short form)

**Stage record**:
What one compiler phase produced for one module, with **Provenance** and **Reasons** for every element.
_Avoid_: dump, trace, snapshot

**Execution record**:
A replayable recording of one run under the interpreter or simulator, steppable forward and backward.
_Avoid_: trace, log

**Stage**:
One compiler phase at work on one module. It produces one **Stage record**.
_Avoid_: pass, step

**Element**:
One thing a stage produced, such as a token or a declaration. It has a **Stable identifier**, a **Payload**, **Provenance** and **Reasons**.
_Avoid_: node, item, entry

**Element table**:
The elements of one record, each under its **Stable identifier**. It is flat: an element names its parent and lists its children by stable identifier, whatever the structure of the stage.
_Avoid_: tree, element list

**Payload**:
The part of a record that one phase defines in its own types: what that phase says about each **Element**, or what a run did. Everything else in the record is shared by all phases.
_Avoid_: body, content

**Payload kind**:
The name that identifies one payload type in every record and in the published schema, such as `tokens`. Unique within a toolchain.
_Avoid_: stage kind, record type

**Envelope**:
The part of a record that every phase shares: its versions, its **Record key**, the module it is for, the **Payload kind**, the structure of the **Element table** and a digest that guards the whole record. It wraps the **Payload**.
_Avoid_: header, wrapper

**Record key**:
The content-hash key a record is stored under: a hash of the stage, the module and its inputs, the toolchain version, the target and the build settings.
_Avoid_: cache key, record hash

**Digest**:
The hash of a record's own text, written in the record, by which a reader detects a corrupted record. Unlike the **Record key**, it covers what the stage produced, not what it was computed from.
_Avoid_: checksum, content hash

**Stable identifier**:
The name of one element of a **Stage record**, derived from its module, the stage that produced it and its place in that stage's structure. The same in every build, whatever else changes in the input.
_Avoid_: element ID, node ID

**Provenance**:
The links from an element of a **Stage record** back to the source text and to the elements of the previous stage it derives from. An element the compiler creates also has a **Chain of origin**.
_Avoid_: lineage, ancestry

**Chain of origin**:
Every step that made a compiler-created element, such as inlining or monomorphization, from the last step back to an element that comes from source text.
_Avoid_: history, derivation

**Reason**:
The recorded justification for one decision a phase made about an element, such as the constraints behind an inferred type.
_Avoid_: explanation, rationale

**Masked value**:
A confidential value, such as one of a `Secret` or `Sensitive` type, that a phase keeps out of a record. The record holds a **Masked marker** in its place.
_Avoid_: redacted value, hidden value

**Masked marker**:
What a record holds in place of a **Masked value**. It says that a value was there and holds nothing of it.
_Avoid_: placeholder, redaction

**Disabled writer**:
The record writer of a build with **Lens** off. It drops every record and does no work for it.
_Avoid_: null writer, no-op writer

**Stage test**:
A comparison of a test program's **Stage records** against reviewed reference records.
_Avoid_: snapshot test, golden test
