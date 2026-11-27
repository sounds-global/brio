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
What an element or a **Reason** holds for each **Masked value**: the place where the value was, in its **Payload** or its justification. The record holds null at that place and nothing of the value. A marker is beside the payload, never inside it, and one place has one marker. A record made again from one that was read keeps its markers.
_Avoid_: placeholder, redaction

**Disabled writer**:
The record writer of a build with **Lens** off. It drops every record and does no work for it: it asks for neither the inputs of a record nor the elements of its payload.
_Avoid_: null writer, no-op writer

**Unfinished record**:
A record its writer is still writing, in a file with a name of that writer's alone. It gets the name of a record only once it is complete, so no reader is given half a record, and a reader passes it over until then.
_Avoid_: temporary file, partial record

**Records lock**:
The lock by which the readers and writers of the records of one build directory, in any number of processes, keep out of each other's way: a file beside the records, locked shared to list the records or to make a new one, and exclusive to be alone at work among them.
_Avoid_: build lock, mutex, lock file (for the lock itself)

**Stage test**:
A comparison of a test program's **Stage records** against reviewed reference records: a set of **Produced records** against a set of **Reference records**.
_Avoid_: snapshot test, golden test

**Record set**:
The records on one side of a **Stage test**, with at most one record of each **Stage**. A test has two: the **Produced records** and the **Reference records**.
_Avoid_: record collection, snapshot directory

**Produced record**:
A record that a build wrote and a **Stage test** checks.
_Avoid_: actual record, output record

**Reference record**:
A reviewed record that the **Produced record** of the same **Stage** is compared against. The two have the same module and **Payload kind**, and may have different **Record keys**.
_Avoid_: golden record, expected record, baseline
