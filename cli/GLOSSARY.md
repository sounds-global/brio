# Tooling

The `brio` command and the editor-facing tools built on the compiler: diagnostics, formatting, the language server and the assistant server.

## Language

**Ownership overlay**:
The editor display that shows borrows, moves and drops inline in the source.
_Avoid_: ownership lens

**Code lens**:
The Language Server Protocol's clickable annotation above a line, such as "run benchmark". Always written in full; it is unrelated to **Lens**.
_Avoid_: lens
