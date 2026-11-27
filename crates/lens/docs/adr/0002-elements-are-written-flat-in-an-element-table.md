# Elements are written flat, in an element table the envelope owns

A phase does not nest the structure of what it produced inside its payload. It gives Lens its elements flat, in the order it produced them. The envelope keeps them in an element table: each element is under its stable identifier, names its parent by stable identifier and lists its children in order. The payload is what the phase says about each element alone. Provenance and Reasons attach to an element, so they belong to the envelope too.

The rules of the table (stable identifiers are unique in a record, every named parent is in the record, no element is its own ancestor) are applied in one place, the same when a record is made as when it is read.

## Why

- Provenance links, Stage tests and the viewer find an element by stable identifier, including in a record whose payload type the reader does not know. A stable identifier that a payload embeds in its own JSON is invisible to Lens.
- A change to one element changes that element in the table, not the text of every ancestor, so a Stage test can report exactly what was added, removed and changed.
- Most stages are lists, tables or graphs, not trees.
- A JSON reader stops at a fixed depth. A nested syntax tree would reach it; a flat table never does.

## Considered Options

- **The payload is one opaque value that nests its own structure.** Rejected, for the reasons above. This was the first form of the envelope.
- **An element only names its parent, and the order of the table gives the order of children.** Rejected. The table is keyed by stable identifier so any JSON tool can find an element, and a keyed table has no order of its own. Each parent lists its children instead, so moving or adding a child changes its parent and nothing else.

## Consequences

- A payload type declares the schema of the payload of one element, not of a whole record.
- A link to an element of another record names the module, the payload kind and the stable identifier, so a reader can find the record without knowing its payload type.
- The parent and the list of children say the same thing twice. A reader rejects a record where they disagree.
