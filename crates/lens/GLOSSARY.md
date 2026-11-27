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

**Payload**:
The part of a record that one phase defines in its own types: what that phase produced, or what a run did. Everything else in the record is shared by all phases.
_Avoid_: body, content

**Payload kind**:
The name that identifies one payload type in every record and in the published schema, such as `tokens`. Unique within a toolchain.
_Avoid_: stage kind, record type

**Envelope**:
The part of a record that every phase shares: its versions, its **Record key**, the module it is for, the **Payload kind** and a digest that guards the whole record. It wraps the **Payload**.
_Avoid_: header, wrapper

**Record key**:
The content-hash key a record is stored under: a hash of the stage, the module and its inputs, the toolchain version, the target and the build settings.
_Avoid_: cache key, record hash

**Stable identifier**:
The name of one element of a **Stage record**, derived from its module, the stage that produced it and its place in that stage's structure. The same in every build, whatever else changes in the input.
_Avoid_: element ID, node ID

**Provenance**:
The links from an element of a **Stage record** back to the source text and to the elements of the previous stage it derives from.
_Avoid_: lineage, ancestry

**Reason**:
The recorded justification for one decision a phase made, such as the constraints behind an inferred type.
_Avoid_: explanation, rationale

**Stage test**:
A comparison of a test program's **Stage records** against reviewed reference records.
_Avoid_: snapshot test, golden test
