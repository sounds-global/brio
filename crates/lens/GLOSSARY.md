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

**Provenance**:
The links from an element of a **Stage record** back to the source text and to the elements of the previous stage it derives from.
_Avoid_: lineage, ancestry

**Reason**:
The recorded justification for one decision a phase made, such as the constraints behind an inferred type.
_Avoid_: explanation, rationale

**Stage test**:
A comparison of a test program's **Stage records** against reviewed reference records.
_Avoid_: snapshot test, golden test
